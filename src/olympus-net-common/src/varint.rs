use std::ops::Deref;

use bytes::{Buf, BufMut, BytesMut};
use zigzag::{ZigZagDecode, ZigZagEncode};

pub struct Variable<T>(pub T);

impl<T> Deref for Variable<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

macro_rules! impl_for_unsigned {
	($($ty:ty),*) => {
		::paste::paste! {
            $(
                impl crate::ProcedureInput for Variable<$ty> {
                    fn deserialize(input: &mut BytesMut) -> Self {
                        Variable([<read_varint_$ty>](input))
                    }
                }

                impl crate::ProcedureOutput for Variable<$ty> {
                    fn serialize(self) -> BytesMut {
                        let mut out = BytesMut::new();
                        [<write_varint_$ty>](*self, &mut out);
                        out
                    }
                }

			    pub fn [<write_varint_$ty>](mut input: $ty, out: &mut BytesMut) {
                    if input == 0 {
                        out.put_u8(0);
                        return;
                    }

                    while input >= 0b1000_0000 {
                        let curr = u8::try_from((input & 0b0111_1111) | 0b1000_0000).unwrap();
			    		input >>= 7;

                        out.put_u8(curr);
                    }

                    out.put_u8(u8::try_from(input & 0b0111_1111).unwrap());
			    }

                pub fn [<read_varint_$ty>](out: &mut BytesMut) -> $ty {
                    let mut res: $ty = 0;
                    let mut idx = 0;
                    loop {
                        let byte = out.get_u8();
                        res |= $ty::try_from(byte & 0b0111_1111).unwrap() << (7 * idx);
                        idx += 1;
                        if byte & 0b1000_0000 == 0 {
                            break;
                        }
                    }
                    res
                }
            )*
		}
	};
}

macro_rules! impl_for_signed {
    ($($bits:expr),*) => {
        ::paste::paste! {
            $(
                impl crate::ProcedureInput for Variable<[<i$bits>]> {
                    fn deserialize(input: &mut BytesMut) -> Self {
                        Variable([<read_varint_i$bits>](input))
                    }
                }

                impl crate::ProcedureOutput for Variable<[<i$bits>]> {
                    fn serialize(self) -> BytesMut {
                        let mut out = BytesMut::new();
                        [<write_varint_i$bits>](*self, &mut out);
                        out
                    }
                }

                pub fn [<write_varint_i$bits>](input: [<i$bits>], out: &mut BytesMut) {
                    [<write_varint_u$bits>](input.zigzag_encode(), out);
                }

                pub fn [<read_varint_i$bits>](out: &mut BytesMut) -> [<i$bits>] {
                    [<read_varint_u$bits>](out).zigzag_decode()
                }
            )*
        }
};
}

impl_for_unsigned!(u8, u16, u32, u64, u128);
impl_for_signed!(8, 16, 32, 64, 128);

#[cfg(test)]
mod tests {
	use std::fmt::Debug;

	use bytes::BytesMut;

	use crate::{
		read_varint_i128, read_varint_i32, read_varint_i64, read_varint_i8, read_varint_u128, read_varint_u16,
		read_varint_u32, read_varint_u64, read_varint_u8, write_varint_i128, write_varint_i32, write_varint_i64,
		write_varint_i8, write_varint_u128, write_varint_u16, write_varint_u32, write_varint_u64, write_varint_u8,
	};

	fn test_write_read<T: Debug + PartialEq + Copy>(
		value: T,
		write_fn: fn(T, &mut BytesMut),
		read_fn: fn(&mut BytesMut) -> T,
	) {
		let mut out = BytesMut::new();
		write_fn(value, &mut out);
		assert_eq!(value, read_fn(&mut out));
	}

	#[test]
	fn test_u8() {
		test_write_read(0u8, write_varint_u8, read_varint_u8);
		test_write_read(u8::MAX, write_varint_u8, read_varint_u8);
	}

	#[test]
	fn test_i8() {
		test_write_read(i8::MIN, write_varint_i8, read_varint_i8);
		test_write_read(0i8, write_varint_i8, read_varint_i8);
		test_write_read(i8::MAX, write_varint_i8, read_varint_i8);
	}

	#[test]
	fn test_u16() {
		test_write_read(0u16, write_varint_u16, read_varint_u16);
		test_write_read(u16::MAX, write_varint_u16, read_varint_u16);
	}

	#[test]
	fn test_u32() {
		test_write_read(0u32, write_varint_u32, read_varint_u32);
		test_write_read(u32::MAX, write_varint_u32, read_varint_u32);
	}

	#[test]
	fn test_i32() {
		test_write_read(i32::MIN, write_varint_i32, read_varint_i32);
		test_write_read(0i32, write_varint_i32, read_varint_i32);
		test_write_read(i32::MAX, write_varint_i32, read_varint_i32);
	}

	#[test]
	fn test_u64() {
		test_write_read(0u64, write_varint_u64, read_varint_u64);
		test_write_read(u64::MAX, write_varint_u64, read_varint_u64);
	}

	#[test]
	fn test_i64() {
		test_write_read(i64::MIN, write_varint_i64, read_varint_i64);
		test_write_read(0i64, write_varint_i64, read_varint_i64);
		test_write_read(i64::MAX, write_varint_i64, read_varint_i64);
	}

	#[test]
	fn test_u128() {
		test_write_read(0u128, write_varint_u128, read_varint_u128);
		test_write_read(u128::MAX, write_varint_u128, read_varint_u128);
	}

	#[test]
	fn test_i128() {
		test_write_read(i128::MIN, write_varint_i128, read_varint_i128);
		test_write_read(0i128, write_varint_i128, read_varint_i128);
		test_write_read(i128::MAX, write_varint_i128, read_varint_i128);
	}
}
