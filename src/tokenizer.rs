#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Name(String),
    Number(f64),
    Boolean(bool),
    Operation(Operation),
    Equals,
    OpeningParen,
    ClosingParen,
    OpeningBracket,
    ClosingBracket,
    Empty
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add, Sub, Mul, Div, Exp
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizingError {
    ParseFloatError(std::num::ParseFloatError)
}

// This can be done in parallel for each line
pub fn tokenize(line: &str) -> Result<Vec<Token>, TokenizingError> {
    let mut out_vec = Vec::<Token>::new();
    let mut current = String::new();
    let mut is_num = true;

    for char in line.chars() {
        match char {
            '=' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::Equals);
                is_num = true;
            }
            '+' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Add));
                is_num = true;
            }
            '-' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Sub));
                is_num = true;
            }
            '*' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Mul));
                is_num = true;
            }
            '/' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Div));
                is_num = true;
            }
            '^' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Exp));
                is_num = true;
            }
            '(' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::OpeningParen);
                is_num = true;
            }
            ')' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::ClosingParen);
                is_num = true;
            }
            '[' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::OpeningBracket);
                is_num = true;
            }
            ']' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                out_vec.push(Token::ClosingBracket);
                is_num = true;
            }
            ' ' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num)?);
                    current.clear();
                }
                // out_vec.push(Token::Equals);
                is_num = true;
            }
            '0'..='9' => {
                current.push(char);
            }
            '.' => {
                current.push(char);
            }
            _ => {
                current.push(char);
                is_num = false;
            }
        }
    }
    if !current.is_empty() {
        out_vec.push(parse_token(&current, is_num)?);
        // current.clear();
    }

    Ok(out_vec)
}

fn parse_token(to_tokenize: &str, is_num: bool) -> Result<Token, TokenizingError> {
    if is_num {
        // turbofish pog
        match to_tokenize.parse::<f64>() {
            Ok(num) => Ok(Token::Number(num)),
            Err(err) => Err(TokenizingError::ParseFloatError(err))
        }
    } else {
        Ok(Token::Name(to_tokenize.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_token_works() {
        assert_eq!(parse_token("guacamole", false).unwrap(), Token::Name("guacamole".into()));

        assert_eq!(parse_token("guacamole33", false).unwrap(), Token::Name("guacamole33".into()));

        assert_eq!(parse_token("33", true).unwrap(), Token::Number(33.0));

        assert!(parse_token("thisisdefinetelynotanumber", true).is_err());
    }

    #[test]
    fn tokenize_creates_works() {
        assert_eq!(
            tokenize("(x+10)/3").unwrap(),
            vec![
                Token::OpeningParen,
                Token::Name("x".into()),
                Token::Operation(Operation::Add),
                Token::Number(10.0),
                Token::ClosingParen,
                Token::Operation(Operation::Div),
                Token::Number(3.0)
            ]
        );

        assert_eq!(
            tokenize("v_t=10 [m/s^2]").unwrap(),
            vec![
                Token::Name("v_t".into()),
                Token::Equals,
                Token::Number(10.0),
                Token::OpeningBracket,
                Token::Name("m".into()),
                Token::Operation(Operation::Div),
                Token::Name("s".into()),
                Token::Operation(Operation::Exp),
                Token::Number(2.0),
                Token::ClosingBracket,
            ]
        );
    }
}
