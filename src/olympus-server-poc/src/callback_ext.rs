use bytes::BytesMut;

use crate::callback::{CallbackInput, CallbackOutput};

impl CallbackInput for String {
	fn deserialize(input: BytesMut) -> Self {
		String::from_utf8_lossy(&input).into_owned()
	}
}

impl CallbackInput for () {
	fn deserialize(_input: BytesMut) -> Self {}
}

impl CallbackOutput for () {
	fn serialize(self) -> BytesMut {
		BytesMut::new()
	}
}

impl CallbackOutput for usize {
	fn serialize(self) -> BytesMut {
		BytesMut::from_iter(usize::to_be_bytes(self))
	}
}

impl CallbackOutput for String {
	fn serialize(self) -> BytesMut {
		self.bytes().collect::<BytesMut>()
	}
}
