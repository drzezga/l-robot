use crate::{parser::node::ASTNodeType, tokenizer::{Operation, Token}};

use super::{*, node::ASTNode, walkers::post_order_mut};

static EXP_TOKENS: &[Token] = &[Token::Operation(Operation::Exp)];
static DIV_TOKENS: &[Token] = &[Token::Operation(Operation::Div)];
static MUL_TOKENS: &[Token] = &[Token::Operation(Operation::Mul)];
static ADD_TOKENS: &[Token] = &[Token::Operation(Operation::Add)];
static SUB_TOKENS: &[Token] = &[Token::Operation(Operation::Sub)];

pub fn parse(tokens: &Vec<Token>) -> Result<ASTNode, ParseError> {
    let mut out = wrap_tokens(tokens);

    // Order of operations:
    //   1. Parens and functions
    //   2. Brackets
    //   3. Negatives
    //   4. Pow
    //   5. Div
    //   6. Mul
    //   7. Add
    //   8. Sub

    let mut tree = parse_parens(&mut out)?;

    parse_brackets(&mut tree);

    parse_negatives(&mut tree);

    // Functions
    walkers::generic_walker(
        &mut tree,
        &|table| matches!(
            table,
            [
                ASTNode { node_type: ASTNodeType::Delimeter(Token::Name(_)), .. },
                ASTNode { node_type: ASTNodeType::Empty, ..}
            ]
        ),
        &|[name, empty]| {
            if let ASTNodeType::Delimeter(Token::Name(name_string)) = name.node_type {
                ASTNode::new(ASTNodeType::Function(name_string), empty.children)
            } else {
                // this is unreachable by the contract of generic_walker
                unreachable!()
            }
        }
    );

    // Division literals (10/3)
    walkers::failing_interfix_walker(
        &mut tree,
        &DIV_TOKENS,
        &|a, b| {
            match (&a.node_type, &b.node_type) {
                (ASTNodeType::Delimeter(Token::Number(_)), ASTNodeType::Delimeter(Token::Number(_))) => Some(ASTNode {
                    node_type: ASTNodeType::Quotient,
                    children: vec![std::mem::take(a), std::mem::take(b)]
                }),
                _ => None
            }
        }
    );

    // Power
    walkers::interfix_walker(
        &mut tree,
        &EXP_TOKENS,
        &|a, b| ASTNode {
            node_type: ASTNodeType::Power,
            children: vec![a, b]
        }
    );

    // Division
    walkers::interfix_walker(
        &mut tree,
        &DIV_TOKENS,
        &|a, b| ASTNode {
            node_type: ASTNodeType::Quotient,
            children: vec![a, b]
        }
    );

    // Multiplication
    walkers::interfix_walker(
        &mut tree,
        &MUL_TOKENS,
        &|a, b| ASTNode {
            node_type: ASTNodeType::Product,
            children: vec![a, b]
        }
    );

    // Addition
    walkers::interfix_walker(
        &mut tree,
        &ADD_TOKENS,
        &|a, b| ASTNode {
            node_type: ASTNodeType::Sum,
            children: vec![a, b]
        }
    );

    // Subtraction
    walkers::interfix_walker(
        &mut tree,
        &SUB_TOKENS,
        &|a, b| ASTNode {
            node_type: ASTNodeType::Difference,
            children: vec![a, b]
        }
    );

    // Lists
    walkers::interfix_walker(
        &mut tree,
        &vec![Token::Comma],
        &|a, b| ASTNode {
            node_type: ASTNodeType::List,
            children: vec![a, b]
        }
    );

    // Equality
    walkers::interfix_walker(
        &mut tree,
        &vec![Token::Equals],
        &|a, b| ASTNode {
            node_type: ASTNodeType::Equality,
            children: vec![a, b]
        }
    );

    // Let bindings
    walkers::prefix_walker(
        &mut tree,
        &vec![Token::Let],
        &|a, | ASTNode {
            node_type: ASTNodeType::Assignment,
            children: vec![a]
        }
    );

    optimise_tree(&mut tree);

    Ok(tree)
}

pub fn wrap_tokens(tokens: &Vec<Token>) -> Vec<ASTNode> {
    tokens
        .iter()
        .map(|x| ASTNode { node_type: ASTNodeType::Delimeter(x.clone()), ..Default::default() })
        .collect()
}

pub fn parse_parens(tokens: &mut Vec<ASTNode>) -> Result<ASTNode, ParseError> {
    // Parens
    // Store the subtree roots in a vec and add them to previous roots on closing parens

    // the first element is the absolute root, must never be popped as it is the result
    let mut roots = vec![ASTNode::default()];

    for token in tokens {
        match token.node_type {
            ASTNodeType::Delimeter(Token::OpeningParen) => {
                roots.push(ASTNode::default());
            }
            ASTNodeType::Delimeter(Token::ClosingParen) => {
                if roots.len() <= 1 {
                    return Err(ParseError::UnmatchedClosingParen);
                } else {
                    // There are >= 2 elements in roots
                    let curr_root = roots.pop().unwrap();

                    roots.last_mut().unwrap().children.push(curr_root);
                }
            }
            _ => {
                if roots.len() >= 1 {
                    let stack_top = roots.last_mut().unwrap();
                    // TODO: maybe find a better way?
                    // without creating the default struct every time
                    // yoink
                    stack_top.children.push(std::mem::take(token));
                }
            }
        }
    }

    if roots.len() != 1 {
        return Err(ParseError::UnmatchedOpeningParen);
    }

    Ok(std::mem::take(&mut roots[0]))
    // roots[0]
}

pub fn parse_brackets(_tree: &mut ASTNode) {
    // println!("parsing brackets");
}

pub fn parse_negatives(tree: &mut ASTNode) {
    post_order_mut(tree, &mut |node| {
        // c-like for loop
        if node.children.len() >= 2 {
            // println!("Interfix walker in {:#?}", node);
            let mut i = 0;
            while i < node.children.len() - 1 {
                match &node.children[i].node_type {
                    ASTNodeType::Delimeter(delimeter) if *delimeter == Token::Operation(Operation::Sub) => {
                        if i == 0 || matches!(node.children[i - 1].node_type, ASTNodeType::Delimeter(Token::Operation(_))) {
                            if let ASTNodeType::Delimeter(Token::Number(num)) = node.children[i + 1].node_type {
                                node.children.splice(i..=(i + 1), vec![ASTNode::number(-num)]);
                            }
                        }
                    }
                    _ => ()
                }
                i += 1;
            }
        }
    });
    // let mut i = 0;
    // while i < tree.children.len() {
    //     match &tree.children[i].node_type {
    //         ASTNodeType::Delimeter(delimeter) if *delimeter == Token::Operation(Operation::Sub) => {
    //             if i != tree.children.len() - 1 {
    //                 if tree.children[i + 1].node_type.is_walkable() {
    //                     parse_negatives(&mut tree.children[i + 1]);
    //                 }

    //                 if i == 0 || !tree.children[i - 1].node_type.is_walkable() {
    //                     if let ASTNodeType::Delimeter(Token::Number(num)) = tree.children[i + 1].node_type {
    //                         let new_node = ASTNode {
    //                             node_type: ASTNodeType::Delimeter(Token::Number(-num)),
    //                             ..Default::default()
    //                         };
    //                         tree.children.splice(i..=(i + 1), vec![new_node]);
    //                     }
    //                 }
    //             }
    //         }
    //         _ => {
    //             // recursively walk subtrees that aren't delimeters and errors
    //             parse_negatives(&mut tree.children[i]);
    //         }
    //     }
    //     i += 1;
    // }
}

pub fn optimise_tree(tree: &mut ASTNode) {
    // println!("optimising");
    // Collapse empty, where possible
    collapse_empty(tree);
}

/// Collapses empty nodes with a single child and performs implied multiplication
pub fn collapse_empty(tree: &mut ASTNode) {
    walkers::post_order_mut(tree, &mut |node| {
        if node.node_type == ASTNodeType::Empty {
            let n = node.children.len();
            if n == 1 {
                let mut el = node.children.pop().unwrap();
                std::mem::swap(node, &mut el);
            } else if n > 1 {
                // skip the last element
                // TODO: Remove this, number of elements decreases
                let mut i = 0;
                while i < node.children.len() - 1 {
                    if is_implied_multiplication(&node.children[i], &node.children[i + 1]) {
                        let removed = node.children
                            .splice(i..=(i + 1), vec![ASTNode::new(ASTNodeType::Product, vec![])])
                            .collect::<Vec<ASTNode>>();
                        node.children[i].children = removed;
                    }
                    i += 1;
                }
            }
        }
    });
    // after walking all children, check if the root can be collapsed
    if tree.node_type == ASTNodeType::Empty && tree.children.len() == 1 {
        let mut el = tree.children.pop().unwrap();
        std::mem::swap(tree, &mut el);
    }
}

/// Decides whether a and b can be multiplied implicitly.
/// Used when no operator is used or to make generated text less verbose.
/// ex. 10(x + 3) is the same as 10 * (x + 3)
pub fn is_implied_multiplication(a: &ASTNode, b: &ASTNode) -> bool {
    match a.node_type {
        // TODO: treat quotients the same as numbers only if quotient children are also numbers
        ASTNodeType::Delimeter(Token::Number(_)) // 10(x + 3)
        | ASTNodeType::Quotient => {
            // 1/2(1/x + 3) // 1/2 x^2 <- should this be allowed? // wolfram says yes, but assumes multiplication
            // according to wolframalpha "10 20" = 200
            match b.node_type {
                ASTNodeType::Sum => true, // yes
                ASTNodeType::Difference => true, // yes
                ASTNodeType::Product => true, // yes
                // ASTNodeType::Quotient => false, // no
                ASTNodeType::Power => match b.children.first().unwrap().node_type { // only if the power base is not a delimeter or another power
                    ASTNodeType::Delimeter(_) | ASTNodeType::Power => false,
                    _ => true
                },
                // ASTNodeType::Equality => false, // no
                ASTNodeType::Delimeter(Token::Name(_)) => true,
                ASTNodeType::Delimeter(Token::Number(_)) => true,
                // ASTNodeType::Function(_) => false, // no
                ASTNodeType::Empty => true, // yes
                _ => false
            }
            // false
        }
        ASTNodeType::Delimeter(Token::Name(_)) => {
            match b.node_type {
                ASTNodeType::Sum // yes
                | ASTNodeType::Difference // yes
                | ASTNodeType::Product // yes
                | ASTNodeType::Quotient // yes
                | ASTNodeType::Delimeter(Token::Name(_)) => true,
                // ASTNodeType::Power => todo!(), // no
                // ASTNodeType::Equality => todo!(), // no
                // ASTNodeType::Function(_) => todo!(), // no
                // ASTNodeType::Empty => todo!(), // yes
                _ => false
            }
            // x(x^2 + 2)
            // x (12) could be x * 12 or f: x(12)
            // false
        }
        ASTNodeType::Power => { // x^2(10)
            false
        }
        // ASTNodeType::Empty => {} // parens
        // ASTNodeType::Sum => {} // ex. 10 3 + 4
        // ASTNodeType::Difference => {} // ex. 10 3 - 4
        // ASTNodeType::Product => {} // ex. 10 3 * 4
        // ASTNodeType::Quotient => {} // ex. 10 1/2
        // ASTNodeType::Equality => { } // what
        // ASTNodeType::Function(_) => {} // no
        _ => false
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::{ParseError, node::{ASTNode, ASTNodeType}, parsers}, tokenizer::{Operation, Token}};

    #[test]
    fn parse_parens() {
        let x = parsers::parse_parens(&mut parsers::wrap_tokens(&mut vec![
            Token::OpeningParen,
            Token::OpeningParen,
            Token::OpeningParen,
            Token::ClosingParen,
            Token::ClosingParen,
            Token::ClosingParen,
            Token::OpeningParen,
            Token::ClosingParen,
        ])).unwrap();
        assert_eq!(x, ASTNode {
            children: vec![
                ASTNode {
                    children: vec![
                        ASTNode {
                            children: vec![
                                ASTNode {
                                    children: vec![],
                                    node_type: ASTNodeType::Empty,
                                },
                            ],
                            node_type: ASTNodeType::Empty,
                        },
                    ],
                    node_type: ASTNodeType::Empty,
                },
                ASTNode {
                    children: vec![],
                    node_type: ASTNodeType::Empty,
                },
            ],
            node_type: ASTNodeType::Empty,
        });
    }
    #[test]
    fn parse_parens_and_other() {
        let x = parsers::parse_parens(&mut parsers::wrap_tokens(&mut vec![
            Token::OpeningParen,
            Token::Name("x".into()),
            Token::Operation(Operation::Add),
            Token::Number(10.0),
            Token::ClosingParen,
            Token::Operation(Operation::Div),
            Token::Number(3.0)
        ])).unwrap();
        assert_eq!(x, ASTNode {
            children: vec![
                ASTNode {
                    children: vec![
                        ASTNode::delimeter(Token::Name("x".into())),
                        ASTNode::delimeter(Token::Operation(Operation::Add)),
                        ASTNode::delimeter(Token::Number(10.0)),
                    ],
                    node_type: ASTNodeType::Empty,
                },
                ASTNode::delimeter(Token::Operation(Operation::Div)),
                ASTNode::delimeter(Token::Number(3.0)),
            ],
            node_type: ASTNodeType::Empty,
        });
    }

    #[test]
    fn parse_parens_errors() {
        let x = parsers::parse_parens(&mut parsers::wrap_tokens(&mut vec![
            Token::OpeningParen,
            Token::ClosingParen,
            Token::OpeningParen,
        ])).unwrap_err();
        assert_eq!(x, ParseError::UnmatchedOpeningParen);

        let x = parsers::parse_parens(&mut parsers::wrap_tokens(&mut vec![
            Token::OpeningParen,
            Token::ClosingParen,
            Token::OpeningParen,
            Token::ClosingParen,
            Token::ClosingParen,
        ])).unwrap_err();
        assert_eq!(x, ParseError::UnmatchedClosingParen);
        // assert_eq!(x.node_type, ASTNodeType::Error("Unmatched closing paren".into()));
    }

    #[test]
    fn parse_negatives() {
        let mut x = parsers::parse_parens(&mut parsers::wrap_tokens(&mut vec![
            Token::OpeningParen,
            Token::Operation(Operation::Sub),
            Token::Number(10.0),
            Token::Operation(Operation::Add),
            Token::Number(7.0),
            Token::ClosingParen,
            Token::Operation(Operation::Sub),
            Token::Number(3.0),
            Token::Operation(Operation::Exp),
            Token::Operation(Operation::Sub),
            Token::Number(10.0)
        ])).unwrap();

        parsers::parse_negatives(&mut x);

        assert_eq!(x, ASTNode {
            children: vec![
                ASTNode {
                    children: vec![
                        ASTNode::delimeter(Token::Number(-10.0)),
                        ASTNode::delimeter(Token::Operation(Operation::Add)),
                        ASTNode::delimeter(Token::Number(7.0)),
                    ],
                    node_type: ASTNodeType::Empty,
                },
                ASTNode::delimeter(Token::Operation(Operation::Sub)),
                ASTNode::delimeter(Token::Number(3.0)),
                ASTNode::delimeter(Token::Operation(Operation::Exp)),
                ASTNode::delimeter(Token::Number(-10.0)),
            ],
            node_type: ASTNodeType::Empty,
        });
    }

    // full parse tests
    #[test]
    fn parse_full_square() {
        let x = parsers::parse(&vec![
            Token::Number(10.),
            Token::Operation(Operation::Exp),
            Token::Number(2.),
        ]).unwrap();
        // println!("{:#?}", &x);
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Power, vec![
                ASTNode::delimeter(Token::Number(10.)),
                ASTNode::delimeter(Token::Number(2.)),
            ])
        );
    }
    #[test]
    fn parse_full_number() {
        let x = parsers::parse(&vec![
            Token::Number(10.),
        ]).unwrap();
        // println!("{:#?}", &x);
        assert_eq!(
            x,
            ASTNode::delimeter(Token::Number(10.)),
        );
    }
    #[test]
    fn parse_full_div() {
        let x = parsers::parse(&vec![
            Token::Number(10.),
            Token::Operation(Operation::Div),
            Token::Number(0.),
        ]).unwrap();
        // println!("{:#?}", &x);
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Quotient, vec![
                ASTNode::delimeter(Token::Number(10.)),
                ASTNode::delimeter(Token::Number(0.)),
            ])
        );
    }
    #[test]
    fn parse_full_sub() {
        let x = parsers::parse(&vec![
            Token::Number(0.005),
            Token::Operation(Operation::Sub),
            Token::Number(0.002),
        ]).unwrap();
        // println!("{:#?}", &x);
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Difference, vec![
                ASTNode::delimeter(Token::Number(0.005)),
                ASTNode::delimeter(Token::Number(0.002)),
            ])
        );
    }
    #[test]
    fn parse_full_negative() {
        let x = parsers::parse(&vec![
            // Token::Number(0.005),
            Token::Operation(Operation::Sub),
            Token::Number(0.002),
        ]).unwrap();
        // println!("{:#?}", &x);
        assert_eq!(
            x,
            ASTNode::number(-0.002),
        );
    }

    #[test]
    fn parse_full_fn() {
        let x = parsers::parse(&vec![
            // Token::Number(0.005),
            Token::Name("my_fun".into()),
            Token::OpeningParen,
            Token::ClosingParen,
        ]).unwrap();
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Function("my_fun".into()), vec![]),
        );
    }

    #[test]
    fn parse_full_fn_args() {
        let x = parsers::parse(&vec![
            // Token::Number(0.005),
            Token::Name("my_fun".into()),
            Token::OpeningParen,
            Token::Number(10.),
            Token::Operation(Operation::Sub),
            Token::Number(3.),
            Token::ClosingParen,
        ]).unwrap();
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Function("my_fun".into()), vec![
                ASTNode::new(ASTNodeType::Difference, vec![
                    ASTNode::number(10.),
                    ASTNode::number(3.)
                ])
            ]),
        );
    }

    #[test]
    fn parse_full_fn_args_nesting() {
        let x = parsers::parse(&vec![
            // Token::Number(0.005),
            Token::Name("my_fun".into()),
            Token::OpeningParen,
            Token::Number(10.),
            Token::Operation(Operation::Mul),
            Token::OpeningParen,
            Token::Operation(Operation::Sub),
            Token::Number(3.),
            Token::Operation(Operation::Add),
            Token::Number(5.),
            Token::ClosingParen,
            Token::ClosingParen,
        ]).unwrap();
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Function("my_fun".into()), vec![
                ASTNode::new(ASTNodeType::Product, vec![
                    ASTNode::number(10.),
                    ASTNode::new(ASTNodeType::Sum, vec![
                        ASTNode::number(-3.),
                        ASTNode::number(5.)
                    ])
                ])
            ]),
        );
    }

    #[test]
    fn is_implied_multiplication() {
        let x = parsers::parse(&vec![
            Token::Number(10.),
            Token::OpeningParen,
            Token::Number(2.),
            Token::Operation(Operation::Add),
            Token::Number(4.),
            Token::ClosingParen,
        ]).unwrap();
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Product, vec![
                ASTNode::delimeter(Token::Number(10.)),
                ASTNode::new(ASTNodeType::Sum, vec![
                    ASTNode::delimeter(Token::Number(2.)),
                    ASTNode::delimeter(Token::Number(4.)),
                ])
            ])
        );
    }

    #[test]
    fn is_implied_multiplication_number() {
        let x = parsers::parse(&vec![
            Token::Number(10.),
            Token::OpeningParen,
            Token::Number(2.),
            Token::ClosingParen,
        ]).unwrap();
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Product, vec![
                ASTNode::delimeter(Token::Number(10.)),
                ASTNode::delimeter(Token::Number(2.)),
            ])
        );
    }

    #[test]
    fn is_implied_multiplication_quotient() {
        let x = parsers::parse(&vec![
            Token::Number(1.),
            Token::Operation(Operation::Div),
            Token::Number(2.),
            Token::OpeningParen,
            Token::Number(2.),
            Token::Operation(Operation::Add),
            Token::Number(4.),
            Token::ClosingParen,
        ]).unwrap();
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Product, vec![
                ASTNode::new(ASTNodeType::Quotient, vec![
                    ASTNode::delimeter(Token::Number(1.)),
                    ASTNode::delimeter(Token::Number(2.)),
                ]),
                ASTNode::new(ASTNodeType::Sum, vec![
                    ASTNode::delimeter(Token::Number(2.)),
                    ASTNode::delimeter(Token::Number(4.)),
                ])
            ])
        );
    }
    #[test]
    fn is_implied_multiplication_num_name() {
        let x = parsers::parse(&vec![
            Token::Number(10.),
            Token::Name("x".into()),
        ]).unwrap();
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Product, vec![
                ASTNode::delimeter(Token::Number(10.)),
                ASTNode::delimeter(Token::Name("x".into())),
            ])
        );
    }
}
