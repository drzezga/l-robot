use crate::parser::{ASTNode, ASTNodeType};
use crate::tokenizer::{Token, Operation};

use itertools::join;

impl ASTNode {
    pub fn to_latex(&self) -> String {
        match &self.node_type {
            ASTNodeType::Error(err) | ASTNodeType::Delimeter(Token::Err(err)) => format!("\\text{{{}}}", err),
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
            ASTNodeType::Empty => if let Some(child) = self.children.first() { format!("{{{}}}", child.to_latex()) } else { String::new() },
            ASTNodeType::Quotient => {
                if self.children.len() != 2 {
                    "Parsing error - quotient has wrong number of children".into()
                } else {
                    format!("\\frac{{{}}}{{{}}}", self.children[0].to_latex(), self.children[1].to_latex())
                }
            }
            ASTNodeType::Power => {
                if self.children.len() != 2 {
                    "Parsing error - power has wrong number of children".into()
                } else {
                    let a = self.children[0].to_latex();
                    let b = if let ASTNodeType::Delimeter(_) = self.children[1].node_type {
                        self.children[1].to_latex()
                    } else {
                        format!("{{{}}}", self.children[1].to_latex())
                    };
                    format!("{}^{}", a, b)
                }
            }
            ASTNodeType::Product => {
                // TODO: multiplying fractions creates unnecessary parens 
                if self.children.len() == 2 {
                    let a = if let ASTNodeType::Delimeter(_) = self.children[0].node_type {
                        self.children[0].to_latex()
                    } else {
                        format!("({})", self.children[0].to_latex())
                    };
                    let b = if let ASTNodeType::Delimeter(_) = self.children[1].node_type {
                        self.children[1].to_latex()
                    } else {
                        format!("({})", self.children[1].to_latex())
                    };
                    format!("{}*{}", a, b)
                } else {
                    "".into()
                }
            }
            ASTNodeType::Sum => {
                if self.children.len() == 2 {
                    let a = if let ASTNodeType::Delimeter(_) = self.children[0].node_type {
                        self.children[0].to_latex()
                    } else {
                        format!("({})", self.children[0].to_latex())
                    };
                    let b = if let ASTNodeType::Delimeter(_) = self.children[1].node_type {
                        self.children[1].to_latex()
                    } else {
                        format!("({})", self.children[1].to_latex())
                    };
                    format!("{}+{}", a, b)
                } else {
                    "".into()
                }
            }
            ASTNodeType::Difference => {
                if self.children.len() == 2 {
                    let a = if let ASTNodeType::Delimeter(_) = self.children[0].node_type {
                        self.children[0].to_latex()
                    } else {
                        format!("({})", self.children[0].to_latex())
                    };
                    let b = if let ASTNodeType::Delimeter(_) = self.children[1].node_type {
                        self.children[1].to_latex()
                    } else {
                        format!("({})", self.children[1].to_latex())
                    };
                    format!("{}-{}", a, b)
                } else {
                    "".into()
                }
            }
            _ => {
                // this is where the fun begins
                // let mut i = 0;

                let separator: &str = match &self.node_type {
                    ASTNodeType::Sum => "+",
                    ASTNodeType::Difference => "-",
                    ASTNodeType::Product => "*",
                    ASTNodeType::Quotient => "/",
                    ASTNodeType::Power => "^",
                    ASTNodeType::Equality => "=",
                    ASTNodeType::Function(name) => name.as_str(),
                    ASTNodeType::Error(err) => err.as_str(),
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