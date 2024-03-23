use std::{collections::HashMap, marker::PhantomData, mem::size_of, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use futures::{Future, SinkExt, StreamExt};
use olympus_net_common::{fnv, OlympusPacketCodec, ProcedureInput, ProcedureOutput};
use tokio::{
	io,
	net::{
		tcp::{OwnedReadHalf, OwnedWriteHalf},
		TcpStream,
	},
	sync::{
		mpsc::{UnboundedReceiver, UnboundedSender},
		Mutex,
	},
};
use tokio_util::{
	bytes::{Buf, BufMut, BytesMut},
	codec::{FramedRead, FramedWrite},
};

#[async_trait]
trait ResponseHandler<Ctx>: Send + Sync {
	async fn call(&self, client: OlympusClient<Ctx>, input: BytesMut);
}

#[derive(Clone)]
struct ProcedureHolder<F, T>(F, PhantomData<T>);

impl<F, T> ProcedureHolder<F, T> {
	pub fn new(procedure: F) -> Self {
		Self(procedure, PhantomData)
	}
}

#[async_trait]
impl<Ctx, F, Fut, I> ResponseHandler<Ctx> for ProcedureHolder<F, I>
where
	Ctx: Send + Sync + 'static,
	F: Fn(OlympusClient<Ctx>, I) -> Fut + Clone + Send + Sync + 'static,
	Fut: Future<Output = ()> + Send,
	I: ProcedureInput + Send + Sync,
{
	async fn call(&self, client: OlympusClient<Ctx>, mut input: BytesMut) {
		self.0(client, I::deserialize(&mut input)).await;
	}
}

type ArcMut<T> = Arc<Mutex<T>>;
type HandlersMap<Ctx> = HashMap<u64, (Box<dyn ResponseHandler<Ctx>>, &'static str)>;

#[derive(Clone)]
pub struct OlympusClient<Ctx> {
	pub context: Ctx,
	response_handlers: ArcMut<HandlersMap<Ctx>>,
	sender: Arc<Option<UnboundedSender<(&'static str, BytesMut)>>>,
}

impl<Ctx: Clone + Send + Sync + 'static> OlympusClient<Ctx> {
	#[must_use]
	pub fn new(context: Ctx) -> Self {
		Self {
			context,
			response_handlers: Arc::default(),
			sender: Arc::new(None),
		}
	}

	pub async fn on_response<F, Fut, I>(&mut self, procedure_name: &'static str, handler: F)
	where
		F: Fn(OlympusClient<Ctx>, I) -> Fut + Clone + Send + Sync + 'static,
		Fut: Future<Output = ()> + Send + Sync,
		I: ProcedureInput + Clone + Send + Sync + 'static,
	{
		self.response_handlers.lock().await.insert(
			fnv(procedure_name),
			(Box::new(ProcedureHolder::new(handler)), procedure_name),
		);
	}

	pub fn send<I: ProcedureOutput + Send + Sync + 'static>(
		&mut self,
		procedure_name: &'static str,
		input: &I,
	) -> eyre::Result<()> {
		(*self.sender)
			.as_ref()
			.unwrap()
			.send((procedure_name, input.serialize()))?;
		Ok(())
	}

	pub async fn connect(&mut self, addr: SocketAddr) -> io::Result<()> {
		let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
		self.sender = Arc::new(Some(tx.clone()));

		let stream = TcpStream::connect(addr).await?;
		let (r, w) = stream.into_split();
		let framed_read = FramedRead::new(r, OlympusPacketCodec::compress(8192));
		let framed_write = FramedWrite::new(w, OlympusPacketCodec::compress(8192));
		tokio::spawn(Self::handle_outgoing(rx, framed_write));
		tokio::spawn(Self::handle_incoming(
			self.clone(),
			self.response_handlers.clone(),
			framed_read,
		));

		Ok(())
	}

	async fn handle_outgoing(
		mut rx: UnboundedReceiver<(&'static str, BytesMut)>,
		mut write: FramedWrite<OwnedWriteHalf, OlympusPacketCodec>,
	) {
		while let Some((procedure_name, data)) = rx.recv().await {
			let mut to_send = BytesMut::new();
			to_send.reserve(size_of::<u64>() + data.len());
			to_send.put_u64(fnv(procedure_name));
			to_send.extend(data);

			if let Err(err) = write.send(to_send).await {
				eprintln!("Error when sending message: {err}");
			}
		}
	}

	async fn handle_incoming(
		client: OlympusClient<Ctx>,
		handlers: ArcMut<HandlersMap<Ctx>>,
		mut read: FramedRead<OwnedReadHalf, OlympusPacketCodec>,
	) {
		while let Some(frame) = read.next().await {
			let mut frame = frame.unwrap();
			let procedure_name_hash = frame.get_u64();

			if Self::run_handler(client.clone(), handlers.clone(), procedure_name_hash, frame)
				.await
				.is_none()
			{
				eprintln!("Handler for hash ({procedure_name_hash}) not found");
			}
		}
	}

	async fn run_handler(
		client: OlympusClient<Ctx>,
		handlers: ArcMut<HandlersMap<Ctx>>,
		name_hash: u64,
		input: BytesMut,
	) -> Option<&'static str> {
		let procedure = handlers.lock().await;
		let (procedure, name) = procedure.get(&name_hash)?;
		procedure.call(client, input).await;
		Some(name)
	}
}
