use olympus_net_common::{CallbackInput, CallbackOutput};

#[repr(i16)]
pub enum Action {
	Delete = 1,
	SecureDelete = 2,
	Encrypt = 3,
}

impl CallbackInput for Action {
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

impl CallbackOutput for Action {
	fn serialize(self) -> ::bytes::BytesMut {
		use ::bytes::BufMut;
		let mut out = ::bytes::BytesMut::with_capacity(::std::mem::size_of::<u16>());
		out.put_u16(self as _);
		out
	}
}

pub struct File {
	pub path: String,
	pub content: Vec<u8>,
}

impl CallbackInput for File {
	fn deserialize(input: &mut ::bytes::BytesMut) -> Self {
		Self {
			path: CallbackInput::deserialize(input),
			content: CallbackInput::deserialize(input),
		}
	}
}

impl CallbackOutput for File {
	fn serialize(self) -> ::bytes::BytesMut {
		let mut out = ::bytes::BytesMut::new();
		out.extend(self.path.serialize());
		out.extend(self.content.serialize());
		out
	}
}
