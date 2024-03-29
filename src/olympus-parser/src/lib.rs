use std::ops::Range;

use olympus_lexer::{AsciiToken, IntToken, KeywordToken, SpannedToken, Token, TypeToken};
use olympus_spanned::{OlympusError, Spanned};

#[derive(Debug)]
pub struct ParsedEnumVariant {
	pub ident: Spanned<String>,
	pub value: i16,
}

#[derive(Debug)]
pub struct ParsedEnum {
	pub ident: Spanned<String>,
	pub variants: Vec<ParsedEnumVariant>,
}

#[derive(Debug)]
pub enum ParsedBultin {
	Nothing,
	Int(IntToken),
	VariableInt(IntToken),
	String,
	Array(Box<Spanned<ParsedTypeKind>>),
	Option(Box<Spanned<ParsedTypeKind>>),
}

#[derive(Debug)]
pub enum ParsedTypeKind {
	Builtin(ParsedBultin),
	External(String),
}

#[derive(Debug)]
pub struct ParsedStructField {
	pub ident: Spanned<String>,
	pub kind: Spanned<ParsedTypeKind>,
}

#[derive(Debug)]
pub struct ParsedStruct {
	pub ident: Spanned<String>,
	pub fields: Vec<ParsedStructField>,
}

#[derive(Debug)]
pub struct ParsedProcedureParam {
	pub ident: Spanned<String>,
	pub kind: Spanned<ParsedTypeKind>,
}

#[derive(Debug)]
pub struct ParsedProcedure {
	pub ident: Spanned<String>,
	pub params: Vec<ParsedProcedureParam>,
	pub return_kind: Spanned<ParsedTypeKind>,
}

#[derive(Debug)]
pub struct ParsedRpcContainer {
	pub procedures: Vec<ParsedProcedure>,
}

pub struct Parser {
	tokens: Vec<SpannedToken>,
	token_idx: usize,
	pub imports: Vec<Spanned<String>>,
	pub enums: Vec<ParsedEnum>,
	pub structs: Vec<ParsedStruct>,
	pub rpc_container: ParsedRpcContainer,
}

impl Parser {
	#[must_use]
	pub fn new(tokens: Vec<SpannedToken>) -> Self {
		Self {
			tokens,
			token_idx: 0,
			imports: Vec::new(),
			enums: Vec::new(),
			structs: Vec::new(),
			rpc_container: ParsedRpcContainer { procedures: vec![] },
		}
	}

	#[must_use]
	fn peek(&self) -> Option<SpannedToken> {
		self.tokens.get(self.token_idx).cloned()
	}

	fn pop(&mut self) -> Option<SpannedToken> {
		let next = self.tokens.get(self.token_idx).cloned();
		self.token_idx += 1;
		next
	}

	#[must_use]
	#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
	fn get_span(&self, offset: isize) -> Range<usize> {
		self.tokens
			.get((self.token_idx as isize - 1 + offset) as usize)
			.cloned()
			.map_or(0..0, |token| token.span)
	}

	fn pop_must_match(&mut self, predicate: impl Fn(Token) -> bool, error: &str) -> Result<SpannedToken, OlympusError> {
		let next = self
			.peek()
			.ok_or(OlympusError::error("Expected token after", self.get_span(-1)))?;

		if !predicate(next.value.clone()) {
			return Err(OlympusError::error(error, self.get_span(0)));
		}

		self.token_idx += 1;
		Ok(next)
	}

	pub fn parse(&mut self) -> Result<(), OlympusError> {
		while self.token_idx < self.tokens.len() {
			let Some(token) = self.pop() else {
				break;
			};

			match &token.value {
				Token::Comment(_) => {}
				Token::Keyword(keyword) => match keyword {
					KeywordToken::Enum => self.parse_enum()?,
					KeywordToken::Struct => self.parse_data()?,
					KeywordToken::Rpc => self.parse_server()?,
					KeywordToken::Proc => {
						return Err(OlympusError::error(
							"This is a bug. Proc shouldn't be parsed here.",
							self.get_span(0),
						))
					}
					KeywordToken::Import => {
						let ident = next_must_match!(self, "Expected Ident for import", Ident);
						self.pop_must_match(
							|t| matches!(t, Token::Ascii(AsciiToken::SemiColon)),
							"Expected ';' after import",
						)?;
						self.imports.push(ident);
					}
				},
				token => {
					return Err(OlympusError::error(
						&format!("Unexpected token: {token:?}"),
						self.get_span(0),
					))
				}
			}
		}

		Ok(())
	}

	fn enum_gather_variants(&mut self) -> Result<Vec<ParsedEnumVariant>, OlympusError> {
		let mut res = Vec::new();

		while let Some(token) = self.pop() {
			match token.value {
				Token::Ident(ident) => {
					self.pop_must_match(|t| matches!(t, Token::Arrow), "Expected '->' after Enum Ident")?;
					let Spanned { value, .. } = next_must_match!(self, "Expected enum tag", Number);
					self.pop_must_match(
						|t| matches!(t, Token::Ascii(AsciiToken::SemiColon)),
						"Expected ';' after Enum Value",
					)?;

					res.push(ParsedEnumVariant {
						ident: Spanned::new(ident, token.span),
						value,
					});
				}
				Token::Ascii(AsciiToken::CloseBrace) => {
					break;
				}
				token => {
					return Err(OlympusError::error(
						&format!("Expected '}}' or Ident. Got: {token:?}"),
						self.get_span(0),
					))
				}
			}
		}

		Ok(res)
	}

	fn parse_enum(&mut self) -> Result<(), OlympusError> {
		let ident = next_must_match!(self, "Expected Ident for enum", Ident);

		self.pop_must_match(
			|t| matches!(t, Token::Ascii(AsciiToken::OpenBrace)),
			"Expected '{' after Enum Ident",
		)?;

		let variants = self.enum_gather_variants()?;

		self.enums.push(ParsedEnum { ident, variants });

		Ok(())
	}

	fn parse_generic_type(&mut self) -> Result<Spanned<ParsedTypeKind>, OlympusError> {
		let this_span = self.get_span(0);

		self.pop_must_match(
			|t| matches!(t, Token::Ascii(AsciiToken::OpenBracket)),
			"Expected generic type",
		)?;

		let array_type = self
			.pop()
			.ok_or(OlympusError::error("Expected generic type", self.get_span(-1)))?;

		let value = match array_type.value {
			Token::Ident(ident) => Ok(Spanned::new(ParsedTypeKind::External(ident), array_type.span)),
			Token::Type(ty) => match ty {
				TypeToken::Int(v) => Ok(Spanned::new(
					ParsedTypeKind::Builtin(ParsedBultin::Int(v)),
					array_type.span,
				)),
				TypeToken::VariableInt(v) => Ok(Spanned::new(
					ParsedTypeKind::Builtin(ParsedBultin::VariableInt(v)),
					array_type.span,
				)),
				TypeToken::String => Ok(Spanned::new(
					ParsedTypeKind::Builtin(ParsedBultin::String),
					array_type.span,
				)),
				TypeToken::Array => Ok(Spanned::new(
					ParsedTypeKind::Builtin(ParsedBultin::Array(Box::new(self.parse_generic_type()?))),
					this_span,
				)),
				TypeToken::Option => Ok(Spanned::new(
					ParsedTypeKind::Builtin(ParsedBultin::Option(Box::new(self.parse_generic_type()?))),
					this_span,
				)),
			},
			_ => Err(OlympusError::error("Expected type", self.get_span(0))),
		}?;

		self.pop_must_match(
			|t| matches!(t, Token::Ascii(AsciiToken::CloseBracket)),
			"Expected ']' after generic type",
		)?;

		Ok(value)
	}

	fn parse_type(&mut self, kind_token: Spanned<Token>) -> Result<Spanned<ParsedTypeKind>, OlympusError> {
		let array_type = match kind_token.value {
			Token::Ident(ident) => return Ok(Spanned::new(ParsedTypeKind::External(ident), kind_token.span)),
			Token::Type(ty) => match ty {
				TypeToken::Int(v) => {
					return Ok(Spanned::new(
						ParsedTypeKind::Builtin(ParsedBultin::Int(v)),
						kind_token.span,
					))
				}
				TypeToken::VariableInt(v) => {
					return Ok(Spanned::new(
						ParsedTypeKind::Builtin(ParsedBultin::VariableInt(v)),
						kind_token.span,
					))
				}
				TypeToken::String => {
					return Ok(Spanned::new(
						ParsedTypeKind::Builtin(ParsedBultin::String),
						kind_token.span,
					))
				}
				TypeToken::Array => Spanned::new(
					ParsedTypeKind::Builtin(ParsedBultin::Array(Box::new(self.parse_generic_type()?))),
					kind_token.span,
				),
				TypeToken::Option => Spanned::new(
					ParsedTypeKind::Builtin(ParsedBultin::Option(Box::new(self.parse_generic_type()?))),
					kind_token.span,
				),
			},
			_ => return Err(OlympusError::error("Expected type", self.get_span(0))),
		};

		Ok(array_type)
	}

	fn data_gather_fields(&mut self) -> Result<Vec<ParsedStructField>, OlympusError> {
		let mut res = Vec::new();

		while let Some(token) = self.pop() {
			match token.value {
				Token::Ident(ident) => {
					self.pop_must_match(|t| matches!(t, Token::Arrow), "Expected '->' after ident")?;

					let kind = self
						.pop()
						.ok_or(OlympusError::error("Expected type", self.get_span(0)))?;
					let kind = self.parse_type(kind)?;

					self.pop_must_match(
						|t| matches!(t, Token::Ascii(AsciiToken::SemiColon)),
						"Expected ';' after type",
					)?;

					res.push(ParsedStructField {
						ident: Spanned::new(ident, token.span),
						kind,
					});
				}
				Token::Ascii(AsciiToken::CloseBrace) => {
					break;
				}
				token => {
					return Err(OlympusError::error(
						&format!("Expected '}}' or ident. Got: {token:?}"),
						self.get_span(0),
					))
				}
			}
		}

		Ok(res)
	}

	fn parse_data(&mut self) -> Result<(), OlympusError> {
		let ident = next_must_match!(self, "Expected Ident for data", Ident);

		self.pop_must_match(
			|t| matches!(t, Token::Ascii(AsciiToken::OpenBrace)),
			"Expected '{' after ident",
		)?;

		let fields = self.data_gather_fields()?;
		self.structs.push(ParsedStruct { ident, fields });

		Ok(())
	}

	fn server_gather_procudures(&mut self) -> Result<Vec<ParsedProcedure>, OlympusError> {
		let mut res = Vec::new();

		while let Some(token) = self.pop() {
			match token.value {
				Token::Keyword(KeywordToken::Proc) => {
					let ident = next_must_match!(self, "Expected ident", Ident);
					self.pop_must_match(
						|t| matches!(t, Token::Ascii(AsciiToken::OpenParen)),
						"Expected '(' after ident",
					)?;

					let mut params: Vec<ParsedProcedureParam> = Vec::new();
					while let Some(token) = self.pop() {
						match token.value {
							Token::Ident(ident) => {
								self.pop_must_match(|t| matches!(t, Token::Arrow), "Expected '->' after ident")?;

								let kind = self
									.pop()
									.ok_or(OlympusError::error("Expected type", self.get_span(0)))?;
								let kind = self.parse_type(kind)?;

								params.push(ParsedProcedureParam {
									ident: Spanned::new(ident, token.span),
									kind,
								});
							}
							Token::Ascii(AsciiToken::CloseParen) => break,
							Token::Ascii(AsciiToken::Comma) => continue,
							token => {
								return Err(OlympusError::error(
									&format!("Expected ident or ')'. Got: {token:?}"),
									self.get_span(0),
								))
							}
						}
					}

					let return_kind = if let Some(Spanned {
						value: Token::Ascii(AsciiToken::SemiColon),
						..
					}) = self.peek()
					{
						self.pop();
						Spanned::new(ParsedTypeKind::Builtin(ParsedBultin::Nothing), self.get_span(-1))
					} else {
						self.pop_must_match(|t| matches!(t, Token::Arrow), "Expected '->' after params")?;

						let return_kind = self
							.pop()
							.ok_or(OlympusError::error("Expected type", self.get_span(0)))?;
						let return_kind = self.parse_type(return_kind)?;

						self.pop_must_match(
							|t| matches!(t, Token::Ascii(AsciiToken::SemiColon)),
							"Expected ';' after return type",
						)?;

						return_kind
					};

					res.push(ParsedProcedure {
						ident,
						params,
						return_kind,
					});
				}
				Token::Ascii(AsciiToken::CloseBrace) => {
					break;
				}
				token => {
					return Err(OlympusError::error(
						&format!("Expected '}}' or proc. Got: {token:?}"),
						self.get_span(0),
					))
				}
			}
		}

		Ok(res)
	}

	fn parse_server(&mut self) -> Result<(), OlympusError> {
		self.pop_must_match(
			|t| matches!(t, Token::Ascii(AsciiToken::OpenBrace)),
			"Expected '{' after server",
		)?;

		let procedures = self.server_gather_procudures()?;
		self.rpc_container.procedures.extend(procedures);
		Ok(())
	}
}

#[macro_export]
macro_rules! next_must_match {
	($self:expr, $expected:expr, $match:ident) => {{
		let peeked = $self.peek();
		match peeked {
			Some(spanned) => match spanned.value {
				Token::$match(v) => {
					let _ = $self.pop();
					let span = $self.get_span(0);
					Spanned::new(v, span)
				}
				_ => return Err(OlympusError::error($expected, $self.get_span(0))),
			},
			_ => return Err(OlympusError::error($expected, $self.get_span(0))),
		}
	}};
}
