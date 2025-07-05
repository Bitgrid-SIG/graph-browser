extern crate graph_lexer as lexer; ***REMOVED***
use lexer::{Lexer, TokenKind, ErrorLexer};

#[test]
fn integration_full_run() {
    let input = "X9_\tZ";
    let expected = [
        TokenKind::SOI,
        TokenKind::Word(0..1),          // X
        TokenKind::Number(1..2),        // 9
        TokenKind::Symbol('_'),         // _
        TokenKind::Whitespace(3..4),    // \t
        TokenKind::Word(4..5),          // Z
        TokenKind::EOI,
    ];
    let iter = Lexer::new(input).unwrap();
    let out: &[TokenKind] = &iter.collect::<Vec<_>>();
    assert_eq!(out, &expected);
}

#[test]
fn integration_error_on_empty() {
    assert_eq!(Lexer::new("").unwrap_err(), ErrorLexer::SourceEmpty);
}