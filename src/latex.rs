use crate::parser::parsers::is_implied_multiplication;
use crate::parser::{ASTNode, ASTNodeType};
use crate::tokenizer::{Token, Operation};

use itertools::join;

impl ASTNode {
    pub fn to_latex(&self) -> String {
        match &self.node_type {
            // ASTNodeType::Error(err) | ASTNodeType::Delimeter(Token::Err(err)) => format!("\\text{{{}}}", err),
            ASTNodeType::Delimeter(token) => match token {
                Token::Name(name) => name.into(),
                Token::Number(num) => num.to_string(),
                Token::Operation(op) => match op {
                    Operation::Add => "+".into(),
                    Operation::Sub => "-".into(),
                    Operation::Mul => "*".into(),
                    Operation::Div => "/".into(),
                    Operation::Exp => "^".into(),
                },
                Token::Equals => "=".into(),
                Token::OpeningBracket => "[".into(),
                Token::ClosingBracket => "]".into(),
                _ => "".into()
            }
            ASTNodeType::Empty => {
                if self.children.len() == 0 {
                    "".into()
                } else if self.children.len() == 1 {
                    self.children.first().unwrap().to_latex()
                } else {
                    join(self.children.iter().map(|x| x.to_latex()), " ")
                }
            }
            ASTNodeType::Quotient => {
                if self.children.len() != 2 {
                    "Parsing error - quotient has wrong number of children".into()
                } else {
                    format!("\\frac{{{}}}{{{}}}", self.children[0].to_latex(), self.children[1].to_latex())
                }
            }
            ASTNodeType::Power => {
                if self.children.len() == 2 {
                    let a = match self.children[0].node_type {
                        ASTNodeType::Delimeter(_)
                        | ASTNodeType::Equality
                        | ASTNodeType::Function(_)
                        | ASTNodeType::Empty => self.children[0].to_latex(),
                        _ => format!("({})", self.children[0].to_latex())
                    };
                    let b = match self.children[1].node_type {
                        ASTNodeType::Delimeter(_)
                        | ASTNodeType::Equality
                        | ASTNodeType::Function(_)
                        | ASTNodeType::Empty => self.children[1].to_latex(),
                        _ => format!("{{{}}}", self.children[1].to_latex())
                    };
                    format!("{}^{}", a, b)
                } else {
                    "".into()
                }
            }
            ASTNodeType::Product => {
                // TODO: multiplying fractions creates unnecessary parens
                // currently seems to work, but there could be a caveat
                if self.children.len() == 2 {
                    let a = match self.children[0].node_type {
                        ASTNodeType::Delimeter(_)
                        | ASTNodeType::Equality
                        | ASTNodeType::Function(_)
                        | ASTNodeType::Quotient
                        | ASTNodeType::Empty => self.children[0].to_latex(),
                        _ => format!("({})", self.children[0].to_latex())
                    };
                    let b = match self.children[1].node_type {
                        ASTNodeType::Delimeter(_)
                        | ASTNodeType::Equality
                        | ASTNodeType::Function(_)
                        | ASTNodeType::Quotient
                        | ASTNodeType::Empty => self.children[1].to_latex(),
                        _ => format!("({})", self.children[1].to_latex())
                    };
                    if is_implied_multiplication(&self.children[0], &self.children[1]) {
                        format!("{} {}", a, b)
                    } else {
                        format!("{}*{}", a, b)
                    }
                } else {
                    "".into()
                }
            }
            ASTNodeType::Sum => {
                if self.children.len() == 2 {
                    let a = match self.children[0].node_type {
                        ASTNodeType::Delimeter(_)
                        | ASTNodeType::Equality
                        | ASTNodeType::Function(_)
                        | ASTNodeType::Empty => self.children[0].to_latex(),
                        _ => format!("({})", self.children[0].to_latex())
                    };
                    let b = match self.children[1].node_type {
                        ASTNodeType::Delimeter(_)
                        | ASTNodeType::Equality
                        | ASTNodeType::Function(_)
                        | ASTNodeType::Empty => self.children[1].to_latex(),
                        _ => format!("({})", self.children[1].to_latex())
                    };
                    format!("{}+{}", a, b)
                } else {
                    "".into()
                }
            }
            ASTNodeType::Difference => {
                if self.children.len() == 2 {
                    let a = match self.children[0].node_type {
                        ASTNodeType::Delimeter(_)
                        | ASTNodeType::Equality
                        | ASTNodeType::Function(_)
                        | ASTNodeType::Empty => self.children[0].to_latex(),
                        _ => format!("({})", self.children[0].to_latex())
                    };
                    let b = match self.children[1].node_type {
                        ASTNodeType::Delimeter(_)
                        | ASTNodeType::Equality
                        | ASTNodeType::Function(_)
                        | ASTNodeType::Empty => self.children[1].to_latex(),
                        _ => format!("({})", self.children[1].to_latex())
                    };
                    format!("{}-{}", a, b)
                } else {
                    "".into()
                }
            }
            _ => {
                let separator: &str = match &self.node_type {
                    ASTNodeType::Sum => "+",
                    ASTNodeType::Difference => "-",
                    ASTNodeType::Product => "*",
                    ASTNodeType::Quotient => "/",
                    ASTNodeType::Power => "^",
                    ASTNodeType::Equality => "=",
                    ASTNodeType::Function(name) => name.as_str(),
                    // ASTNodeType::Error(err) => err.as_str(),
                    _ => "",
                };

                join(self.children
                    .iter()
                    .map(|child| child.to_latex()),
                    separator
                )
            }
        }


    }
}
