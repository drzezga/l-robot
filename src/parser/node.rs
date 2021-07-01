use crate::tokenizer::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct ASTNode {
    pub children: Vec<ASTNode>, // Maybe make this option, s.t. there are no vector deallocations on every node drop and no allocations on delimeters
    pub node_type: ASTNodeType
}

impl ASTNode {
    pub fn delimeter(token: Token) -> Self {
        ASTNode {
            children: vec![],
            node_type: ASTNodeType::Delimeter(token)
        }
    }

    pub fn number(num: f64) -> Self {
        Self::delimeter(Token::Number(num))
    }

    pub fn empty(children: Vec<ASTNode>) -> Self {
        ASTNode {
            children,
            node_type: ASTNodeType::Empty
        }
    }

    pub fn new(node_type: ASTNodeType, children: Vec<ASTNode>) -> Self {
        ASTNode {
            children,
            node_type
        }
    }
}

impl Default for ASTNode {
    fn default() -> Self {
        ASTNode {
            children: vec![],
            node_type: ASTNodeType::Empty
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNodeType {
    Sum,
    Difference,
    Product,
    Quotient,
    Power,
    Equality,
    Delimeter(Token),
    // Error(String),
    Function(String),
    List,
    Assignment,
    Empty,
}