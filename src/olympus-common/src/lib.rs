use std::ops::Range;

pub type SpannedErr = Spanned<String>;

#[derive(Debug, Clone)]
pub struct Spanned<T, S = Range<usize>> {
	pub value: T,
	pub span: S,
}

impl<T, S> Spanned<T, S> {
	#[must_use]
	pub fn new(value: T, span: S) -> Self {
		Self { value, span }
	}
}
