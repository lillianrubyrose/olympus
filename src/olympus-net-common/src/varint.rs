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
	use bytes::BytesMut;

	use crate::varint::{read_varint_i64, read_varint_u64, write_varint_i64, write_varint_u64};

	#[test]
	fn u64() {
		let mut out = BytesMut::new();
		write_varint_u64(u64::MAX, &mut out);
		assert_eq!(u64::MAX, read_varint_u64(&mut out));
	}

	#[test]
	fn i64() {
		let mut out = BytesMut::new();
		write_varint_i64(i64::MIN, &mut out);
		assert_eq!(i64::MIN, read_varint_i64(&mut out));
	}
}
