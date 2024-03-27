pub mod procedure;

use std::{
	collections::{HashMap, HashSet},
	mem::size_of,
	net::SocketAddr,
	sync::Arc,
};

use futures::{Future, SinkExt, StreamExt};
use olympus_net_common::{
	bytes::{Buf, BytesMut},
	fnv, OlympusPacketCodec, ProcedureInput, ProcedureOutput,
};
use procedure::{Procedure, ProcedureHolder};
use tokio::{
	net::{TcpListener, TcpStream},
	sync::Mutex,
};
use tokio_util::{
	bytes::BufMut,
	codec::{FramedRead, FramedWrite},
};

type ArcMut<T> = Arc<Mutex<T>>;
type HandlersMap<Ctx> = HashMap<u64, (Box<dyn Procedure<Ctx>>, &'static str)>;

pub struct OlympusServer<Ctx>
where
	Ctx: Clone + Send + Sync,
{
	context: Ctx,
	procedures: ArcMut<HandlersMap<Ctx>>,
	connected_clients: ArcMut<HashSet<u64>>,
}

impl<Ctx> OlympusServer<Ctx>
where
	Ctx: Clone + Send + Sync + 'static,
{
	pub fn new(context: Ctx) -> Self {
		Self {
			context,
			procedures: Arc::default(),
			connected_clients: Arc::default(),
		}
	}

	pub async fn register_procedure<F, Fut, Res, I>(&mut self, name: &'static str, procedure_fn: F)
	where
		F: Fn(Ctx, I) -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Res> + Send,
		Res: ProcedureOutput,
		I: ProcedureInput + Send + Sync + 'static,
	{
		self.procedures
			.lock()
			.await
			.insert(fnv(name), (Box::new(ProcedureHolder::new(procedure_fn)), name));
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
			let procedures = self.procedures.clone();
			tokio::spawn(async move {
				let session_id = lid::easy::generate_random();
				let session_id = fnv(&session_id);
				connected_clients.lock().await.insert(session_id);

				if let Err(err) = Self::handle_connection(ctx, procedures, session_id, stream).await {
					eprintln!("Err handling connection: {err}");
				}

				connected_clients.lock().await.remove(&session_id);
			});
		}
	}

	async fn handle_connection(
		context: Ctx,
		procedures: ArcMut<HandlersMap<Ctx>>,
		_: u64,
		stream: TcpStream,
	) -> std::io::Result<()> {
		let (r, w) = stream.into_split();
		let mut framed_read = FramedRead::new(r, OlympusPacketCodec::compress(8192));
		let mut framed_write = FramedWrite::new(w, OlympusPacketCodec::compress(8192));

		while let Some(frame) = framed_read.next().await {
			let mut frame = frame?;
			let procedure_name_hash = frame.get_u64();

			match Self::run_procedure(context.clone(), procedures.clone(), procedure_name_hash, frame).await {
				Some((response, procedure_name)) if !response.is_empty() => {
					println!("Procedure '{procedure_name}' has response");
					let mut out = BytesMut::new();
					out.reserve(size_of::<u64>() + response.len());
					out.put_u64(procedure_name_hash);
					out.extend(response);

					framed_write.send(out).await?;
				}
				Some((_, procedure_name)) => {
					println!("Procedure '{procedure_name}' has no response");
				}
				None => {
					eprintln!("Procedure with hash ({procedure_name_hash}) not found");
				}
			}
		}

		Ok(())
	}

	async fn run_procedure(
		context: Ctx,
		procedures: ArcMut<HandlersMap<Ctx>>,
		name_hash: u64,
		input: BytesMut,
	) -> Option<(BytesMut, &'static str)> {
		let procedure = procedures.lock().await;
		let (procedure, name) = procedure.get(&name_hash)?;
		Some((procedure.call(context, input).await, name))
	}
}
