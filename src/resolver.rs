use std::collections::HashMap;

use crate::parser::ASTNode;

use crate::parser::walkers::post_order;

pub enum ResolveMessageType {
    Error,
    Info,
    Output,
}

pub enum ResolveValue {
    Number(f64),
    // BigNum(f64), // TODO: More types
    // Matrix(f64)
}

pub fn resolve(nodes: Vec<(usize, ASTNode)>) -> Vec<(ResolveMessageType, String)> {
    let out = Vec::new();
    let variables: HashMap<&str, ResolveValue> = HashMap::new();

    for (line_num, node) in nodes {
        post_order(node, )
    }

    out
}
