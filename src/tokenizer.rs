#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Name(String),
    Number(f64),
    Operation(Operation),
    Equals,
    OpeningParen,
    ClosingParen,
    OpeningBracket,
    ClosingBracket,
    Empty,
    Err(String)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Add, Sub, Mul, Div, Exp
}

// This can be done in parallel for each line
pub fn tokenize(line: &str) -> Vec<Token> {
    let mut out_vec = Vec::<Token>::new();
    let mut current = String::new();
    let mut is_num = true;

    for char in line.chars() {
        match char {
            '=' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::Equals);
                is_num = true;
            }
            '+' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Add));
                is_num = true;
            }
            '-' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Sub));
                is_num = true;
            }
            '*' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Mul));
                is_num = true;
            }
            '/' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Div));
                is_num = true;
            }
            '^' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::Operation(Operation::Exp));
                is_num = true;
            }
            '(' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::OpeningParen);
                is_num = true;
            }
            ')' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::ClosingParen);
                is_num = true;
            }
            '[' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::OpeningBracket);
                is_num = true;
            }
            ']' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
                    current.clear();
                }
                out_vec.push(Token::ClosingBracket);
                is_num = true;
            }
            ' ' => {
                if !current.is_empty() {
                    out_vec.push(parse_token(&current, is_num));
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
        out_vec.push(parse_token(&current, is_num));
        // current.clear();
    }

    out_vec
}

fn parse_token(to_tokenize: &str, is_num: bool) -> Token {
    if is_num {
        if let Ok(num) = to_tokenize.parse::<f64>() {
            Token::Number(num)
        } else {
            Token::Err(String::from("Error parsing number"))
        }
    } else {
        Token::Name(String::from(to_tokenize))
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_token, tokenize, Token, Operation};
    #[test]
    fn parse_token_creates_names() {
        assert_eq!(parse_token("guacamole", false), Token::Name(String::from("guacamole")));
    }

    #[test]
    fn parse_token_creates_names_with_num() {
        assert_eq!(parse_token("guacamole33", false), Token::Name(String::from("guacamole33")));
    }

    #[test]
    fn tokenize_creates_tokens() {
        assert_eq!(
            tokenize("(x+10)/3"),
            vec![
                Token::OpeningParen,
                Token::Name(String::from("x")),
                Token::Operation(Operation::Add),
                Token::Number(10.0),
                Token::ClosingParen,
                Token::Operation(Operation::Div),
                Token::Number(3.0)
            ]
        );

        assert_eq!(
            tokenize("v_t=10 [m/s^2]"),
            vec![
                Token::Name(String::from("v_t")),
                Token::Equals,
                Token::Number(10.0),
                Token::OpeningBracket,
                Token::Name(String::from("m")),
                Token::Operation(Operation::Div),
                Token::Name(String::from("s")),
                Token::Operation(Operation::Exp),
                Token::Number(2.0),
                Token::ClosingBracket,
            ]
        );
    }
}