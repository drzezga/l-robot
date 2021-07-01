pub mod tokenizer;
pub mod parser;
pub mod latex;
pub mod resolver;

use resolver::resolve_message::ResolveMessage;

enum PipelineError {
    TokenizeError(tokenizer::TokenizeError),
    ParsingError(parser::ParseError),
}

impl PipelineError {
    fn to_resolve_message(&self) -> ResolveMessage {
        match self {
            PipelineError::TokenizeError(err) => match err {
                tokenizer::TokenizeError::ParseFloatError(err) => ResolveMessage::error(&err.to_string()),
            },
            PipelineError::ParsingError(err) => match err {
                parser::ParseError::UnmatchedOpeningParen => ResolveMessage::error("Unmatched opening paren"),
                parser::ParseError::UnmatchedClosingParen => ResolveMessage::error("Unmatched closing paren"),
                parser::ParseError::UnmatchedOpeningBracket => ResolveMessage::error("Unmatched opening bracket"),
                parser::ParseError::UnmatchedClosingBracket => ResolveMessage::error("Unmatched closing bracket"),
                parser::ParseError::WrongBracket => ResolveMessage::error("Wrong bracket"),
            },
        }
    }
}

pub fn resolve_lines(lines: Vec<String>) -> Vec<(usize, ResolveMessage)> {
    let mut resolver = resolver::Resolver::new();

    lines.iter()
        .map(|line| {
            match tokenizer::tokenize(line) {
                Ok(x) => Ok(x),
                Err(err) => Err(PipelineError::TokenizeError(err)),
            }
        })
        // .filter(|(_, tokens)| tokens.len() != 0)
        .map(|tokens| {
            match tokens {
                Ok(tokens) => {
                    match parser::parsers::parse(&tokens) {
                        Ok(x) => Ok(x),
                        Err(err) => Err(PipelineError::ParsingError(err)),
                    }
                },
                Err(x) => Err(x),
            }
        })
        .enumerate()
        .map(|(line_num, output)| {
            match output {
                Ok(x) => resolver.resolve_line(x).into_iter().map(|y| (line_num + 1, y)).collect(),
                Err(x) => vec![(line_num + 1, x.to_resolve_message())],
            }
        })
        .flatten()
        .collect()
}