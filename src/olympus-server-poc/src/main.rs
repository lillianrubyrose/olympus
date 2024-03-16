mod callback;
mod callback_ext;
mod codec;
mod fnv;

use std::{
	collections::{HashMap, HashSet},
	net::SocketAddr,
	sync::Arc,
};

use bytes::{Buf, BytesMut};
use callback::{Callback, CallbackHolder, CallbackInput, CallbackOutput};
use codec::OlympusPacketCodec;
use fnv::fnv;
use futures::{Future, SinkExt, StreamExt};
use tokio::{
	net::{TcpListener, TcpStream},
	sync::Mutex,
};
use tokio_util::codec::{FramedRead, FramedWrite};

type ArcMut<T> = Arc<Mutex<T>>;
type HandlersMap<Ctx> = HashMap<u64, (Box<dyn Callback<Ctx>>, &'static str)>;

struct OlympusServer<Ctx>
where
	Ctx: Clone + Send + Sync,
{
	context: Ctx,
	handlers: ArcMut<HandlersMap<Ctx>>,
	connected_clients: ArcMut<HashSet<u64>>,
}

impl<Ctx> OlympusServer<Ctx>
where
	Ctx: Clone + Send + Sync + 'static,
{
	pub fn new(context: Ctx) -> Self {
		Self {
			context,
			handlers: Arc::default(),
			connected_clients: Arc::default(),
		}
	}

	// TODO: this doesn't allow for functions without parameters and idk what to do.
	// i have spent far too long on this already!
	pub async fn register_callback<F, Fut, Res, I>(&mut self, endpoint: &'static str, callback: F)
	where
		F: Fn(Ctx, I) -> Fut + Clone + Send + Sync + 'static,
		Fut: Future<Output = Res> + Send + Sync,
		Res: CallbackOutput,
		I: CallbackInput + Clone + Send + Sync + 'static,
	{
		let endpoint_hash = fnv(endpoint);
		dbg!(endpoint_hash);
		self.handlers
			.lock()
			.await
			.insert(endpoint_hash, (Box::new(CallbackHolder::new(callback)), endpoint));
	}

	pub async fn run(&mut self, addr: SocketAddr) -> std::io::Result<()> {
		let tcp = TcpListener::bind(addr).await?;

		loop {
			let (stream, _) = match tcp.accept().await {
				Ok(pair) => pair,
				Err(err) => {
					eprintln!("Failed to accept connection: {err}");
					continue;
				}
			};

			let ctx = self.context.clone();
			let connected_clients = self.connected_clients.clone();
			let handlers = self.handlers.clone();
			tokio::spawn(async move {
				let session_id = lid::easy::generate_random();
				let session_id = fnv(&session_id);
				connected_clients.lock().await.insert(session_id);

				if let Err(err) = Self::handle_connection(ctx, handlers, session_id, stream).await {
					eprintln!("Err handling connection: {err}");
				}

				connected_clients.lock().await.remove(&session_id);
			});
		}
	}

	async fn handle_connection(
		context: Ctx,
		handlers: ArcMut<HandlersMap<Ctx>>,
		_: u64,
		stream: TcpStream,
	) -> std::io::Result<()> {
		let (r, w) = stream.into_split();
		let mut framed_read = FramedRead::new(r, OlympusPacketCodec::default());
		let mut framed_write = FramedWrite::new(w, OlympusPacketCodec::default());

		while let Some(frame) = framed_read.next().await {
			let mut frame = frame?;
			let endpoint_hash = frame.get_u64();

			match Self::run_callback(context.clone(), handlers.clone(), endpoint_hash, &frame).await {
				Some((response, endpoint_name)) if !response.is_empty() => {
					println!("callback '{endpoint_name}' has response");
					let mut output = BytesMut::with_capacity(response.len());
					output.extend_from_slice(&response);
					framed_write.send(output).await?;
				}
				Some((_, endpoint_name)) => {
					println!("callback '{endpoint_name}' has no response");
				}
				None => {
					eprintln!("callback with hash ({endpoint_hash}) not found");
				}
			}
		}

		Ok(())
	}

	async fn run_callback(
		context: Ctx,
		handlers: ArcMut<HandlersMap<Ctx>>,
		endpoint_hash: u64,
		input: &[u8],
	) -> Option<(Vec<u8>, &'static str)> {
		let callback = handlers.lock().await;
		let (callback, endpoint_name) = callback.get(&endpoint_hash)?;
		Some((callback.call(context, input).await, endpoint_name))
	}
}

type Context = ();

async fn printer_callback((): Context, input: String) {
	println!("{input}");
}

async fn return_str_callback((): Context, (): ()) -> String {
	"OLYMPUS".into()
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let mut server = OlympusServer::new(());
	server.register_callback("sample", printer_callback).await;
	server.register_callback("sample:ret", return_str_callback).await;

	println!("Listening @ tcp://127.0.0.1:9999");
	server.run("127.0.0.1:9999".parse()?).await?;
	Ok(())
}
