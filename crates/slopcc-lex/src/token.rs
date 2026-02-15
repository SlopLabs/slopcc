use slopcc_common::span::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    #[must_use]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    PpNumber,
    CharConst,
    StringLiteral,
    Ident,
    HeaderName,
    Hash,
    HashHash,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Colon,
    Ellipsis,
    Dot,
    Arrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusPlus,
    MinusMinus,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Not,
    Amp,
    Pipe,
    Caret,
    Tilde,
    Shl,
    Shr,
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    AmpAssign,
    PipeAssign,
    CaretAssign,
    ShlAssign,
    ShrAssign,
    Question,
    Whitespace,
    Newline,
    Comment,
    Eof,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::{Token, TokenKind};
    use slopcc_common::source::FileId;
    use slopcc_common::span::Span;

    fn fid() -> FileId {
        FileId::new_for_tests(0)
    }

    #[test]
    fn token_construction() {
        let token = Token::new(TokenKind::Ident, Span::new(fid(), 2, 5));
        assert_eq!(token.kind, TokenKind::Ident);
        assert_eq!(token.span, Span::new(fid(), 2, 5));
    }

    #[test]
    fn token_is_copy() {
        let token = Token::new(TokenKind::PpNumber, Span::new(fid(), 0, 1));
        let copied = token;
        assert_eq!(token, copied);
    }
}
