pub mod node;
pub mod parsers;
pub mod walkers;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    // TokenizingError(TokenizingError),
    UnmatchedOpeningParen,
    UnmatchedClosingParen,
    UnmatchedOpeningBracket,
    UnmatchedClosingBracket,
    WrongBracket
}