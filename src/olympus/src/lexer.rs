use std::ops::Range;

use crate::cursor::{CodeCursor, CodeCursorPoint};

#[derive(Debug)]
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

#[derive(Debug)]
pub enum KeywordToken {
    Proc,
    Data,
    Server,
    Enum,
}

impl From<KeywordToken> for Token {
    fn from(val: KeywordToken) -> Self {
        Token::Keyword(val)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum TypeToken {
    Int(IntToken),
    VariableInt(IntToken),
    String,
    Generator,
}

impl From<TypeToken> for Token {
    fn from(val: TypeToken) -> Self {
        Token::Type(val)
    }
}

#[derive(Debug)]
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

pub struct Spanned<T, S = Range<usize>> {
    pub value: T,
    pub span: S,
}

pub type SpannedErr = Spanned<String>;

impl SpannedErr {
    pub fn new(span: Range<usize>, err: String) -> Self {
        Self { value: err, span }
    }
}

pub struct Lexer {
    cursor: CodeCursor,
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self {
            cursor: CodeCursor::new(src),
            tokens: Vec::new(),
        }
    }

    fn ident_char(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    pub fn lex(&mut self) -> Result<(), SpannedErr> {
        while !self.cursor.is_eof() {
            self.cursor.skip_whitespace();

            let Some(point) = self.cursor.pop() else {
                break;
            };

            match point.value {
                '#' => {
                    let mut comment = String::new();
                    for c in self.cursor.pop_while(|c| c != '\n') {
                        comment.push(c.value);
                    }

                    if comment.starts_with(' ') {
                        comment.remove(0);
                    }

                    self.add(Token::Comment(comment))
                }
                '{' => self.add(AsciiToken::OpenBrace),
                '}' => self.add(AsciiToken::CloseBrace),
                '(' => self.add(AsciiToken::OpenParen),
                ')' => self.add(AsciiToken::CloseParen),
                '[' => self.add(AsciiToken::OpenBracket),
                ']' => self.add(AsciiToken::CloseBracket),
                '-' if self.cursor.pop_if('>').is_some() => self.add(Token::Arrow),
                ';' => self.add(AsciiToken::SemiColon),
                '@' if matches!(self.cursor.peek_chr(), Some(v) if Self::ident_char(v)) => {
                    let points = self
                        .cursor
                        .pop_while(Self::ident_char)
                        .into_iter()
                        .collect::<Vec<CodeCursorPoint>>();
                    let points_span = points[0].file_idx..points[points.len() - 1].file_idx + 1;
                    let ident = points.iter().map(|p| p.value).collect::<String>();

                    match ident.as_str() {
                        "int8" => self.add(IntToken::Int8),
                        "uint8" => self.add(IntToken::UInt8),
                        "int16" => self.add(IntToken::Int16),
                        "uint16" => self.add(IntToken::UInt16),
                        "int32" => self.add(IntToken::Int32),
                        "uint32" => self.add(IntToken::UInt32),
                        "int64" => self.add(IntToken::Int64),
                        "uint64" => self.add(IntToken::UInt64),

                        "varint8" => self.add(TypeToken::VariableInt(IntToken::Int8)),
                        "varuint8" => self.add(TypeToken::VariableInt(IntToken::UInt8)),
                        "varint16" => self.add(TypeToken::VariableInt(IntToken::Int16)),
                        "varuint16" => self.add(TypeToken::VariableInt(IntToken::UInt16)),
                        "varint32" => self.add(TypeToken::VariableInt(IntToken::Int32)),
                        "varuint32" => self.add(TypeToken::VariableInt(IntToken::UInt32)),
                        "varint64" => self.add(TypeToken::VariableInt(IntToken::Int64)),
                        "varuint64" => self.add(TypeToken::VariableInt(IntToken::UInt64)),

                        "string" => self.add(TypeToken::String),
                        "Generator" => self.add(TypeToken::Generator),

                        _ => {
                            return Err(SpannedErr::new(
                                points_span,
                                "Unrecognized builtin".to_string(),
                            ))
                        }
                    }
                }
                '@' => {
                    return Err(SpannedErr::new(
                        point.file_idx..point.file_idx,
                        "Expected builtin after '@'".into(),
                    ));
                }
                c if Self::ident_char(c) => {
                    let ident = std::iter::once(c)
                        .chain(
                            self.cursor
                                .pop_while(Self::ident_char)
                                .into_iter()
                                .map(|p| p.value),
                        )
                        .collect::<String>();

                    match ident.as_str() {
                        "data" => self.add(KeywordToken::Data),
                        "server" => self.add(KeywordToken::Server),
                        "proc" => self.add(KeywordToken::Proc),
                        "enum" => self.add(KeywordToken::Enum),

                        ident => self.add(Token::Ident(ident.to_string())),
                    }
                }
                c if c.is_ascii_digit() => {
                    let points = std::iter::once(point)
                        .chain(self.cursor.pop_while(|c| c.is_ascii_digit()).into_iter())
                        .collect::<Vec<CodeCursorPoint>>();
                    let points_span = points[0].file_idx..points[points.len() - 1].file_idx + 1;
                    let number = points.iter().map(|p| p.value).collect::<String>();
                    let num = number.parse().map_err(|_| {
                        SpannedErr::new(points_span, format!("Max enum tag is {}", i16::MAX))
                    })?;

                    self.add(Token::Number(num));
                }
                c => {
                    return Err(SpannedErr::new(
                        point.file_idx..point.file_idx,
                        format!("Unexpected character: {c}"),
                    ))
                }
            }
        }

        Ok(())
    }

    fn add<T: Into<Token>>(&mut self, token: T) {
        self.tokens.push(token.into());
    }
}
