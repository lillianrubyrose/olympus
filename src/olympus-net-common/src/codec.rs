use std::{io::ErrorKind, mem::size_of};

use crate::bytes::BytesMut;
use lz4_flex::block::{compress, decompress};
use tokio_util::{
	bytes::{Buf, BufMut},
	codec::{Decoder, Encoder},
};

enum OlympusPacketCodecState {
	Header,
	Data {
		compressed: bool,
		decompressed_length: usize,
		length: usize,
	},
}

struct OlympusCompressionSettings {
	enabled: bool,
	min_size_to_compress: u32,
}

pub struct OlympusPacketCodec {
	state: OlympusPacketCodecState,
	compression: OlympusCompressionSettings,
}

impl OlympusPacketCodec {
	const MAX_PACKET_SIZE: u32 = 8 * 1024 * 1024;

	#[must_use]
	pub fn compress(min_size_to_compress: u32) -> Self {
		Self {
			state: OlympusPacketCodecState::Header,
			compression: OlympusCompressionSettings {
				enabled: true,
				min_size_to_compress,
			},
		}
	}
}

impl Default for OlympusPacketCodec {
	fn default() -> Self {
		Self {
			state: OlympusPacketCodecState::Header,
			compression: OlympusCompressionSettings {
				enabled: false,
				min_size_to_compress: u32::MAX,
			},
		}
	}
}

impl Decoder for OlympusPacketCodec {
	type Item = BytesMut;
	type Error = std::io::Error;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		let (compressed, decompressed_length, length) = match self.state {
			OlympusPacketCodecState::Header => {
				if src.len() < size_of::<u8>() + size_of::<u32>() {
					return Ok(None);
				}

				let compressed = if self.compression.enabled {
					src.get_u8() != 0
				} else {
					false
				};

				let length = src.get_u32();
				assert!(length <= Self::MAX_PACKET_SIZE, "packet too big");
				let length = length as usize;

				let decompressed_length = if compressed { src.get_u32() } else { 0 } as usize;

				self.state = OlympusPacketCodecState::Data {
					compressed,
					decompressed_length,
					length,
				};
				src.reserve(length as usize);
				(compressed, decompressed_length as usize, length as usize)
			}
			OlympusPacketCodecState::Data {
				compressed,
				decompressed_length,
				length,
			} => (compressed, decompressed_length, length),
		};

		if src.len() < length {
			dbg!(length);
			dbg!(src.len());
			return Ok(None);
		}

		let data = if compressed {
			BytesMut::from_iter(
				decompress(&src.split_to(length), decompressed_length).map_err(|_| ErrorKind::InvalidData)?,
			)
		} else {
			src.split_to(length)
		};

		self.state = OlympusPacketCodecState::Header;
		Ok(Some(data))
	}
}

impl Encoder<BytesMut> for OlympusPacketCodec {
	type Error = std::io::Error;

	// aaaa clippy i dont care right now!
	#[allow(clippy::cast_possible_truncation)]
	fn encode(&mut self, item: BytesMut, dst: &mut BytesMut) -> Result<(), Self::Error> {
		let len = item.len() as u32;
		assert!(len < Self::MAX_PACKET_SIZE, "packet too big");

		dst.reserve(size_of::<u32>() * 2 + len as usize);
		if self.compression.enabled {
			dst.put_u8(u8::from(len >= self.compression.min_size_to_compress));
		}

		if self.compression.enabled && len >= self.compression.min_size_to_compress {
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
