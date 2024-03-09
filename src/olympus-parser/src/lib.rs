use std::ops::Range;

use olympus_lexer::lexer::{AsciiToken, KeywordToken, SpannedErr, SpannedToken, Token, TypeToken};

#[derive(Debug)]
pub struct ParsedEnumVariant {
    pub ident: String,
    pub value: i16,
}

#[derive(Debug)]
pub struct ParsedEnum {
    pub ident: String,
    pub variants: Vec<ParsedEnumVariant>,
}

#[derive(Debug)]
pub enum ParsedDataKind {
    Builtin(TypeToken),
    External(String),
}

#[derive(Debug)]
pub struct ParsedDataField {
    pub ident: String,
    pub kind: ParsedDataKind,
}

#[derive(Debug)]
pub struct ParsedData {
    pub ident: String,
    pub fields: Vec<ParsedDataField>,
}

#[derive(Debug)]
pub struct ParsedProcedureParam {
    pub ident: String,
    pub kind: ParsedDataKind,
}

#[derive(Debug)]
pub struct ParsedProcedure {
    pub ident: String,
    pub params: Vec<ParsedProcedureParam>,
    pub return_kind: ParsedDataKind,
}

#[derive(Debug)]
pub struct ParsedServer {
    pub procedures: Vec<ParsedProcedure>,
}

pub struct Parser {
    tokens: Vec<SpannedToken>,
    token_idx: usize,
    pub enums: Vec<ParsedEnum>,
    pub datas: Vec<ParsedData>,
    pub servers: Vec<ParsedServer>,
}

impl Parser {
    #[must_use]
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            token_idx: 0,
            enums: Vec::new(),
            datas: Vec::new(),
            servers: Vec::new(),
        }
    }

    #[must_use]
    pub fn peek(&self) -> Option<SpannedToken> {
        self.tokens.get(self.token_idx).cloned()
    }

    pub fn pop(&mut self) -> Option<SpannedToken> {
        let next = self.tokens.get(self.token_idx).cloned();
        self.token_idx += 1;
        next
    }

    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    pub fn get_span(&self, offset: isize) -> Range<usize> {
        self.tokens
            .get((self.token_idx as isize - 1 + offset) as usize)
            .cloned()
            .map_or(0..0, |token| token.span)
    }

    pub fn pop_must_match(
        &mut self,
        predicate: impl Fn(Token) -> bool,
        error: String,
    ) -> Result<SpannedToken, SpannedErr> {
        let next = self.peek().ok_or(SpannedErr::new(
            "Expected token after".into(),
            self.get_span(-1),
        ))?;

        if !predicate(next.value.clone()) {
            return Err(SpannedErr::new(error, self.get_span(0)));
        }

        self.token_idx += 1;
        Ok(next)
    }

    pub fn parse(&mut self) -> Result<(), SpannedErr> {
        while self.token_idx < self.tokens.len() {
            let Some(token) = self.pop() else {
                break;
            };

            match &token.value {
                Token::Comment(_) => {}
                Token::Keyword(keyword) => match keyword {
                    KeywordToken::Enum => self.parse_enum()?,
                    KeywordToken::Data => self.parse_data()?,
                    KeywordToken::Server => self.parse_server()?,
                    KeywordToken::Proc => {
                        return Err(SpannedErr::new(
                            "This is a bug. Proc shouldn't be parsed here.".to_string(),
                            self.get_span(0),
                        ))
                    }
                },
                token => {
                    return Err(SpannedErr::new(
                        format!("Unexpected token: {token:?}"),
                        self.get_span(0),
                    ))
                }
            }
        }

        Ok(())
    }

    fn enum_gather_variants(&mut self) -> Result<Vec<(String, i16)>, SpannedErr> {
        let mut res = Vec::new();

        while let Some(token) = self.pop() {
            match token.value {
                Token::Ident(ident) => {
                    self.pop_must_match(
                        |t| matches!(t, Token::Arrow),
                        "Expected '->' after Enum Ident".into(),
                    )?;
                    let number = next_must_match!(self, "Expected enum tag".into(), Number);
                    self.pop_must_match(
                        |t| matches!(t, Token::Ascii(AsciiToken::SemiColon)),
                        "Expected ';' after Enum Value".into(),
                    )?;

                    res.push((ident, number));
                }
                Token::Ascii(AsciiToken::CloseBrace) => {
                    break;
                }
                token => {
                    return Err(SpannedErr::new(
                        format!("Expected '}}' or Ident. Got: {token:?}"),
                        self.get_span(0),
                    ))
                }
            }
        }

        Ok(res)
    }

    pub fn parse_enum(&mut self) -> Result<(), SpannedErr> {
        let ident = next_must_match!(self, "Expected Ident for enum".into(), Ident);

        self.pop_must_match(
            |t| matches!(t, Token::Ascii(AsciiToken::OpenBrace)),
            "Expected '{' after Enum Ident".into(),
        )?;

        let variants = self
            .enum_gather_variants()?
            .into_iter()
            .map(|v| ParsedEnumVariant {
                ident: v.0,
                value: v.1,
            })
            .collect();

        self.enums.push(ParsedEnum { ident, variants });

        Ok(())
    }

    fn data_gather_fields(&mut self) -> Result<Vec<ParsedDataField>, SpannedErr> {
        let mut res = Vec::new();

        while let Some(token) = self.pop() {
            match token.value {
                Token::Ident(ident) => {
                    self.pop_must_match(
                        |t| matches!(t, Token::Arrow),
                        "Expected '->' after ident".into(),
                    )?;

                    let kind = self
                        .pop()
                        .ok_or(SpannedErr::new("Expected type".into(), self.get_span(0)))?;
                    let kind = match kind.value {
                        Token::Ident(ident) => ParsedDataKind::External(ident),
                        Token::Type(ty) => ParsedDataKind::Builtin(ty),
                        _ => return Err(SpannedErr::new("Expected type".into(), self.get_span(0))),
                    };

                    self.pop_must_match(
                        |t| matches!(t, Token::Ascii(AsciiToken::SemiColon)),
                        "Expected ';' after type".into(),
                    )?;

                    res.push(ParsedDataField { ident, kind });
                }
                Token::Ascii(AsciiToken::CloseBrace) => {
                    break;
                }
                token => {
                    return Err(SpannedErr::new(
                        format!("Expected '}}' or ident. Got: {token:?}"),
                        self.get_span(0),
                    ))
                }
            }
        }

        Ok(res)
    }

    pub fn parse_data(&mut self) -> Result<(), SpannedErr> {
        let ident = next_must_match!(self, "Expected Ident for data".into(), Ident);

        self.pop_must_match(
            |t| matches!(t, Token::Ascii(AsciiToken::OpenBrace)),
            "Expected '{' after ident".into(),
        )?;

        let fields = self.data_gather_fields()?;
        self.datas.push(ParsedData { ident, fields });

        Ok(())
    }

    fn server_gather_procudures(&mut self) -> Result<Vec<ParsedProcedure>, SpannedErr> {
        let mut res = Vec::new();

        while let Some(token) = self.pop() {
            match token.value {
                Token::Keyword(KeywordToken::Proc) => {
                    let ident = next_must_match!(self, "Expected ident".into(), Ident);
                    self.pop_must_match(
                        |t| matches!(t, Token::Ascii(AsciiToken::OpenParen)),
                        "Expected '(' after ident".into(),
                    )?;

                    let mut params: Vec<ParsedProcedureParam> = Vec::new();
                    while let Some(token) = self.pop() {
                        match token.value {
                            Token::Ident(ident) => {
                                self.pop_must_match(
                                    |t| matches!(t, Token::Arrow),
                                    "Expected '->' after ident".into(),
                                )?;

                                let kind = self.pop().ok_or(SpannedErr::new(
                                    "Expected type".into(),
                                    self.get_span(0),
                                ))?;
                                let kind = match kind.value {
                                    Token::Ident(ident) => ParsedDataKind::External(ident),
                                    Token::Type(ty) => ParsedDataKind::Builtin(ty),
                                    _ => {
                                        return Err(SpannedErr::new(
                                            "Expected type".into(),
                                            self.get_span(0),
                                        ))
                                    }
                                };

                                params.push(ParsedProcedureParam { ident, kind });
                            }
                            Token::Ascii(AsciiToken::CloseParen) => break,
                            Token::Ascii(AsciiToken::Comma) => continue,
                            token => {
                                return Err(SpannedErr::new(
                                    format!("Expected ident or ')'. Got: {token:?}"),
                                    self.get_span(0),
                                ))
                            }
                        }
                    }

                    self.pop_must_match(
                        |t| matches!(t, Token::Arrow),
                        "Expected '->' after params".into(),
                    )?;

                    let return_kind = self
                        .pop()
                        .ok_or(SpannedErr::new("Expected type".into(), self.get_span(0)))?;
                    let return_kind = match return_kind.value {
                        Token::Ident(ident) => ParsedDataKind::External(ident),
                        Token::Type(ty) => ParsedDataKind::Builtin(ty),
                        _ => return Err(SpannedErr::new("Expected type".into(), self.get_span(0))),
                    };

                    self.pop_must_match(
                        |t| matches!(t, Token::Ascii(AsciiToken::SemiColon)),
                        "Expected ';' after return type".into(),
                    )?;

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
                    return Err(SpannedErr::new(
                        format!("Expected '}}' or proc. Got: {token:?}"),
                        self.get_span(0),
                    ))
                }
            }
        }

        Ok(res)
    }

    pub fn parse_server(&mut self) -> Result<(), SpannedErr> {
        self.pop_must_match(
            |t| matches!(t, Token::Ascii(AsciiToken::OpenBrace)),
            "Expected '{' after server".into(),
        )?;

        let procedures = self.server_gather_procudures()?;
        self.servers.push(ParsedServer { procedures });
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
                    v
                }
                _ => return Err(SpannedErr::new($expected, $self.get_span(0))),
            },
            _ => return Err(SpannedErr::new($expected, $self.get_span(0))),
        }
    }};
}
