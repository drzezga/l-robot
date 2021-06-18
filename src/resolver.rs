use std::collections::HashMap;

use crate::tokenizer::Token;

use crate::parser::{ASTNode, ASTNodeType};

use crate::parser::walkers::post_order;

#[derive(Clone, Debug, PartialEq)]
pub struct Resolver<'a> {
    namespace: HashMap<&'a str, f64>,
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self {
        Self {
            namespace: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, nodes: Vec<(usize, ASTNode)>) -> Vec<(usize, ResolveMessage)> {
        let mut out = Vec::new();
        // let namespace: Namespace = HashMap::new();

        // How many unknowns are in the expression, useful for determining whether we are able to solve it
        // multiple occurances count as different unknowns for now, as we can't yet solve them

        // node is the root node of a line
        for (line_num, node) in nodes {
            out.append(&mut self.resolve_line(node).into_iter().map(|x| (line_num, x)).collect());
        }

        out
    }

    pub fn resolve_line(&mut self, mut root: ASTNode) -> Vec<ResolveMessage> {
        let mut out = Vec::new();
        let mut encountered_unknowns: u32 = 0;

        post_order(&mut root, &mut |x| {
            match &x.node_type {
                ASTNodeType::Sum => { resolve_numbers(x, |a, b| Ok(a + b)); },
                ASTNodeType::Difference => { resolve_numbers(x, |a, b| Ok(a - b)); },
                ASTNodeType::Product => { resolve_numbers(x, |a, b| Ok(a * b)); },
                ASTNodeType::Quotient => {
                    let result = resolve_numbers(
                        x,
                        |a, b| if b != 0. { Ok(a / b) } else { Err((ResolveMessageType::Error, "Divide by zero".into())) }
                    );
                    if let Some(err) = result {
                        out.push(err);
                    }
                },
                ASTNodeType::Power => { resolve_numbers(x, |a, b| Ok(f64::powf(a, b))); },
                ASTNodeType::Equality => todo!(),
                ASTNodeType::Function(f_name) => {
                    // currently if the name is in the namespace, it is multiplication
                    if let Some(_) = self.namespace.get(f_name.as_str()) {
                        // the "function" is actually implied multiplication, we just multiply
                        resolve_numbers(x, |a, b| Ok(a * b));
                    }
                },
                ASTNodeType::Empty => (), // leave it be
                ASTNodeType::Delimeter(delimeter) => {
                    match delimeter {
                        Token::Name(name) => {
                            // Check if the name has been defined in the namespace and substitute it
                            if let Some(value) = self.namespace.get(name.as_str()) {
                                *x = ASTNode::number(*value); // learned something here
                                // you can assign a new value to a mutable reference by dereferencing
                                // instead of:
                                // std::mem::replace(x, ASTNode::delimeter(Token::Number(*value)));
                            } else {
                                // the name is an unknown
                                encountered_unknowns += 1;
                            }
                        }
                        // Token::Number => ()
                        _ => ()
                    };
                },
            };
        });

        if root.children.len() > 0 {
            out.push((ResolveMessageType::Error, "Could not resolve".into()));
        } else {
            match root.node_type {
                ASTNodeType::Delimeter(Token::Number(num)) => {
                    out.push((ResolveMessageType::Output, format!("? = {}", num).into()));
                },
                _ => ()
            }
        }

        out
    }

    pub fn resolve_fn(&mut self, _name: &str) -> Result<ASTNode, ResolveMessage> {
        todo!()
    }
}

// this seems like a bad idea
// TODO: replace this ASAP, as we want to do a more generous match
fn resolve_numbers(node: &mut ASTNode, operate: fn(f64, f64) -> Result<f64, ResolveMessage>) -> Option<ResolveMessage> {
    if node.children.len() == 2 {
        match (&node.children[0].node_type, &node.children[1].node_type) {
            (ASTNodeType::Delimeter(Token::Number(a)), ASTNodeType::Delimeter(Token::Number(b))) => {
                let result = operate(*a, *b);
                if let Ok(num) = result {
                    *node = ASTNode::number(num);
                } else {
                    return Some(result.unwrap_err());
                }
            }
            _ => ()
        }
        None
    } else {
        Some((ResolveMessageType::Error, "Too many children".into()))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResolveMessageType {
    Error,
    Info,
    Output,
}

type ResolveMessage = (ResolveMessageType, String);

/// For future use, now names can be only numbers
pub enum NamespaceElement {
    Number(f64),
    BigNum(f64), // TODO: More types
    // Matrix(f64),
    Function(Box<dyn Fn(&mut Vec<ASTNode>) -> ASTNode>)
}

#[cfg(test)]
mod tests {
    use crate::{parser::{ASTNode, ASTNodeType}, resolver::{self, ResolveMessageType, Resolver}};

    #[test]
    fn resolve_numbers() {
        let mut node = ASTNode::new(ASTNodeType::Sum, vec![ASTNode::number(10.), ASTNode::number(2.)]);

        let output = resolver::resolve_numbers(&mut node, |a, b| Ok(a + b));

        assert!(output.is_none());

        assert_eq!(node, ASTNode::number(12.));
    }

    #[test]
    fn resolve_numbers_error() {
        let mut node = ASTNode::new(ASTNodeType::Sum, vec![ASTNode::number(10.), ASTNode::number(2.)]);

        let output = resolver::resolve_numbers(&mut node, |_a, _b| Err((ResolveMessageType::Error, "this is a test".into())));

        assert_eq!(output, Some((ResolveMessageType::Error, "this is a test".into())));
    }

    #[test]
    fn resolve_line_sum_2_2() {
        let node = ASTNode::new(ASTNodeType::Sum, vec![ASTNode::number(10.), ASTNode::number(2.)]);

        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(node);

        // println!("{:?}", output);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().1, "? = 12");
    }
}