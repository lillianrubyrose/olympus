use crate::callback::{CallbackInput, CallbackOutput};

impl CallbackInput for String {
	fn deserialize(input: &[u8]) -> Self {
		String::from_utf8_lossy(input).into_owned()
	}
}

impl CallbackInput for () {
	fn deserialize(_input: &[u8]) -> Self {}
}

impl CallbackOutput for () {
	fn serialize(self) -> Vec<u8> {
		Vec::new()
	}
}

impl CallbackOutput for usize {
	fn serialize(self) -> Vec<u8> {
		usize::to_be_bytes(self).to_vec()
	}
}
