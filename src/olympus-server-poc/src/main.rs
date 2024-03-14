mod callback;
mod callback_ext;
mod fnv;

use std::collections::HashMap;

use callback::{Callback, CallbackHolder, CallbackInput, CallbackOutput};
use fnv::fnv;
use futures::Future;

struct OlympusServer<Ctx>
where
	Ctx: Clone,
{
	context: Ctx,
	handlers: HashMap<u64, Box<dyn Callback>>,
}

impl<Ctx> OlympusServer<Ctx>
where
	Ctx: Clone,
{
	pub fn new(context: Ctx) -> Self {
		Self {
			context,
			handlers: HashMap::default(),
		}
	}

	// TODO: this doesn't allow for functions without parameters and idk what to do.
	// i have spent far too long on this already!
	pub fn register_callback<F, Fut, Res, I>(&mut self, endpoint: &str, callback: F)
	where
		F: Fn(I) -> Fut + Clone + Send + Sync + 'static,
		Fut: Future<Output = Res> + Send + Sync,
		Res: CallbackOutput,
		I: CallbackInput + Send + Sync + 'static,
	{
		self.handlers
			.insert(fnv(endpoint), Box::new(CallbackHolder::new(callback)));
	}

	pub async fn run_callback(&mut self, endpoint: &str, input: &[u8]) -> Vec<u8> {
		let callback = self.handlers.get(&fnv(endpoint)).unwrap();
		callback.call(input).await
	}
}

async fn handler(input: String) {
	println!("{input}");
}

async fn handler2((): ()) -> usize {
	1
}

#[tokio::main]
async fn main() {
	let mut server = OlympusServer::new(());
	server.register_callback("user:1", handler);
	server.register_callback("user:2", handler2);

	dbg!(server.run_callback("user:1", "hello world".as_bytes()).await);
	dbg!(server.run_callback("user:2", &[]).await);
}
