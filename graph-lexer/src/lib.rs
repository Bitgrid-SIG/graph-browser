#![no_std]

#[derive(Debug)]
pub enum ErrorLexer {
    SourceEmpty,
}

/// The type emitted by the lexer holding the type of token and the 
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum TokenKind {
    /// Start-Of-Input (always the first token to be emitted)
    SOI,

    /// Character Set: \[a-zA-Z]*+
    ///
    /// Fields: (start index, end index)
    Word(::core::ops::Range<usize>),

    /// Character Set: \[0-9]*+
    ///
    /// Fields: (start index, end index)
    Number(::core::ops::Range<usize>),

    /// Character Set: \[\t \n\r\f]*+
    ///
    /// Fields: (start index, end index)
    Whitespace(::core::ops::Range<usize>),

    /// Character Set: \[^a-zA-Z0-9\t \n\r\f]
    ///
    /// Fields: (symbol)
    Symbol(char),

    /// End-Of-Input (always the final token to be emitted)
    EOI,
}

/// Data-less enum variant of [`TokenKind`] used to track the lexer's internal state.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TokenKindDiscriminant {
    /// See [`TokenKind::Word`].
    Word        = 1,

    /// See [`TokenKind::Number`].
    Number      = 2,

    /// See [`TokenKind::Whitespace`].
    WSpace      = 3,

    /// See [`TokenKind::Symbol`].
    Symbol      = 4,
}

#[derive(Debug)]
#[repr(u8)]
pub enum LexerState {
    None = 0,
    Ready = 1,
    EOI = 2,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    chars: ::core::iter::Peekable<::core::str::Chars<'a>>,
    cursor: usize,
    state: LexerState,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Result<Self, ErrorLexer> {
        if source.is_empty() {
            Err(ErrorLexer::SourceEmpty)

        } else {
            Ok(Self {
                chars: source.chars().peekable(),
                cursor: 0,
                state: LexerState::None,
            })
        }
    }

    #[inline(always)]
    fn token_type(copt: Option<char>) -> Option<TokenKindDiscriminant> {
        use TokenKindDiscriminant::*;

        match copt {
            Option::None => None,
            Option::Some(c) if c.is_alphabetic()    => Some(Word),
            Option::Some(c) if c.is_numeric()       => Some(Number),
            Option::Some(c) if c.is_whitespace()    => Some(WSpace),
            Option::Some(_)                         => Some(Symbol),
        }
    }

    fn until_boundary(&mut self) -> TokenKind {
        use TokenKindDiscriminant::*;

        let kind = Lexer::token_type(self.chars.peek().copied());
        
        // the inner loop is specifically designed for repeating sequences of chars of a particular
        // token-type. Symbols are a single char, so it would false-positive on sequences of them.
        match kind {
            Option::None => {
                self.state = LexerState::EOI;
                TokenKind::EOI
            }
            Option::Some(Word | Number | WSpace) => {
                let start = self.cursor;
                while Lexer::token_type(self.chars.peek().copied()) == kind {
                    let c = unsafe { self.chars.next().unwrap_unchecked() };
                    self.cursor += c.len_utf8();
                }

                let rn = start..self.cursor;
                match unsafe { kind.unwrap_unchecked() } {
                    Word    => TokenKind::Word(rn),
                    Number  => TokenKind::Number(rn),
                    WSpace  => TokenKind::Whitespace(rn),

                    _ => unreachable!()
                }
            }
            Option::Some(Symbol) => {
                let c = unsafe { self.chars.next().unwrap_unchecked() };
                self.cursor += c.len_utf8();
                TokenKind::Symbol(c)
            }
        }
    }

    #[inline]
    fn next_token(&mut self) -> Option<TokenKind> {
        match self.state {
            LexerState::Ready => Some(self.until_boundary()),
            LexerState::EOI => {
                self.state = LexerState::None;
                None
            },
            LexerState::None if (self.cursor == 0) => {
                self.state = LexerState::Ready;
                Some(TokenKind::SOI)
            },
            LexerState::None => Option::None
        }
    }
}

impl TokenKind {
    #[inline(always)]
    pub fn discriminant(&self) -> Option<TokenKindDiscriminant> {
        self.try_into().ok()
    }

    #[inline]
    pub fn range(&self) -> Option<&::core::ops::Range<usize>> {
        match self {
            TokenKind::SOI => None,
            TokenKind::Word(range) => Some(range),
            TokenKind::Number(range) => Some(range),
            TokenKind::Whitespace(range) => Some(range),
            TokenKind::Symbol(_) => None,
            TokenKind::EOI => None,
        }
    }

    #[inline]
    pub fn symbol(&self) -> Option<char> {
        match self {
            TokenKind::Symbol(c) => Some(*c),
            _ => None,
        }
    }
}

impl ::core::fmt::Display for ErrorLexer {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            ErrorLexer::SourceEmpty => write!(f, "Source string is empty"),
        }
    }
}

impl ::core::cmp::PartialEq for ErrorLexer {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl ::core::error::Error for ErrorLexer {}
impl ::core::cmp::Eq for ErrorLexer {}

impl ::core::cmp::PartialEq<TokenKind> for TokenKind {
    #[inline]
    fn eq(&self, other: &TokenKind) -> bool {
        match (self, other) {
            (TokenKind::SOI, TokenKind::SOI) => true,
            (TokenKind::Word(range1), TokenKind::Word(range2)) => range1 == range2,
            (TokenKind::Number(range1), TokenKind::Number(range2)) => range1 == range2,
            (TokenKind::Whitespace(range1), TokenKind::Whitespace(range2)) => range1 == range2,
            (TokenKind::Symbol(c1), TokenKind::Symbol(c2)) => c1 == c2,
            (TokenKind::EOI, TokenKind::EOI) => true,
            _ => false
        }
    }
}

impl ::core::cmp::PartialEq<TokenKindDiscriminant> for TokenKind {
    #[inline(always)]
    fn eq(&self, other: &TokenKindDiscriminant) -> bool {
        self.discriminant().is_some_and(|s| s == *other)
    }
}

impl ::core::cmp::Eq for TokenKind {}

impl TryFrom<&TokenKind> for TokenKindDiscriminant {
    type Error = ();
    
    #[inline]
    fn try_from(value: &TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Word(..)         => Ok(TokenKindDiscriminant::Word),
            TokenKind::Number(..)       => Ok(TokenKindDiscriminant::Number),
            TokenKind::Whitespace(..)   => Ok(TokenKindDiscriminant::WSpace),
            TokenKind::Symbol(..)       => Ok(TokenKindDiscriminant::Symbol),
            _ => Err(())
        }
    }
}

impl ::core::cmp::PartialEq<TokenKindDiscriminant> for TokenKindDiscriminant {
    #[inline(always)]
    fn eq(&self, other: &TokenKindDiscriminant) -> bool {
        (*self as u8) == (*other as u8)
    }
}

impl ::core::cmp::PartialEq<TokenKind> for TokenKindDiscriminant {
    #[inline(always)]
    fn eq(&self, other: &TokenKind) -> bool {
        other.discriminant().is_some_and(|s| *self == s)
    }
}

impl ::core::cmp::Eq for TokenKindDiscriminant {}

impl ::core::iter::Iterator for Lexer<'_> {
    type Item = TokenKind;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn new_empty_source_err() {
        assert_eq!(Lexer::new("").unwrap_err(), ErrorLexer::SourceEmpty);
    }

    #[test]
    fn single_char_word() {
        let mut lex = Lexer::new("A").unwrap();
        assert_eq!(lex.next(), Some(TokenKind::SOI));
        assert_eq!(lex.next(), Some(TokenKind::Word(0..1)));
        assert_eq!(lex.next(), Some(TokenKind::EOI));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn mixed_sequence() {
        extern crate std;
        use std::{vec::Vec, vec};

        let s = "abc 123!"; // len = 8
        let lex = Lexer::new(s).unwrap();
        let toks: Vec<TokenKind> = lex.collect();
        assert_eq!(
            toks,
            vec![
                TokenKind::SOI,
                TokenKind::Word(0..3),
                TokenKind::Whitespace(3..4),
                TokenKind::Number(4..7),
                TokenKind::Symbol('!'),
                TokenKind::EOI,
            ]
        );
    }

    #[test]
    fn discriminant_matches() {
        let tk = TokenKind::Number(5..6);
        assert_eq!(tk.discriminant(), Some(TokenKindDiscriminant::Number));
        assert!(tk == TokenKindDiscriminant::Number);
    }

    #[test]
    fn display_error() {
        extern crate std;
        use std::format;

        let e = ErrorLexer::SourceEmpty;
        assert_eq!(format!("{}", e), "Source string is empty");
    }
}
