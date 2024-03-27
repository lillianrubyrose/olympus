use olympus_net_common::Variable;
use olympus_server::OlympusServer;

type Context = ();

async fn get_file((): Context, params: GetFileParams) -> File {
	dbg!(params.after_action);

	let content = tokio::fs::read(&params.path).await.unwrap();
	File {
		path: params.path,
		file_size: Variable(content.len() as u64),
		content,
	}
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let mut server = OlympusServer::new(());
	server.register_procedure("getFile", get_file).await;

	println!("Listening @ tcp://127.0.0.1:9999");
	server.run("127.0.0.1:9999".parse()?).await?;
	Ok(())
}

// olympus-compiler output below:

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum Action {
	Delete = 1,
	SecureDelete = 2,
	Encrypt = 3,
}

impl ::olympus_net_common::ProcedureInput for Action {
	fn deserialize(input: &mut ::olympus_net_common::bytes::BytesMut) -> Self {
		use ::olympus_net_common::bytes::Buf;
		let tag = input.get_u16();
		match tag {
			1 => Self::Delete,
			2 => Self::SecureDelete,
			3 => Self::Encrypt,
			_ => panic!("invalid tag: {tag}"),
		}
	}
}

impl ::olympus_net_common::ProcedureOutput for Action {
	fn serialize(&self) -> ::olympus_net_common::bytes::BytesMut {
		use ::olympus_net_common::bytes::BufMut;
		let mut out = ::olympus_net_common::bytes::BytesMut::with_capacity(::std::mem::size_of::<u16>());
		out.put_u16(*self as _);
		out
	}
}

#[derive(Debug, Clone)]
pub struct File {
	pub path: String,
	pub file_size: ::olympus_net_common::Variable<u64>,
	pub content: Vec<u8>,
}

impl ::olympus_net_common::ProcedureInput for File {
	fn deserialize(input: &mut ::olympus_net_common::bytes::BytesMut) -> Self {
		Self {
			path: ::olympus_net_common::ProcedureInput::deserialize(input),
			file_size: ::olympus_net_common::ProcedureInput::deserialize(input),
			content: ::olympus_net_common::ProcedureInput::deserialize(input),
		}
	}
}

impl ::olympus_net_common::ProcedureOutput for File {
	fn serialize(&self) -> ::olympus_net_common::bytes::BytesMut {
		let mut out = ::olympus_net_common::bytes::BytesMut::new();
		out.extend(self.path.serialize());
		out.extend(self.file_size.serialize());
		out.extend(self.content.serialize());
		out
	}
}

#[derive(Debug, Clone)]
pub struct GetFileParams {
	path: String,
	after_action: Option<Action>,
}

impl ::olympus_net_common::ProcedureInput for GetFileParams {
	fn deserialize(input: &mut ::olympus_net_common::bytes::BytesMut) -> Self {
		Self {
			path: ::olympus_net_common::ProcedureInput::deserialize(input),
			after_action: ::olympus_net_common::ProcedureInput::deserialize(input),
		}
	}
}

impl ::olympus_net_common::ProcedureOutput for GetFileParams {
	fn serialize(&self) -> ::olympus_net_common::bytes::BytesMut {
		let mut out = ::olympus_net_common::bytes::BytesMut::new();
		out.extend(self.path.serialize());
		out.extend(self.after_action.serialize());
		out
	}
}
