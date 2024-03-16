use olympus_server::OlympusServer;

type Context = ();

async fn save_file((): Context, file: crate::File) -> bool {
	println!("{file:?}");
	true
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let mut server = OlympusServer::new(());
	server.register_callback("file", save_file).await;

	println!("Listening @ tcp://127.0.0.1:9999");
	server.run("127.0.0.1:9999".parse()?).await?;
	Ok(())
}

// olympus-compiler output below:

#[repr(i16)]
pub enum Action {
	Delete = 1,
	SecureDelete = 2,
	Encrypt = 3,
}

impl ::olympus_server::callback::CallbackInput for Action {
	fn deserialize(input: &mut ::bytes::BytesMut) -> Self {
		use ::bytes::Buf;
		let tag = input.get_u16();
		match tag {
			1 => Self::Delete,
			2 => Self::SecureDelete,
			3 => Self::Encrypt,
			_ => panic!("invalid tag: {tag}"),
		}
	}
}

impl ::olympus_server::callback::CallbackOutput for Action {
	fn serialize(self) -> ::bytes::BytesMut {
		use ::bytes::BufMut;
		let mut out = ::bytes::BytesMut::with_capacity(::std::mem::size_of::<u16>());
		out.put_u16(self as _);
		out
	}
}

#[derive(Debug, Clone)]
pub struct File {
	pub path: String,
	pub content: Vec<u8>,
}

impl ::olympus_server::callback::CallbackInput for File {
	fn deserialize(input: &mut ::bytes::BytesMut) -> Self {
		Self {
			path: ::olympus_server::callback::CallbackInput::deserialize(input),
			content: ::olympus_server::callback::CallbackInput::deserialize(input),
		}
	}
}

impl ::olympus_server::callback::CallbackOutput for File {
	fn serialize(self) -> ::bytes::BytesMut {
		let mut out = ::bytes::BytesMut::new();
		out.extend(self.path.serialize());
		out.extend(self.content.serialize());
		out
	}
}
