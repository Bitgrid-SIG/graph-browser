extern crate graph_lexer as lexer;
use lexer::{Lexer, TokenKind, ErrorLexer};

#[test]
fn integration_full_run() {
    let input = "X9_\tZ";
    const EXPECTED: [TokenKind; 7] = [
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
    assert_eq!(out, &EXPECTED);
}

#[test]
fn integration_error_on_empty() {
    assert_eq!(Lexer::new("").unwrap_err(), ErrorLexer::SourceEmpty);
}

#[test]
fn integration_for_loop() {
    let input = "bAbYsEaL536@%+\t \n\n";
    const EXPECTED: [(TokenKind, Option<&'static str>); 8] = [
        (TokenKind::SOI, None),
        (TokenKind::Word(0..8), Some("bAbYsEaL")),
        (TokenKind::Number(8..11), Some("536")),
        (TokenKind::Symbol('@'), Some("@")),
        (TokenKind::Symbol('%'), Some("%")),
        (TokenKind::Symbol('+'), Some("+")),
        (TokenKind::Whitespace(14..18), Some("\t \n\n")),
        (TokenKind::EOI, None)
    ];

    for (i, token) in Lexer::new(input).unwrap().into_iter().enumerate() {
        let expect = &EXPECTED[i];
        assert_eq!(token, expect.0, "{token:?} does not equal {:?}", expect.0);

        if let Some(range) = token.range() {
            let substr = &input[range.clone()];
            assert_eq!(substr, expect.1.unwrap(), "'{substr}' does not equal '{}'", expect.1.unwrap());

        } else if let Some(symbol) = token.symbol() {
            let new_str = symbol.to_string();
            assert_eq!(new_str, expect.1.unwrap(), "'{new_str}' does not equal '{}'", expect.1.unwrap());
        }
    }
}
