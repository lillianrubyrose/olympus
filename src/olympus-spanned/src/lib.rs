pub use ariadne::Color as ErrorColor;
use std::ops::Range;

pub struct OlympusLabeledError {
	pub message: String,
	pub span: Range<usize>,
	pub color: ErrorColor,
}

pub struct OlympusError {
	pub subject: String,
	pub labels: Vec<OlympusLabeledError>,
}

impl OlympusError {
	pub fn error<S: ToOwned<Owned = String> + ?Sized>(subject: &S, span: Range<usize>) -> Self {
		Self {
			subject: subject.to_owned(),
			labels: vec![OlympusLabeledError {
				message: subject.to_owned(),
				span,
				color: ErrorColor::Red,
			}],
		}
	}

	pub fn new<S: ToOwned<Owned = String> + ?Sized>(subject: &S) -> Self {
		Self {
			subject: subject.to_owned(),
			labels: vec![],
		}
	}

	#[must_use]
	pub fn span(mut self, span: Range<usize>, color: ErrorColor) -> Self {
		self.labels.push(OlympusLabeledError {
			message: self.subject.clone(),
			span,
			color,
		});
		self
	}

	#[must_use]
	pub fn label<S: ToOwned<Owned = String> + ?Sized>(
		mut self,
		message: &S,
		span: Range<usize>,
		color: ErrorColor,
	) -> Self {
		self.labels.push(OlympusLabeledError {
			message: message.to_owned(),
			span,
			color,
		});
		self
	}
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
	pub value: T,
	pub span: Range<usize>,
}

impl<T> Spanned<T> {
	#[must_use]
	pub fn new(value: T, span: Range<usize>) -> Self {
		Self { value, span }
	}
}
