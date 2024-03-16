use std::mem::size_of;

use crate::bytes::BytesMut;
use tokio_util::{
	bytes::{Buf, BufMut},
	codec::{Decoder, Encoder},
};

enum OlympusPacketCodecState {
	Header,
	Data { length: u32 },
}

pub struct OlympusPacketCodec {
	state: OlympusPacketCodecState,
}

impl OlympusPacketCodec {
	const MAX_PACKET_SIZE: u32 = 8 * 1024 * 1024;
}

impl Default for OlympusPacketCodec {
	fn default() -> Self {
		Self {
			state: OlympusPacketCodecState::Header,
		}
	}
}

impl Decoder for OlympusPacketCodec {
	type Item = BytesMut;
	type Error = std::io::Error;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		let to_read = match self.state {
			OlympusPacketCodecState::Header => {
				if src.len() < size_of::<u32>() {
					return Ok(None);
				}

				let length = src.get_u32();
				dbg!(length);
				assert!(length <= Self::MAX_PACKET_SIZE, "packet too big");

				self.state = OlympusPacketCodecState::Data { length };
				src.reserve(length as usize);
				length
			}
			OlympusPacketCodecState::Data { length } => length,
		} as usize;

		if src.len() < to_read {
			return Ok(None);
		}

		self.state = OlympusPacketCodecState::Header;
		Ok(Some(src.split_to(to_read)))
	}
}

impl Encoder<BytesMut> for OlympusPacketCodec {
	type Error = std::io::Error;

	// aaaa clippy i dont care right now!
	#[allow(clippy::cast_possible_truncation)]
	fn encode(&mut self, item: BytesMut, dst: &mut BytesMut) -> Result<(), Self::Error> {
		let len = item.len() as u32;
		assert!(len < Self::MAX_PACKET_SIZE, "packet too big");

		dst.reserve(size_of::<u32>() + len as usize);
		dst.put_u32(len);
		dst.extend_from_slice(&item);
		Ok(())
	}
}
