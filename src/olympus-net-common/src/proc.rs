use std::mem::size_of;

use crate::bytes::{Buf, BufMut, BytesMut};
use crate::Result;

pub trait ProcedureInput {
	fn deserialize(input: &mut BytesMut) -> Result<Self>
	where
		Self: Sized;
}

pub trait ProcedureOutput {
	fn serialize(&self) -> Result<BytesMut>;
}

impl ProcedureInput for String {
	fn deserialize(input: &mut BytesMut) -> Result<Self> {
		let len = input.get_u32();
		Ok(String::from_utf8(input.split_to(len as usize).to_vec())?)
	}
}

impl ProcedureOutput for String {
	#[allow(clippy::cast_possible_truncation)]
	fn serialize(&self) -> Result<BytesMut> {
		let mut out = BytesMut::with_capacity(size_of::<u32>() + self.len());
		out.put_u32(self.len() as u32);
		out.extend(self.bytes());
		Ok(out)
	}
}

impl ProcedureInput for () {
	fn deserialize(_input: &mut BytesMut) -> Result<Self> {
		Ok(())
	}
}

impl ProcedureOutput for () {
	fn serialize(&self) -> Result<BytesMut> {
		Ok(BytesMut::new())
	}
}

impl<T: ProcedureInput> ProcedureInput for Vec<T> {
	fn deserialize(input: &mut BytesMut) -> Result<Self> {
		let len = input.get_u32() as usize;
		let mut vec = Vec::with_capacity(len);
		for _ in 0..len {
			vec.push(T::deserialize(input)?);
		}
		Ok(vec)
	}
}

impl<T: ProcedureOutput> ProcedureOutput for Vec<T> {
	#[allow(clippy::cast_possible_truncation)]
	fn serialize(&self) -> Result<BytesMut> {
		let mut buf = BytesMut::with_capacity((self.len() * size_of::<T>()) + size_of::<u32>());
		buf.put_u32(self.len() as u32);
		for ele in self {
			buf.extend(ele.serialize()?);
		}
		Ok(buf)
	}
}

impl ProcedureInput for bool {
	fn deserialize(input: &mut BytesMut) -> Result<Self> {
		Ok(input.get_u8() != 0)
	}
}

impl ProcedureOutput for bool {
	fn serialize(&self) -> Result<BytesMut> {
		let mut out = BytesMut::with_capacity(1);
		out.put_u8(u8::from(*self));
		Ok(out)
	}
}

macro_rules! impl_for_nums {
	($($ty:ty),*) => {
		$(
			impl ProcedureInput for $ty {
				fn deserialize(input: &mut BytesMut) -> Result<Self> {
					::paste::paste! { Ok(input.[<get_$ty>]()) }
				}
			}

			impl ProcedureOutput for $ty {
				fn serialize(&self) -> Result<BytesMut> {
					Ok(BytesMut::from_iter(self.to_be_bytes()))
				}
			}
		)*
	};
}

impl_for_nums!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

impl<T: ProcedureInput> ProcedureInput for Option<T> {
	fn deserialize(input: &mut BytesMut) -> Result<Self> {
		let present = input.get_u8();
		if present == 0 {
			return Ok(None);
		}
		Ok(Some(T::deserialize(input)?))
	}
}

impl<T: ProcedureOutput> ProcedureOutput for Option<T> {
	fn serialize(&self) -> Result<BytesMut> {
		let mut out = BytesMut::new();
		out.put_u8(u8::from(self.is_some()));
		if let Some(inner) = self {
			out.extend(inner.serialize()?);
		}
		Ok(out)
	}
}
