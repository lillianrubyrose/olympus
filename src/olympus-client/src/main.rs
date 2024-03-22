mod models;

use futures::{SinkExt, StreamExt};
use olympus_net_common::{
	bytes::{Buf, BufMut, BytesMut},
	fnv, CompressedOlympusPacketCodec, ProcedureOutput,
};
use rand::{thread_rng, RngCore};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let stream = TcpStream::connect("127.0.0.1:9999").await?;
	let (r, w) = stream.into_split();
	let mut framed_read = FramedRead::new(r, CompressedOlympusPacketCodec::default());
	let mut framed_write = FramedWrite::new(w, CompressedOlympusPacketCodec::default());

	let procedure_hash = fnv("file");
	let mut bytes = vec![0; 1024 * 10];
	let mut rng = thread_rng();
	rng.fill_bytes(&mut bytes);
	let param = models::File {
		path: "/home/lily/puppywoofwoof.txt".into(),
		content: bytes,
	}
	.serialize();

	let mut to_send = BytesMut::new();
	to_send.put_u64(procedure_hash);
	to_send.extend(param);

	framed_write.send(to_send).await?;

	let Some(Ok(mut frame)) = framed_read.next().await else {
		panic!("couldn't read frame");
	};

	println!("result = {}", frame.get_u8());

	Ok(())
}
