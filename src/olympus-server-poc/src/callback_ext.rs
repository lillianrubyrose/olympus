use crate::callback::{CallbackInput, CallbackOutput};

impl CallbackInput for String {
	fn deserialize(raw_input: &[u8]) -> Self {
		String::from_utf8_lossy(raw_input).into_owned()
	}
}

impl CallbackOutput for usize {
	fn serialize(self) -> Vec<u8> {
		usize::to_be_bytes(self).to_vec()
	}
}
