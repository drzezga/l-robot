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
    Error(String),
    Function(String),
    Empty,
}

impl ASTNodeType {
    fn is_walkable(&self) -> bool {
        match self {
            &Self::Error(_) | &Self::Delimeter(_) => false,
            _ => true
        }
    }
}

pub mod parsers {
    use crate::tokenizer::Operation;

    use super::*;

    static EXP_TOKENS: &[Token] = &[Token::Operation(Operation::Exp)];
    static DIV_TOKENS: &[Token] = &[Token::Operation(Operation::Div)];
    static MUL_TOKENS: &[Token] = &[Token::Operation(Operation::Mul)];
    static ADD_TOKENS: &[Token] = &[Token::Operation(Operation::Add)];
    static SUB_TOKENS: &[Token] = &[Token::Operation(Operation::Sub)];

    pub fn parse(tokens: &Vec<Token>) -> ASTNode {
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
    
        let mut tree = parse_parens(&mut out);

        parse_brackets(&mut tree);
        
        parse_negatives(&mut tree);

        // TODO: Special case for fractions with direct numbers being parsed here (to make x^1/2 work)

        // TODO: Inferred multiplying (10x, 10(3 + 4), n(n + 1))

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

        // Subtraction
        walkers::interfix_walker(
            &mut tree,
            &vec![Token::Equals],
            &|a, b| ASTNode {
                node_type: ASTNodeType::Equality,
                children: vec![a, b]
            }
        );

        optimise_tree(&mut tree);

        tree
    }

    pub fn wrap_tokens(tokens: &Vec<Token>) -> Vec<ASTNode> {
        tokens
            .iter()
            .map(|x| ASTNode { node_type: ASTNodeType::Delimeter(x.clone()), ..Default::default() })
            .collect()
    }
    
    pub fn parse_parens(tokens: &mut Vec<ASTNode>) -> ASTNode {
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
                        return ASTNode {
                            node_type: ASTNodeType::Error("Unmatched closing paren".into()),
                            ..Default::default()
                        }
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
            return ASTNode {
                node_type: ASTNodeType::Error("Unmatched opening paren".into()),
                ..Default::default()
            }
        }
        std::mem::take(&mut roots[0])
        // roots[0]
    }
    
    pub fn parse_brackets(tree: &mut ASTNode) {
        println!("parsing brackets");
    }

    pub fn parse_negatives(tree: &mut ASTNode) {
        let mut i = 0;
        while i < tree.children.len() {
            match &tree.children[i].node_type {
                ASTNodeType::Delimeter(delimeter) if *delimeter == Token::Operation(Operation::Sub) => {
                    if i != tree.children.len() - 1 {
                        if tree.children[i + 1].node_type.is_walkable() {
                            parse_negatives(&mut tree.children[i + 1]);
                        }

                        if i == 0 || !tree.children[i - 1].node_type.is_walkable() {
                            if let ASTNodeType::Delimeter(Token::Number(num)) = tree.children[i + 1].node_type {
                                let new_node = ASTNode {
                                    node_type: ASTNodeType::Delimeter(Token::Number(-num)),
                                    ..Default::default()
                                };
                                tree.children.splice(i..=(i + 1), vec![new_node]);
                            }
                        }
                    }
                }
                ASTNodeType::Error(_) => (),
                _ => {
                    // recursively walk subtrees that aren't delimeters and errors
                    parse_negatives(&mut tree.children[i]);
                }
            }
            i += 1;
        }
    }
    
    pub fn optimise_tree(_tree: &mut ASTNode) {
        println!("optimising");
    }
}

pub mod walkers {
    use std::ops::Deref;

    use crate::{parser::ASTNodeType, tokenizer::Token};

    use super::ASTNode;

    /// Walks over a tree and folds expressions of form (* interfix *)
    pub fn interfix_walker<F, T>(tree: &mut ASTNode, interfix_list: &T, create: &F)
        where F : Fn(ASTNode, ASTNode) -> ASTNode, T: Deref<Target = [Token]> {
        // need to iterate over all children to recursively walk nested parens
        let mut i = 0;
        while i < tree.children.len() {
            match &tree.children[i].node_type {
                ASTNodeType::Delimeter(delimeter) if interfix_list.contains(delimeter) => {
                    if i != 0 && i != tree.children.len() - 1 {
                        // the interfix isn't on the edges
                        match &tree.children[i + 1].node_type {
                            ASTNodeType::Error(_) | ASTNodeType::Delimeter(_) => (),
                            _ => {
                                interfix_walker(&mut tree.children[i + 1], interfix_list, create);
                            }
                        }

                        let new_token = create(
                            std::mem::take(&mut tree.children[i - 1]),
                            std::mem::take(&mut tree.children[i + 1])
                        );

                        tree.children.splice((i - 1)..=(i + 1), vec![new_token]);
                        // the inserted element is at position i - 1
                        // decrease i, s.t. the next loop is at i
                        i -= 1;
                    }
                }
                ASTNodeType::Error(_) => (),
                _ => {
                    // recursively walk subtrees that aren't delimeters and errors
                    interfix_walker(&mut tree.children[i], interfix_list, create);
                }
            }
            i += 1;
        }
    }

    /// Walks over a tree and folds expressions of form (prefix *)
    pub fn prefix_walker<F>(tree: &mut ASTNode, prefix: Token, create: F)
        where F : Fn(&mut ASTNode) {
        for node in &mut tree.children {
            if let ASTNodeType::Delimeter(token) = &node.node_type {
                if *token == prefix {

                }
            }
        }
        println!("das");
    }
}

#[cfg(test)]
mod tests {
    use super::{parsers, walkers, *};
    use crate::tokenizer::Operation;

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
        ]));
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

        let x = parsers::parse_parens(&mut parsers::wrap_tokens(&mut vec![
            Token::OpeningParen,
            Token::Name("x".into()),
            Token::Operation(Operation::Add),
            Token::Number(10.0),
            Token::ClosingParen,
            Token::Operation(Operation::Div),
            Token::Number(3.0)
        ]));
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
        ]));
        assert_eq!(x.node_type, ASTNodeType::Error("Unmatched opening paren".into()));

        let x = parsers::parse_parens(&mut parsers::wrap_tokens(&mut vec![
            Token::OpeningParen,
            Token::ClosingParen,
            Token::OpeningParen,
            Token::ClosingParen,
            Token::ClosingParen,
        ]));
        assert_eq!(x.node_type, ASTNodeType::Error("Unmatched closing paren".into()));
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
        ]));

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

    #[test]
    fn interfix_walker() {
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

    // full parse tests
    #[test]
    fn parse() {
        let x = parsers::parse(&vec![
            Token::Number(10.),
            Token::Operation(Operation::Exp),
            Token::Number(2.),
        ]);
        // println!("{:#?}", &x);
        assert_eq!(
            x.children,
            vec![
                ASTNode::new(ASTNodeType::Power, vec![
                    ASTNode::delimeter(Token::Number(10.)),
                    ASTNode::delimeter(Token::Number(2.)),
                ])
            ]
        )
    }
}