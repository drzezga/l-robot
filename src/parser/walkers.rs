use std::ops::Deref;
use std::convert::TryInto;

use crate::tokenizer::Token;

use super::node::{ASTNode, ASTNodeType};

/// Walks over every element in the tree, pre-order, calling modify. Stops on delimeters.
/// Modifies elements from top to bottom.
// pub fn pre_order<F>(tree: &mut ASTNode, modify: &F)
//     where F : Fn(&mut ASTNode) {
//     // iterate recursively over everything, stopping on delimeters
//     modify(tree);
//     match tree.node_type {
//         ASTNodeType::Delimeter(_) => (),
//         _ => {
//             for child in &mut tree.children {
//                 pre_order(child, modify);
//             }
//         }
//     }
// }

/// Walks over every element in the tree, post-order, calling modify. Stops on delimeters.
/// Modifies elements bottom up.
pub fn post_order<F>(tree: &mut ASTNode, modify: &mut F)
    where F : FnMut(&mut ASTNode) {
    // iterate recursively over everything, stopping on delimeters
    match tree.node_type {
        ASTNodeType::Delimeter(_) => (),
        _ => {
            for child in &mut tree.children {
                post_order(child, modify);
            }
        }
    }
    modify(tree);
}

// Interfix walker reimplemented with standard node traversal functions
// TODO: Interfix walker as a special case for a generic walker
/// Walks over a tree and folds expressions of form (* interfix *)
pub fn interfix_walker<F, T>(tree: &mut ASTNode, interfix_list: &T, create: &F)
    where F : Fn(ASTNode, ASTNode) -> ASTNode, T: Deref<Target = [Token]> {
    post_order(tree, &mut |node| {
        // c-like for loop
        if node.children.len() >= 3 {
            let mut i = 1;
            while i < node.children.len() - 1 {
                match &node.children[i].node_type {
                    ASTNodeType::Delimeter(delimeter) if interfix_list.contains(delimeter) => {
                        let new_token = create(
                            std::mem::take(&mut node.children[i - 1]),
                            std::mem::take(&mut node.children[i + 1])
                        );

                        // println!("Creating token, {:#?}", new_token);

                        node.children.splice((i - 1)..=(i + 1), vec![new_token]);
                        // the inserted element is at position i - 1
                        // decrease i, s.t. the next loop is at i
                        i -= 1;
                    }
                    _ => ()
                }
                i += 1;
            }
        }
    });
}

// Interfix walker reimplemented with standard node traversal functions
/// Walks over a tree and folds expressions of form (* interfix *)
pub fn failing_interfix_walker<F, T>(tree: &mut ASTNode, interfix_list: &T, create: &F)
    where F : Fn(&mut ASTNode, &mut ASTNode) -> Option<ASTNode>, T: Deref<Target = [Token]> {
    post_order(tree, &mut |node| {
        // c-like for loop
        let mut i = 1;
        if node.children.len() >= 3 {
            while i < node.children.len() - 1 {
                match &node.children[i].node_type {
                    ASTNodeType::Delimeter(delimeter) if interfix_list.contains(delimeter) => {
                        let (split_first, split_second) = node.children.split_at_mut(i);
                        if let Some(new_token) = create(
                            split_first.last_mut().unwrap(),
                            &mut split_second[1]
                        ) {
                            node.children.splice((i - 1)..=(i + 1), vec![new_token]);
                            // the inserted element is at position i - 1
                            // decrease i, s.t. the next loop is at i
                            i -= 1;
                        }
                    }
                    _ => ()
                }
                i += 1;
            }
        }
    });
}

/// Walks over a tree and folds expressions of form (* interfix *)
// pub fn interfix_walker<F, T>(tree: &mut ASTNode, interfix_list: &T, create: &F)
//     where F : Fn(ASTNode, ASTNode) -> ASTNode, T: Deref<Target = [Token]> {
//     // need to iterate over all children and recursively walk nested parens
//     let mut i = 0;
//     while i < tree.children.len() {
//         match &tree.children[i].node_type {
//             ASTNodeType::Delimeter(delimeter) if interfix_list.contains(delimeter) => {
//                 if i != 0 && i != tree.children.len() - 1 {
//                     // the interfix isn't on the edges
//                     match &tree.children[i + 1].node_type {
//                         ASTNodeType::Delimeter(_) => (),
//                         _ => {
//                             interfix_walker(&mut tree.children[i + 1], interfix_list, create);
//                         }
//                     }

//                     let new_token = create(
//                         std::mem::take(&mut tree.children[i - 1]),
//                         std::mem::take(&mut tree.children[i + 1])
//                     );

//                     tree.children.splice((i - 1)..=(i + 1), vec![new_token]);
//                     // the inserted element is at position i - 1
//                     // decrease i, s.t. the next loop is at i
//                     i -= 1;
//                 }
//             }
//             _ => {
//                 // recursively walk subtrees that aren't delimeters and errors
//                 interfix_walker(&mut tree.children[i], interfix_list, create);
//             }
//         }
//         i += 1;
//     }
// }

// pub fn failing_interfix_walker<F, T>(tree: &mut ASTNode, interfix_list: &T, create: &F)
//     where F : Fn(ASTNode, ASTNode) -> Option<ASTNode>, T: Deref<Target = [Token]> {
//     // need to iterate over all children and recursively walk nested parens
//     let mut i = 0;
//     while i < tree.children.len() {
//         match &tree.children[i].node_type {
//             ASTNodeType::Delimeter(delimeter) if interfix_list.contains(delimeter) => {
//                 if i != 0 && i != tree.children.len() - 1 {
//                     // the interfix isn't on the edges
//                     match &tree.children[i + 1].node_type {
//                         ASTNodeType::Delimeter(_) => (),
//                         _ => {
//                             failing_interfix_walker(&mut tree.children[i + 1], interfix_list, create);
//                         }
//                     }

//                     if let Some(new_token) = create(
//                         std::mem::take(&mut tree.children[i - 1]),
//                         std::mem::take(&mut tree.children[i + 1])
//                     ) {
//                         tree.children.splice((i - 1)..=(i + 1), vec![new_token]);
//                         // the inserted element is at position i - 1
//                         // decrease i, s.t. the next loop is at i
//                         i -= 1;
//                     }
//                 }
//             }
//             _ => {
//                 // recursively walk subtrees that aren't delimeters and errors
//                 failing_interfix_walker(&mut tree.children[i], interfix_list, create);
//             }
//         }
//         i += 1;
//     }
// }

/// Walks over a tree and folds expressions of form (prefix *)
pub fn prefix_walker<F, T>(tree: &mut ASTNode, prefix_list: &T, create: &F)
    where F : Fn(ASTNode) -> ASTNode, T: Deref<Target = [Token]> {
    post_order(tree, &mut |node| {
        // c-like for loop, because node.children.len() changes
        if node.children.len() >= 2 {
            let mut i = 0;
            while i < node.children.len() - 1 {
                match &node.children[i].node_type {
                    ASTNodeType::Delimeter(delimeter) if prefix_list.contains(delimeter) => {
                        let new_token = create(
                            // std::mem::take(&mut node.children[i - 1]),
                            std::mem::take(&mut node.children[i + 1])
                        );

                        node.children.splice(i..=(i + 1), vec![new_token]);
                    }
                    _ => ()
                }
                i += 1;
            }
        }
    });
}

/// Walks over a tree, post-order, using `match_fn` as a conditional on slices of length `N` to create a new token with `create`.
///
/// The argument of `create` is always a table of size N, on which `match_fn` returned true.
pub fn generic_walker<F, T, const N: usize>(tree: &mut ASTNode, match_fn: &T, create: &F)
    where T: Fn(&[ASTNode]) -> bool, F : Fn([ASTNode; N]) -> ASTNode {
    post_order(tree, &mut |node| {
        // c-like for loop
        if node.children.len() >= N {
            let mut i = 0;
            // instead of while i <= node.children.len() - N to avoid negative numbers in usize
            while i + N <= node.children.len() {
                if match_fn(&node.children[i..(i + N)]) {
                    let spliced: Vec<_> = node.children.splice(i..(i + N), vec![ASTNode::empty(vec![])]).collect();
                    let result: [ASTNode; N] = spliced.try_into().unwrap(); // the vec will always be exactly N elements
                    let new_token = create(result);

                    node.children[i] = new_token;
                }
                i += 1;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::{parser::{node::{ASTNode, ASTNodeType}, parsers, walkers}, tokenizer::{Operation, Token}};

    #[test]
    fn interfix_walker_div_sum() {
        // simple test
        let mut x = ASTNode {
            children: parsers::wrap_tokens(&mut vec![
                Token::Number(10.),
                Token::Operation(Operation::Div),
                Token::Number(2.),
                Token::Operation(Operation::Add),
                Token::Number(7.),
            ]),
            ..Default::default()
        };

        walkers::interfix_walker(
            &mut x,
            &vec![Token::Operation(Operation::Div)],
            &|a, b| ASTNode {
                node_type: ASTNodeType::Quotient,
                children: vec![a, b]
            }
        );

        assert_eq!(
            x.children,
            vec![
                ASTNode {
                    node_type: ASTNodeType::Quotient,
                    children: parsers::wrap_tokens(&vec![
                        Token::Number(10.),
                        Token::Number(2.),
                    ])
                },
                ASTNode::delimeter(Token::Operation(Operation::Add)),
                ASTNode::delimeter(Token::Number(7.))
            ]
        )
    }

    #[test]
    fn interfix_walker_recursion() {
        let mut x = ASTNode {
            children: vec![
                ASTNode::delimeter(Token::Number(7.)),
                ASTNode::delimeter(Token::Operation(Operation::Div)),
                ASTNode::empty(vec![
                    ASTNode::delimeter(Token::Number(13.)),
                    ASTNode::delimeter(Token::Operation(Operation::Div)),
                    ASTNode::delimeter(Token::Number(9.))
                ])
            ],
            ..Default::default()
        };

        walkers::interfix_walker(
            &mut x,
            &vec![Token::Operation(Operation::Div)],
            &|a, b| ASTNode {
                node_type: ASTNodeType::Quotient,
                children: vec![a, b]
            }
        );

        assert_eq!(
            x.children,
            vec![
                ASTNode::new(ASTNodeType::Quotient, vec![
                    ASTNode::delimeter(Token::Number(7.)),
                    ASTNode::empty(vec![
                        ASTNode::new(ASTNodeType::Quotient, vec![
                            ASTNode::delimeter(Token::Number(13.)),
                            ASTNode::delimeter(Token::Number(9.))
                        ])
                    ])
                ])
            ]
        )
    }

    #[test]
    fn generic_walker_interfix() {
        let mut x = ASTNode {
            children: vec![
                ASTNode::delimeter(Token::Number(7.)),
                ASTNode::delimeter(Token::Operation(Operation::Div)),
                ASTNode::empty(vec![
                    ASTNode::delimeter(Token::Number(13.)),
                    ASTNode::delimeter(Token::Operation(Operation::Div)),
                    ASTNode::delimeter(Token::Number(9.))
                ])
            ],
            ..Default::default()
        };

        walkers::generic_walker(
            &mut x,
            &|table: &[ASTNode]| 
                matches!(table[1].node_type, ASTNodeType::Delimeter(Token::Operation(Operation::Div))),
            &|[a, _, c]| ASTNode {
                node_type: ASTNodeType::Quotient,
                children: vec![a, c]
            }
        );

        assert_eq!(
            x.children,
            vec![
                ASTNode::new(ASTNodeType::Quotient, vec![
                    ASTNode::delimeter(Token::Number(7.)),
                    ASTNode::empty(vec![
                        ASTNode::new(ASTNodeType::Quotient, vec![
                            ASTNode::delimeter(Token::Number(13.)),
                            ASTNode::delimeter(Token::Number(9.))
                        ])
                    ])
                ])
            ]
        )
    }
}