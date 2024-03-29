use std::{ops::Range, rc::Rc};

use olympus_spanned::{CodeSource, OlympusError, Spanned};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub enum AsciiToken {
	OpenBrace,
	CloseBrace,
	OpenParen,
	CloseParen,
	OpenBracket,
	CloseBracket,
	Comma,
	SemiColon,
}

impl From<AsciiToken> for Token {
	fn from(val: AsciiToken) -> Self {
		Token::Ascii(val)
	}
}

#[derive(Debug, Clone)]
pub enum KeywordToken {
	Enum,
	Struct,
	Rpc,
	Proc,
	Import,
}

impl From<KeywordToken> for Token {
	fn from(val: KeywordToken) -> Self {
		Token::Keyword(val)
	}
}

#[derive(Debug, Clone)]
pub enum IntToken {
	Int8,
	Int16,
	Int32,
	Int64,
	UInt8,
	UInt16,
	UInt32,
	UInt64,
}

impl From<IntToken> for Token {
	fn from(val: IntToken) -> Self {
		Token::Type(TypeToken::Int(val))
	}
}

#[derive(Debug, Clone)]
pub enum TypeToken {
	Int(IntToken),
	VariableInt(IntToken),
	String,
	Array,
	Option,
}

impl From<TypeToken> for Token {
	fn from(val: TypeToken) -> Self {
		Token::Type(val)
	}
}

#[derive(Debug, Clone)]
pub enum Token {
	Comment(String),
	Ident(String),
	Arrow,

	Keyword(KeywordToken),
	Type(TypeToken),
	Ascii(AsciiToken),

	// The only usage of this is when manually tagging an enum and if you have this many enum
	// variants you need to seek help immediately.
	Number(i16),
}

pub type SpannedToken = Spanned<Token>;

#[derive(Debug, Clone, Copy)]
pub struct LexPoint {
	pub line: usize,
	pub segment_idx: usize,
	pub file_idx: usize,
}

pub struct Lexer {
	pub source: Rc<CodeSource>,
	graphemes: Vec<String>,
	curr_point: LexPoint,
	pub tokens: Vec<SpannedToken>,
}

impl Lexer {
	#[must_use]
	pub fn new(source: Rc<CodeSource>) -> Self {
		let graphemes = source.src.clone();
		let graphemes = graphemes.graphemes(true).map(str::to_string).collect();
		Self {
			source,
			graphemes,
			curr_point: LexPoint {
				line: 1,
				segment_idx: 0,
				file_idx: 0,
			},
			tokens: Vec::new(),
		}
	}

	fn move_point(&mut self, value: &str) {
		for ele in value.chars() {
			if ele == '\n' {
				self.curr_point.line += 1;
			}
		}

		self.curr_point.segment_idx += 1;
		self.curr_point.file_idx += value.len();
	}

	#[must_use]
	fn is_eof(&self) -> bool {
		self.curr_point.segment_idx >= self.graphemes.len()
	}

	#[must_use]
	fn peek(&self) -> Option<String> {
		self.graphemes.get(self.curr_point.segment_idx).cloned()
	}

	fn pop(&mut self) -> Option<String> {
		let popped = self.graphemes.get(self.curr_point.segment_idx).cloned()?;
		self.move_point(&popped);
		Some(popped)
	}

	fn pop_if(&mut self, predicate: impl Fn(&String) -> bool) -> Option<String> {
		let popped = self.graphemes.get(self.curr_point.segment_idx).cloned()?;
		if !predicate(&popped) {
			return None;
		}
		self.move_point(&popped);
		Some(popped)
	}

	fn pop_if_all(&mut self, predicate: impl Fn(char) -> bool) -> Option<String> {
		let popped = self.graphemes.get(self.curr_point.segment_idx).cloned()?;
		for ele in popped.chars() {
			if !predicate(ele) {
				return None;
			}
		}
		self.move_point(&popped);
		Some(popped)
	}

	#[must_use]
	fn get_span(&self, start: &LexPoint) -> Range<usize> {
		start.file_idx..self.curr_point.file_idx
	}

	fn add<T: Into<Token>>(&mut self, token: T, start: &LexPoint) {
		self.tokens.push(SpannedToken::new(token.into(), self.get_span(start)));
	}

	fn skip_whitespace(&mut self) {
		while self.pop_if(|v| v.chars().all(|c| c.is_ascii_whitespace())).is_some() {}
	}

	#[must_use]
	fn is_ident_chr_first(v: char) -> bool {
		v.is_ascii_alphabetic() || v == '_'
	}

	#[must_use]
	fn is_ident_chr_rest(v: char) -> bool {
		v.is_ascii_alphanumeric() || v == '_'
	}

	fn pop_ident(&mut self, start: Option<&str>) -> Option<String> {
		let mut ident = String::new();
		if let Some(start) = start {
			ident.push_str(start);
		} else {
			ident.push_str(&self.pop()?);
		}
		while let Some(v) = self.pop_if_all(Self::is_ident_chr_rest) {
			ident.push_str(&v);
		}

		Some(ident)
	}

	pub fn lex(&mut self) -> Result<(), OlympusError> {
		while !self.is_eof() {
			self.skip_whitespace();

			let start = self.curr_point;

			let Some(c) = self.pop() else {
				break;
			};

			match c.as_str() {
				"#" => {
					let mut comment = String::new();
					while let Some(v) = self.pop_if(|c| !c.ends_with('\n')) {
						comment.push_str(&v);
					}

					if comment.starts_with(' ') {
						comment.remove(0);
					}

					self.add(Token::Comment(comment), &start);
				}
				"{" => self.add(AsciiToken::OpenBrace, &start),
				"}" => self.add(AsciiToken::CloseBrace, &start),
				"(" => self.add(AsciiToken::OpenParen, &start),
				")" => self.add(AsciiToken::CloseParen, &start),
				"[" => self.add(AsciiToken::OpenBracket, &start),
				"]" => self.add(AsciiToken::CloseBracket, &start),
				";" => self.add(AsciiToken::SemiColon, &start),
				"," => self.add(AsciiToken::Comma, &start),
				"-" if self.pop_if(|v| v == ">").is_some() => self.add(Token::Arrow, &start),
				"@" if matches!(self.peek(), Some(v) if v.chars().all(Self::is_ident_chr_first)) => {
					let ident = self.pop_ident(None).ok_or(OlympusError::error(
						self.source.clone(),
						"Couldn't pop ident after finding it, this shouldn't ever happen.",
						self.get_span(&start),
					))?;

					match ident.as_str() {
						"int8" => self.add(IntToken::Int8, &start),
						"uint8" => self.add(IntToken::UInt8, &start),
						"int16" => self.add(IntToken::Int16, &start),
						"uint16" => self.add(IntToken::UInt16, &start),
						"int32" => self.add(IntToken::Int32, &start),
						"uint32" => self.add(IntToken::UInt32, &start),
						"int64" => self.add(IntToken::Int64, &start),
						"uint64" => self.add(IntToken::UInt64, &start),

						"varint8" => self.add(TypeToken::VariableInt(IntToken::Int8), &start),
						"varuint8" => self.add(TypeToken::VariableInt(IntToken::UInt8), &start),
						"varint16" => self.add(TypeToken::VariableInt(IntToken::Int16), &start),
						"varuint16" => self.add(TypeToken::VariableInt(IntToken::UInt16), &start),
						"varint32" => self.add(TypeToken::VariableInt(IntToken::Int32), &start),
						"varuint32" => self.add(TypeToken::VariableInt(IntToken::UInt32), &start),
						"varint64" => self.add(TypeToken::VariableInt(IntToken::Int64), &start),
						"varuint64" => self.add(TypeToken::VariableInt(IntToken::UInt64), &start),

						"string" => self.add(TypeToken::String, &start),
						"array" => self.add(TypeToken::Array, &start),
						"option" => self.add(TypeToken::Option, &start),

						_ => {
							return Err(OlympusError::error(
								self.source.clone(),
								"Unrecognized builtin",
								self.get_span(&start),
							))
						}
					}
				}
				c if c.chars().all(Self::is_ident_chr_first) => {
					let ident = self.pop_ident(Some(c)).ok_or(OlympusError::error(
						self.source.clone(),
						"Couldn't pop ident after finding it, this shouldn't ever happen.",
						self.get_span(&start),
					))?;

					match ident.as_str() {
						"struct" => self.add(KeywordToken::Struct, &start),
						"rpc" => self.add(KeywordToken::Rpc, &start),
						"proc" => self.add(KeywordToken::Proc, &start),
						"enum" => self.add(KeywordToken::Enum, &start),
						"import" => self.add(KeywordToken::Import, &start),

						ident => self.add(Token::Ident(ident.to_string()), &start),
					}
				}
				c if c.chars().all(char::is_numeric) => {
					let mut number = c.to_string();
					while let Some(v) = self.pop_if_all(char::is_numeric) {
						number.push_str(&v);
					}

					let source = self.source.clone().clone();
					let number = number.parse::<i16>().map_err(|_| {
						OlympusError::error(source, &format!("Max enum tag is {}", i16::MAX), self.get_span(&start))
					})?;
					self.add(Token::Number(number), &start);
				}
				_ => {
					return Err(OlympusError::error(
						self.source.clone().clone(),
						&format!("Unexpected character: {c}"),
						self.get_span(&start),
					))
				}
			}
		}
		Ok(())
	}
}
