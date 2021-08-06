pub mod resolve_message;
pub mod namespace;

use std::collections::{HashMap, HashSet};

use crate::{parser::{node::{ASTNode, ASTNodeType}, walkers::{self, post_order_mut}}, tokenizer::Token};

use namespace::NamespaceElement;
use resolve_message::ResolveMessage;

#[derive(Clone, Debug, PartialEq)]
pub struct Resolver {
    pub namespace: HashMap<String, NamespaceElement>,
}

// enum OperationMode {
//     Equation, Assignment, Expression
// }

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

        // Check if there are no empty astnodes, as they indicate an error
        let mut has_empty = false;
        post_order_mut(&mut root, &mut |x| {
            if x.node_type == ASTNodeType::Empty {
                out.push(ResolveMessage::error("Wrong usage of operation"));
                out.push(ResolveMessage::error("Note: This usually means you forgot a +, -, /, etc."));
                has_empty = true;
            }
        });

        if has_empty {
            return out;
        }

        // Substitute function usages

        // Resolve the root and check for errors
        let resolve_result = match &root.node_type {
            // if the root is an assignment, only resolve the right side
            ASTNodeType::Assignment => self.resolve_expression(&mut root.children[0].children[1]),
            _ => self.resolve_expression(&mut root),
        };

        let encountered_unknowns = match resolve_result {
            Ok(set) => set,
            Err(error) => return error
        };

        // TODO:
        //  [X] check if the root is an assignment and only resolve the right side
        //  [ ] walk the root to substitute functions for their nodes
        //  [ ] substitute args

        // Finish by interpreting the resulting tree
        // If it is a single node, the line resolved successfully
        // Otherwise, walk trough the tree and check where resolving stopped to throw an approppriate error
        match &root.node_type {
            ASTNodeType::Equality => {
                match encountered_unknowns.len() {
                    1 => out.push(self.resolve_equation(&mut root)), // equation
                    0 => { // equality, print true or false
                        if root.node_type == ASTNodeType::Equality {
                            out.push(ResolveMessage::output(&format!("{}", root.children[0] == root.children[1])));
                        } else {
                            out.push(ResolveMessage::error("Could not resolve"));
                        }
                    }
                    _ => out.push(ResolveMessage::error("Could not resolve an equation with more than one unknown")) // equation with more than one unknown
                }
            }
            ASTNodeType::Assignment => { // assignment
                match &root.children[0].node_type {
                    &ASTNodeType::Equality => {
                        let body = root.children[0].children.pop().unwrap();
                        let fn_declaration = root.children[0].children.pop().unwrap(); // TODO: Drill down and check for illegal elements
                        // let (args, body) = (, root.children[0].children.pop().unwrap());
                        match &fn_declaration.node_type {
                            ASTNodeType::Function(name) => {
                                let name = name.clone();
                                let processed_fn = self.process_fn(&fn_declaration.children[0], body);
                                if let Err(error) = processed_fn {
                                    out.push(error);
                                } else {
                                    let (processed_body, arguments) = processed_fn.unwrap();
                                    self.namespace.insert(name.to_string(), NamespaceElement::Function(processed_body));
                                    out.push(ResolveMessage::output(&format!("{}({}) = [...]", name, arguments.join(", "))));
                                }
                            }
                            _ => out.push(ResolveMessage::error("Assignment requires function on left side"))
                        }
                    }
                    _ => out.push(ResolveMessage::error("Let assignments must be followed by a valid equality"))
                }
            }
            _ => { // expression
                if encountered_unknowns.len() == 0 {
                    match root.node_type {
                        ASTNodeType::Delimeter(Token::Number(num)) => {
                            out.push(ResolveMessage::output(&format!("? = {}", num)));
                        },
                        _ => {
                            out.push(ResolveMessage::error("Could not resolve expression")); // TODO: Drill down on error
                        }
                    }
                } else {
                    out.push(ResolveMessage::error("Could not resolve expression with unknown"));
                    out.push(ResolveMessage::info("Hint: To solve for an unknown, make this into an equation"));
                }
            }
        }

        // TODO: An assumption is made here, that the unknown is a number
        // if encountered_unknowns.len() == 1 {
        //     if root.node_type == ASTNodeType::Equality {
        //         match (&root.children[0].node_type, &root.children[1].node_type) {
        //             (_, ASTNodeType::Delimeter(Token::Number(_)))
        //             | (ASTNodeType::Delimeter(Token::Number(_)), _) => {
        //                 let (mut unknown_side, mut other_side) = (root.children.pop().unwrap(), root.children.pop().unwrap());
        //                 if matches!(unknown_side.node_type, ASTNodeType::Delimeter(Token::Number(_))) {
        //                     std::mem::swap(&mut unknown_side, &mut other_side);
        //                 }

        //                 loop {
        //                     if unknown_side.children.len() == 2 {
        //                         // right and left are in reverse, as items are popped from the right
        //                         let (right, left) = (unknown_side.children.pop().unwrap(), unknown_side.children.pop().unwrap());
        //                         let (unknown_on_left, unknown_side_val) = match (&left.node_type, &right.node_type) {
        //                             (ASTNodeType::Delimeter(Token::Number(a)), _) => (false, *a),
        //                             (_, ASTNodeType::Delimeter(Token::Number(a))) => (true, *a),
        //                             _ => {
        //                                 out.push(ResolveMessage::error("Side other to unknown is not a number"));
        //                                 break;
        //                             }
        //                         };
        //                         let other_side_val = match &mut other_side.node_type {
        //                             ASTNodeType::Delimeter(Token::Number(b)) => b,
        //                             _ => {
        //                                 out.push(ResolveMessage::error("Side other to unknown is not a number"));
        //                                 break;
        //                             }
        //                         };
        //                         match &unknown_side.node_type {
        //                             ASTNodeType::Sum => {
        //                                 *other_side_val -= unknown_side_val;
        //                                 unknown_side = if unknown_on_left { left } else { right };
        //                             },
        //                             ASTNodeType::Difference => {
        //                                 *other_side_val += unknown_side_val;
        //                                 unknown_side = if unknown_on_left { left } else { right };
        //                             },
        //                             ASTNodeType::Product => {
        //                                 *other_side_val /= unknown_side_val;
        //                                 unknown_side = if unknown_on_left { left } else { right };
        //                             },
        //                             ASTNodeType::Quotient => {
        //                                 if unknown_on_left { // x / 2 = 10 -> x = 20
        //                                     *other_side_val *= unknown_side_val;
        //                                     unknown_side = left;
        //                                 } else { // 2 / x = 10 -> x = 2 / 10
        //                                     *other_side_val = unknown_side_val / *other_side_val;
        //                                     unknown_side = right;
        //                                 }
        //                             },
        //                             ASTNodeType::Power => {
        //                                 out.push(ResolveMessage::error("Cannot evaluate powers of unknowns")); // TODO: resolve x^(2n+1)
        //                                 break;
        //                             },
        //                             // ASTNodeType::Function(_) => todo!(), // all functions should have been evaluated
        //                             // ASTNodeType::Empty => (), // all empty objects should have been converted to parse errors
        //                             _ => {
        //                                 break;
        //                             },
        //                         }
        //                     } else {
        //                         match &unknown_side.node_type {
        //                             ASTNodeType::Delimeter(Token::Name(name)) => {
        //                                 // we have arrived at the end
        //                                 match &other_side.node_type {
        //                                     ASTNodeType::Delimeter(Token::Number(num)) => {
        //                                         // let clone = name.clone();
        //                                         self.namespace.insert(name.to_string(), NamespaceElement::Number(*num));
        //                                         out.push(ResolveMessage::output(&format!("{} = {}", name, num)));
        //                                     }
        //                                     _ => {
        //                                         out.push(ResolveMessage::error("Could not resolve equation"));
        //                                     }
        //                                 }
        //                                 break;
        //                             },
        //                             ASTNodeType::Equality => {
        //                                 out.push(ResolveMessage::error("Multiple equality is disallowed"));
        //                                 break;
        //                             },
        //                             _ => {
        //                                 break;
        //                             }
        //                         }
        //                     }
        //                 }
        //             },
        //             _ => {
        //                 out.push(ResolveMessage::error("Could not resolve equation"));
        //             }
        //         }
        //     } else {
        //         out.push(ResolveMessage::error("Could not resolve expression with unknown"));
        //         out.push(ResolveMessage::info("To solve this, you can change the expression to an equation"));
        //     }
        // } else if encountered_unknowns.len() > 1 {
        //     out.push(ResolveMessage::error("Could not resolve with more than one unknown"));
        // } else {
        //     if root.children.len() > 0 {
        //         if root.node_type == ASTNodeType::Equality {
        //             out.push(ResolveMessage::output(&format!("{}", root.children[0] == root.children[1])));
        //         } else {
        //             out.push(ResolveMessage::error("Could not resolve"));
        //         }
        //     } else {
        //         match root.node_type {
        //             ASTNodeType::Delimeter(Token::Number(num)) => {
        //                 out.push(ResolveMessage::output(&format!("? = {}", num)));
        //             },
        //             _ => {
        //                 out.push(ResolveMessage::error("Could not resolve"));
        //             }
        //         }
        //     }
        // }

        out
    }

    /// Expands functions from the namespace
    // pub fn expand_functions(&self, expr: &mut ASTNode) -> Result<(), ResolveMessage> {

    //     todo!()
    // }

    /// Returns a result of the set of encountered unknowns or a resolve message containing an error
    pub fn resolve_expression(&self, expr: &mut ASTNode) -> Result<HashSet<String>, Vec<ResolveMessage>> {
        let mut encountered_unknowns = HashSet::<String>::new();
        let mut errors: Vec<ResolveMessage> = vec![];

        post_order_mut(expr, &mut |x| {
            match &x.node_type {
                ASTNodeType::Sum => { resolve_numbers(x, |a, b| Ok(a + b)); },
                ASTNodeType::Difference => { resolve_numbers(x, |a, b| Ok(a - b)); },
                ASTNodeType::Product => { resolve_numbers(x, |a, b| Ok(a * b)); },
                ASTNodeType::Quotient => {
                    let result = resolve_numbers(
                        x,
                        |a, b| if b != 0. { Ok(a / b) } else { Err(ResolveMessage::error("Divide by zero")) } // TODO: Drilldown
                    );
                    if let Some(err) = result {
                        errors.push(err);
                    }
                },
                ASTNodeType::Power => { resolve_numbers(x, |a, b| Ok(f64::powf(a, b))); },
                ASTNodeType::Function(f_name) => { // TODO: Move this out of here to a different loop
                    // currently if the name is in the namespace, it is multiplication
                    if let Some(element) = self.namespace.get(f_name.as_str()) {
                        match element {
                            // the "function" is actually implied multiplication, we just multiply
                            NamespaceElement::Number(_) => {
                                resolve_numbers(x, |a, b| Ok(a * b));
                            }
                            NamespaceElement::Function(body) => {
                                match self.resolve_fn(x, body) {
                                    Ok(set) => encountered_unknowns.extend(set),
                                    Err(mut new_errors) => errors.append(&mut new_errors)
                                }
                                // if let Err(mut error) = self.resolve_fn(x, body) {
                                //     errors.append(&mut error);
                                // }
                            },
                        }
                    }
                },
                ASTNodeType::Empty => (), // leave it be
                ASTNodeType::List => (), // TODO: Think what should be the behavior here
                ASTNodeType::FnArgument(_) => (), // impossible to be here
                ASTNodeType::Assignment => (), // the end
                ASTNodeType::Equality => (), // the end
                // ASTNodeType::Assignment => {
                //     let equality = &mut x.children[0];
                //     match equality.node_type {
                //         ASTNodeType::Equality => {
                //             // right and left in reverse because of popping order
                //             // equality always has 2 children
                //             let body_node = equality.children.pop().unwrap();
                //             let fun_node = equality.children.pop().unwrap();

                //             match &fun_node.node_type {
                //                 ASTNodeType::Function(fn_name) => {
                //                     // let assignment currently only works for functions
                //                     match self.process_fn(&fun_node, body_node) {
                //                         Ok(processed) => { self.namespace.insert(fn_name.clone(), NamespaceElement::Function(processed)); }
                //                         Err(err) => { out.push(err); }
                //                     }
                //                 }
                //                 _ => ()
                //             }
                //         }
                //         _ => { out.push(ResolveMessage::error("Invalid let assignment")) }
                //     }
                // }
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
                                encountered_unknowns.insert(name.clone());
                            }
                        }
                        _ => ()
                    };
                },
            };
        });
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(encountered_unknowns)
        }
    }

    pub fn resolve_equation(&mut self, root: &mut ASTNode) -> ResolveMessage {
        // TODO: An assumption is made here, that the unknown is a number
        let (mut unknown_side, mut other_side) = match (&root.children[0].node_type, &root.children[1].node_type) {
            (_, ASTNodeType::Delimeter(Token::Number(_))) | (ASTNodeType::Delimeter(Token::Number(_)), _) => {
                // Decide what is the unknown side and what is the other side
                let (mut unknown_side, mut other_side) = (root.children.pop().unwrap(), root.children.pop().unwrap());
                if matches!(unknown_side.node_type, ASTNodeType::Delimeter(Token::Number(_))) {
                    std::mem::swap(&mut unknown_side, &mut other_side);
                }
                (unknown_side, other_side)
            },
            _ => {
                // If the equation is not in the above form, it cannot be solved
                return ResolveMessage::error("Equation could not be solved");
            }
        };

        loop {
            if unknown_side.children.len() == 2 {
                // right and left are in reverse, as items are popped from the right
                let (right, left) = (unknown_side.children.pop().unwrap(), unknown_side.children.pop().unwrap());
                let (unknown_on_left, unknown_side_val) = match (&left.node_type, &right.node_type) {
                    (ASTNodeType::Delimeter(Token::Number(a)), _) => (false, *a),
                    (_, ASTNodeType::Delimeter(Token::Number(a))) => (true, *a),
                    _ => return ResolveMessage::error("Side other to unknown is not a number")
                };

                let other_side_val = match &mut other_side.node_type {
                    ASTNodeType::Delimeter(Token::Number(b)) => b,
                    _ => return ResolveMessage::error("Side other to unknown is not a number")
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
                    ASTNodeType::Power => return ResolveMessage::error("Cannot evaluate powers of unknowns"), // TODO: resolve x^(2n+1)
                    _ => ()
                    // ASTNodeType::Function(_) => todo!(), // all functions should have been evaluated
                    // ASTNodeType::Empty => (), // all empty objects should have been converted to parse errors
                }
            } else {
                match &unknown_side.node_type {
                    ASTNodeType::Delimeter(Token::Name(name)) => {
                        // we have arrived at the end
                        match &other_side.node_type {
                            ASTNodeType::Delimeter(Token::Number(num)) => {
                                // let clone = name.clone();
                                self.namespace.insert(name.to_string(), NamespaceElement::Number(*num));
                                return ResolveMessage::output(&format!("{} = {}", name, num));
                            }
                            _ => {
                                return ResolveMessage::error("Could not resolve equation"); // TODO Drill down on error
                            }
                        }
                    },
                    // ASTNodeType::Equality => return ResolveMessage::error("Multiple equality is disallowed"),
                    _ => () // TODO Drill down on error
                }
            }
        }
    }

    /// Processes the function body, substituting and replacing argument names with argument placeholders
    /// @returns Result of a tuple of the resulting Function body and a vector of argument names
    pub fn process_fn(&mut self, args: &ASTNode, mut body: ASTNode) -> Result<(ASTNode, Vec<String>), ResolveMessage> {
        let processed_args = process_fn_args(args)?;
        let mut unknown_name: String = String::new();

        walkers::post_order_mut(&mut body, &mut |x| match &x.node_type {
            ASTNodeType::Delimeter(Token::Name(name)) =>
                if let Some(index) = processed_args.iter().position(|x| x == name) { // could be optimised with a map assigning strings to arg numbers
                    *x = ASTNode::new(ASTNodeType::FnArgument(index), vec![]);
                } else {
                    unknown_name = name.clone();
                },
            _ => ()
        });
        if unknown_name != "" {
            Err(ResolveMessage::error(&format!("Unknown name: {}", unknown_name)))
        } else {
            Ok((body, processed_args))
        }
    }

    fn resolve_fn(&self, node: &mut ASTNode, body: &ASTNode) -> Result<HashSet<String>, Vec<ResolveMessage>> {
        // let args = process_fn_args(&node.children[0])?;
        let args = list_node_to_vec(&node.children[0]);

        let mut working_body = body.clone();

        // std::mem::swap(node, body.clone());

        let mut error: Option<ResolveMessage> = None;

        walkers::post_order_mut(
            &mut working_body,
            &mut |x| if let ASTNodeType::FnArgument(arg) = x.node_type {
                if arg + 1 > args.len() { // TODO: Argument count as function signature, and actual message here
                    error = Some(ResolveMessage::error(&format!("Provided {} args, while this function expects {}", args.len(), args.len() + 1)));
                } else {
                    *x = args[arg].clone();
                }
            }
        );

        *node = working_body;

        let set = self.resolve_expression(node)?;

        if error.is_some() {
            Err(vec![error.unwrap()])
        } else {
            Ok(set)
        }
    }
}

/// Transforms a List ASTNode into a vector of arg names
pub fn process_fn_args(args: &ASTNode) -> Result<Vec<String>, ResolveMessage> {
    // println!("{:?}", args);
    let mut out = vec![];
    let mut has_invalid_args = false;
    walkers::post_order(&args, &mut |x| {
        match &x.node_type { // function args can only have names or other lists
            ASTNodeType::Delimeter(Token::Name(name)) => {
                out.push(name.clone());
            }
            ASTNodeType::List => (),
            _ => { has_invalid_args = true }
        }
    });
    if has_invalid_args {
        Err(ResolveMessage::error("Invalid function arguments"))
    } else {
        Ok(out)
    }
}

pub fn list_node_to_vec(node: &ASTNode) -> Vec<ASTNode> {
    let mut out = vec![];
    walkers::post_order(&node, &mut |x| {
        match &x.node_type {
            ASTNodeType::List => (),
            _ => out.push(x.clone())
        }
    });
    out
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

    #[test]
    fn resolve_assignment_inserts_into_namespace() {
        let node = ASTNode::new(ASTNodeType::Assignment, vec![
            ASTNode::new(ASTNodeType::Equality, vec![
                ASTNode::new(ASTNodeType::Function("fn".into()), vec![
                    ASTNode::delimeter(Token::Name("x".into())),
                ]),
                ASTNode::new(ASTNodeType::Sum, vec![
                    ASTNode::delimeter(Token::Name("x".into())),
                    ASTNode::number(10.),
                ]),
            ]),
        ]);

        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(node);

        // println!("{:?}", output);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "fn(x) = [...]");

        assert_eq!(
            resolver.namespace.get("fn".into()),
            Some(&NamespaceElement::Function(ASTNode::new(ASTNodeType::Sum, vec![
                ASTNode::new(ASTNodeType::FnArgument(0), vec![]),
                ASTNode::number(10.),
            ])))
        );
    }

    #[test]
    fn resolve_fn_works_with_one_arg() {
        let mut resolver = Resolver::new();

        resolver.namespace.insert(String::from("f"), NamespaceElement::Function(
            ASTNode::new(ASTNodeType::Sum, vec![
                ASTNode::new(ASTNodeType::FnArgument(0), vec![]),
                ASTNode::number(4.)
            ]),
        ));

        let node = ASTNode::new(ASTNodeType::Function("f".into()), vec![
            ASTNode::number(10.),
        ]);

        let output = resolver.resolve_line(node);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "? = 14");

        // println!("{:#?}", output);
    }

    #[test]
    fn resolve_fn_works_with_composition() {
        let mut resolver = Resolver::new();

        resolver.namespace.insert(String::from("f"), NamespaceElement::Function(
            ASTNode::new(ASTNodeType::Sum, vec![
                ASTNode::new(ASTNodeType::FnArgument(0), vec![]),
                ASTNode::number(4.)
            ]),
        ));

        resolver.namespace.insert(String::from("g"), NamespaceElement::Function(
            ASTNode::new(ASTNodeType::Power, vec![
                ASTNode::new(ASTNodeType::FnArgument(0), vec![]),
                ASTNode::number(2.)
            ]),
        ));

        let node = ASTNode::new(ASTNodeType::Function("f".into()), vec![
            ASTNode::new(ASTNodeType::Function("g".into()), vec![
                ASTNode::number(10.),
            ]),
        ]);

        let output = resolver.resolve_line(node);

        assert_eq!(output.len(), 1);

        assert_eq!(output.first().unwrap().content, "? = 104");
    }
}