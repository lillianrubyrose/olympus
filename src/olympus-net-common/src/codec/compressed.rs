use std::mem::size_of;

use crate::bytes::BytesMut;
use lz4_flex::{block::decompress, compress};
use tokio_util::{
	bytes::{Buf, BufMut},
	codec::{Decoder, Encoder},
};

enum CompressedOlympusPacketCodecState {
	Header,
	Data {
		compressed: bool,
		length: u32,
		uncompressed_length: u32,
	},
}

pub struct CompressedOlympusPacketCodec {
	state: CompressedOlympusPacketCodecState,
	min_size_to_compress: usize,
}

impl CompressedOlympusPacketCodec {
	const MAX_PACKET_SIZE: u32 = 8 * 1024 * 1024;

	#[must_use]
	pub fn new(min_size: usize) -> Self {
		Self {
			state: CompressedOlympusPacketCodecState::Header,
			min_size_to_compress: min_size,
		}
	}
}

impl Default for CompressedOlympusPacketCodec {
	fn default() -> Self {
		Self {
			state: CompressedOlympusPacketCodecState::Header,
			min_size_to_compress: 8096,
		}
	}
}

impl Decoder for CompressedOlympusPacketCodec {
	type Item = BytesMut;
	type Error = std::io::Error;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		let (compressed, to_read, uncompressed_length) = match self.state {
			CompressedOlympusPacketCodecState::Header => {
				if src.len() < size_of::<u8>() + size_of::<u32>() {
					return Ok(None);
				}

				let compressed = src.get_u8() != 0;
				let length = src.get_u32();
				let uncompressed_length = if compressed { src.get_u32() } else { 0 };
				assert!(length <= Self::MAX_PACKET_SIZE, "packet too big");

				self.state = CompressedOlympusPacketCodecState::Data {
					compressed,
					length,
					uncompressed_length,
				};
				src.reserve(length as usize);
				(compressed, length, uncompressed_length)
			}
			CompressedOlympusPacketCodecState::Data {
				compressed,
				length,
				uncompressed_length,
			} => (compressed, length, uncompressed_length),
		};
		let to_read = to_read as usize;

		if src.len() < to_read {
			return Ok(None);
		}

		let data = if compressed {
			BytesMut::from_iter(
				decompress(&src.split_to(to_read), uncompressed_length as usize).map_err(|err| {
					dbg!(err);
					std::io::ErrorKind::InvalidData
				})?,
			)
		} else {
			src.split_to(to_read)
		};

		self.state = CompressedOlympusPacketCodecState::Header;
		Ok(Some(data))
	}
}

impl Encoder<BytesMut> for CompressedOlympusPacketCodec {
	type Error = std::io::Error;

	// aaaa clippy i dont care right now!
	#[allow(clippy::cast_possible_truncation)]
	fn encode(&mut self, item: BytesMut, dst: &mut BytesMut) -> Result<(), Self::Error> {
		let len = item.len() as u32;
		assert!(len < Self::MAX_PACKET_SIZE, "packet too big");

		let compressed = item.len() >= self.min_size_to_compress;

		dst.reserve(size_of::<u8>() + size_of::<u32>() + len as usize);
		dst.put_u8(u8::from(compressed));
		if compressed {
			let compressed = compress(&item);
			dst.put_u32(compressed.len() as u32);
			dst.put_u32(len);
			dst.extend(compressed);
		} else {
			dst.put_u32(len);
			dst.extend_from_slice(&item);
		}
		Ok(())
	}
}
