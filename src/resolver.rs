use std::collections::HashMap;

use crate::parser::ASTNode;

use crate::parser::walkers::post_order;

pub enum ResolveMessageType {
    Error,
    Info,
    Output,
}

pub enum NamespaceElement {
    Number(f64),
    BigNum(f64), // TODO: More types
    Matrix(f64)
}

type Namespace<'a> = HashMap<&'a str, f64>;

pub fn resolve(nodes: Vec<(usize, ASTNode)>) -> Vec<(ResolveMessageType, String)> {
    let out = Vec::new();
    let namespace: Namespace = HashMap::new();

    for (line_num, mut node) in nodes {
        // let mut reference = &mut node;
        post_order(&mut node, &|x| {
            match &mut x.node_type {
                crate::parser::ASTNodeType::Sum => todo!(),
                crate::parser::ASTNodeType::Difference => todo!(),
                crate::parser::ASTNodeType::Product => todo!(),
                crate::parser::ASTNodeType::Quotient => todo!(),
                crate::parser::ASTNodeType::Power => todo!(),
                crate::parser::ASTNodeType::Equality => todo!(),
                crate::parser::ASTNodeType::Function(f_name) => {

                },
                crate::parser::ASTNodeType::Empty => (), // throw error
                crate::parser::ASTNodeType::Delimeter(_) => (),
            }
        });
    }

    out
}

pub fn resolve_fn(namespace: &Namespace) -> Result<ASTNode, (ResolveMessageType, String)> {
    todo!()
}