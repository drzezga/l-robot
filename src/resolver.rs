pub mod resolve_message;
pub mod namespace;

use std::collections::HashMap;

use crate::{parser::{node::{ASTNode, ASTNodeType}, walkers::post_order}, tokenizer::Token};

use namespace::NamespaceElement;
use resolve_message::ResolveMessage;

#[derive(Clone, Debug, PartialEq)]
pub struct Resolver {
    pub namespace: HashMap<String, NamespaceElement>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            namespace: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, nodes: Vec<(usize, ASTNode)>) -> Vec<(usize, ResolveMessage)> {
        let mut out = Vec::new();
        // let namespace: Namespace = HashMap::new();

        // How many unknowns are in the expression, useful for determining whether we are able to solve it
        // multiple occurances count as different unknowns for now, as we can't solve them yet

        // node is the root node of a line
        for (line_num, node) in nodes {
            out.append(&mut self.resolve_line(node).into_iter().map(|x| (line_num, x)).collect());
        }

        out
    }

    pub fn resolve_line(&mut self, mut root: ASTNode) -> Vec<ResolveMessage> {
        let mut out = Vec::new();
        let mut encountered_unknowns: u32 = 0;

        let mut has_empty = false;
        post_order(&mut root, &mut |x| {
            if x.node_type == ASTNodeType::Empty {
                out.push(ResolveMessage::error("Wrong usage of operation"));
                has_empty = true;
            }
        });

        if has_empty {
            return out;
        }

        post_order(&mut root, &mut |x| {
            match &x.node_type {
                ASTNodeType::Sum => { resolve_numbers(x, |a, b| Ok(a + b)); },
                ASTNodeType::Difference => { resolve_numbers(x, |a, b| Ok(a - b)); },
                ASTNodeType::Product => { resolve_numbers(x, |a, b| Ok(a * b)); },
                ASTNodeType::Quotient => {
                    let result = resolve_numbers(
                        x,
                        |a, b| if b != 0. { Ok(a / b) } else { Err(ResolveMessage::error("Divide by zero")) }
                    );
                    if let Some(err) = result {
                        out.push(err);
                    }
                },
                ASTNodeType::Power => { resolve_numbers(x, |a, b| Ok(f64::powf(a, b))); },
                ASTNodeType::Equality => (), // we're done
                ASTNodeType::Function(f_name) => {
                    // currently if the name is in the namespace, it is multiplication
                    if let Some(_) = self.namespace.get(f_name.as_str()) {
                        // the "function" is actually implied multiplication, we just multiply
                        resolve_numbers(x, |a, b| Ok(a * b));
                    }
                },
                ASTNodeType::Empty => (), // leave it be
                ASTNodeType::List => (), // TODO: Think what should be the behavior here
                ASTNodeType::Assignment => {
                    let equality = &mut x.children[0];
                    match equality.node_type {
                        ASTNodeType::Equality => {
                            // right and left in reverse because of popping order
                            // equality always has 2 children
                            let right = equality.children.pop().unwrap();
                            let left = equality.children.pop().unwrap();

                            match &left.node_type {
                                ASTNodeType::Function(fn_name) => {
                                    let processed = self.process_fn(right);

                                    if let Ok(processed) = processed {
                                        self.namespace.insert(fn_name.clone(), NamespaceElement::Function());

                                    }
                                }
                                _ => ()
                            }
                        }
                        _ => { out.push(ResolveMessage::error("Invalid let assignment")) }
                    }
                }
                ASTNodeType::Delimeter(delimeter) => {
                    match delimeter {
                        Token::Name(name) => {
                            if let Some(element) = self.namespace.get(name) {
                                // The element exists is the namespace
                                if let Some(node) = element.as_astnode() {
                                    // The namespace element can be represented as a node
                                    *x = node;
                                }
                            } else {
                                encountered_unknowns += 1;
                            }
                        }
                        _ => ()
                    };
                },
            };
        });

        // TODO: An assumption is made here, that the unknown is a number
        if encountered_unknowns == 1 {
            if root.node_type == ASTNodeType::Equality {
                match (&root.children[0].node_type, &root.children[1].node_type) {
                    (_, ASTNodeType::Delimeter(Token::Number(_)))
                    | (ASTNodeType::Delimeter(Token::Number(_)), _) => {
                        let (mut unknown_side, mut other_side) = (root.children.pop().unwrap(), root.children.pop().unwrap());
                        if matches!(unknown_side.node_type, ASTNodeType::Delimeter(Token::Number(_))) {
                            std::mem::swap(&mut unknown_side, &mut other_side);
                        }

                        loop {
                            if unknown_side.children.len() == 2 {
                                // right and left are in reverse, as items are popped from the right
                                let (right, left) = (unknown_side.children.pop().unwrap(), unknown_side.children.pop().unwrap());
                                let (unknown_on_left, unknown_side_val) = match (&left.node_type, &right.node_type) {
                                    (ASTNodeType::Delimeter(Token::Number(a)), _) => (false, *a),
                                    (_, ASTNodeType::Delimeter(Token::Number(a))) => (true, *a),
                                    _ => {
                                        out.push(ResolveMessage::error("Side other to unknown is not a number"));
                                        break;
                                    }
                                };
                                let other_side_val = match &mut other_side.node_type {
                                    ASTNodeType::Delimeter(Token::Number(b)) => b,
                                    _ => {
                                        out.push(ResolveMessage::error("Side other to unknown is not a number"));
                                        break;
                                    }
                                };
                                match &unknown_side.node_type {
                                    ASTNodeType::Sum => {
                                        *other_side_val -= unknown_side_val;
                                        unknown_side = if unknown_on_left { left } else { right };
                                    },
                                    ASTNodeType::Difference => {
                                        *other_side_val += unknown_side_val;
                                        unknown_side = if unknown_on_left { left } else { right };
                                    },
                                    ASTNodeType::Product => {
                                        *other_side_val /= unknown_side_val;
                                        unknown_side = if unknown_on_left { left } else { right };
                                    },
                                    ASTNodeType::Quotient => {
                                        if unknown_on_left { // x / 2 = 10 -> x = 20
                                            *other_side_val *= unknown_side_val;
                                            unknown_side = left;
                                        } else { // 2 / x = 10 -> x = 2 / 10
                                            *other_side_val = unknown_side_val / *other_side_val;
                                            unknown_side = right;
                                        }
                                    },
                                    ASTNodeType::Power => {
                                        out.push(ResolveMessage::error("Cannot evaluate powers of unknowns")); // TODO: resolve x^(2n+1)
                                        break;
                                    },
                                    // ASTNodeType::Function(_) => todo!(), // all functions should have been evaluated
                                    // ASTNodeType::Empty => (), // all empty objects should have been converted to parse errors
                                    _ => {
                                        break;
                                    },
                                }
                            } else {
                                match &unknown_side.node_type {
                                    ASTNodeType::Delimeter(Token::Name(name)) => {
                                        // we have arrived at the end
                                        match &other_side.node_type {
                                            ASTNodeType::Delimeter(Token::Number(num)) => {
                                                // let clone = name.clone();
                                                self.namespace.insert(name.to_string(), NamespaceElement::Number(*num));
                                                out.push(ResolveMessage::output(&format!("{} = {}", name, num)));
                                            }
                                            _ => {
                                                out.push(ResolveMessage::error("Could not resolve equation"));
                                            }
                                        }
                                        break;
                                    },
                                    ASTNodeType::Equality => {
                                        out.push(ResolveMessage::error("Multiple equality is disallowed"));
                                        break;
                                    },
                                    _ => {
                                        break;
                                    }
                                }
                            }
                        }
                    },
                    _ => {
                        out.push(ResolveMessage::error("Could not resolve equation"));
                    }
                }
            } else {
                out.push(ResolveMessage::error("Could not resolve expression with unknown"));
                out.push(ResolveMessage::info("To solve this, you can change the expression to an equation"));
            }
        } else if encountered_unknowns > 1 {
            out.push(ResolveMessage::error("Could not resolve with more than one unknown"));
        } else {
            if root.children.len() > 0 {
                if root.node_type == ASTNodeType::Equality {
                    out.push(ResolveMessage::output(&format!("{}", root.children[0] == root.children[1])));
                } else {
                    out.push(ResolveMessage::error("Could not resolve"));
                }
            } else {
                match root.node_type {
                    ASTNodeType::Delimeter(Token::Number(num)) => {
                        out.push(ResolveMessage::output(&format!("? = {}", num)));
                    },
                    _ => {
                        out.push(ResolveMessage::error("Could not resolve"));
                    }
                }
            }
        }

        out
    }

    pub fn process_fn(&mut self, body: ASTNode) -> Result<ASTNode, ResolveMessage> {
        todo!()
    }
}

// this seems like a bad idea
// TODO: replace this ASAP, as we want to be able to do a more generous match
/// Performs an arithmetic operation on the children of `node`, replacing `node` with the result
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
        Some(ResolveMessage::error("Too many children"))
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::node::{ASTNode, ASTNodeType}, resolver::{self, NamespaceElement, ResolveMessage, Resolver}, tokenizer::Token};

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

        let output = resolver::resolve_numbers(&mut node, |_a, _b| Err(ResolveMessage::error("this is a test")));

        assert_eq!(output, Some(ResolveMessage::error("this is a test")));
    }

    #[test]
    fn resolve_line_sum_2_2() {
        let node = ASTNode::new(ASTNodeType::Sum, vec![ASTNode::number(10.), ASTNode::number(2.)]);

        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(node);

        // println!("{:?}", output);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "? = 12");
    }

    #[test]
    fn resolve_unknowns_sum() {
        let node = ASTNode::new(ASTNodeType::Equality, vec![
            ASTNode::new(ASTNodeType::Sum, vec![ASTNode::delimeter(Token::Name("y".into())), ASTNode::number(2.)]),
            ASTNode::number(100.)
        ]);

        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(node);

        // println!("{:?}", output);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "y = 98");
    }

    #[test]
    fn resolve_unknowns_diff() {
        let node = ASTNode::new(ASTNodeType::Equality, vec![
            ASTNode::new(ASTNodeType::Difference, vec![ASTNode::delimeter(Token::Name("y".into())), ASTNode::number(2.)]),
            ASTNode::number(100.)
        ]);

        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(node);

        // println!("{:?}", output);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "y = 102");
    }

    #[test]
    fn resolve_unknowns_product() {
        let node = ASTNode::new(ASTNodeType::Equality, vec![
            ASTNode::new(ASTNodeType::Product, vec![ASTNode::delimeter(Token::Name("y".into())), ASTNode::number(2.)]),
            ASTNode::number(100.)
        ]);

        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(node);

        // println!("{:?}", output);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "y = 50");
    }

    #[test]
    fn resolve_unknowns_quotient_left() {
        let node = ASTNode::new(ASTNodeType::Equality, vec![
            ASTNode::new(ASTNodeType::Quotient, vec![ASTNode::delimeter(Token::Name("y".into())), ASTNode::number(2.)]),
            ASTNode::number(100.)
        ]);

        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(node);

        // println!("{:?}", output);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "y = 200");
    }

    #[test]
    fn resolve_unknowns_quotient_right() {
        let node = ASTNode::new(ASTNodeType::Equality, vec![
            ASTNode::new(ASTNodeType::Quotient, vec![ASTNode::number(2.), ASTNode::delimeter(Token::Name("y".into()))]),
            ASTNode::number(100.)
        ]);

        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(node);

        // println!("{:?}", output);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "y = 0.02");
    }

    #[test]
    fn resolve_inserts_into_namespace() {
        let node = ASTNode::new(ASTNodeType::Equality, vec![
            ASTNode::delimeter(Token::Name("guacamole".into())),
            ASTNode::number(100.)
        ]);

        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(node);

        // println!("{:?}", output);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "guacamole = 100");

        assert_eq!(resolver.namespace.get("guacamole".into()), Some(&NamespaceElement::Number(100.)));
    }
}