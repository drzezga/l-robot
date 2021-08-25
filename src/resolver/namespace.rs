use crate::parser::node::ASTNode;

// TODO: New ASTNode type for values (data types) specifically and a system of operations between them

#[derive(Clone, Debug, PartialEq)]
pub enum NamespaceElement {
    Number(f64),
    // BigNum(f64),
    // Matrix(f64),
    Function(ASTNode)
}

impl NamespaceElement {
    pub fn as_astnode(&self) -> Option<ASTNode> {
        match self {
            NamespaceElement::Number(num) => Some(ASTNode::number(*num)),
            NamespaceElement::Function(_) => None,
            // _ => None,
        }
    }
}