use crate::{ErrorKind, ParseResult, Reader};

pub trait Leb128: Sized {
    fn decode<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self>;
}

macro_rules! impl_leb_unsigned {
	($($ty:ty),*) => {
		$(
			impl Leb128 for $ty {
				fn decode<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
					let mark = reader.mark();
					let mut result: $ty = 0;
					let mut shift: $ty = 0;
					const MAXBITS: usize = core::mem::size_of::<$ty>() * 8;
					loop {
						let byte = reader.read_u8()?;
						result |= (byte & 0x7F) as $ty << shift;
						shift += 7;
						if (byte & 0x80) == 0 {
							return Ok(result);
						}
						if shift as usize > MAXBITS {
							return Err(mark.to_error(ErrorKind::IntegerOverflow(core::any::type_name::<$ty>())));
						}
					}
				}
			}
		)*
	}
}

impl_leb_unsigned!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_leb_signed {
	($($ty:ty),*) => {
		$(
			impl Leb128 for $ty {
				fn decode<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
					let mark = reader.mark();
					let mut result: $ty = 0;
					let mut shift: $ty = 0;
					const MAXBITS: usize = core::mem::size_of::<$ty>() * 8;
					loop {
						let byte = reader.read_u8()?;
						result |= (byte & 0x7F) as $ty << shift;
						shift += 7;
						if (byte & 0x80) == 0 {
							if (shift as usize) < MAXBITS && (byte & 0x40) != 0 {
								result |= <$ty>::MAX << shift;
							}
							return Ok(result);
						}
						if shift as usize > MAXBITS {
							return Err(mark.to_error(ErrorKind::IntegerOverflow(core::any::type_name::<$ty>())));
						}
					}
				}
			}
		)*
	}
}

impl_leb_signed!(i8, i16, i32, i64, i128, isize);
