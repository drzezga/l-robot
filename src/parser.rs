use crate::tokenizer::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct ASTNode {
    children: Vec<ASTNode>,
    // left: Option<Box<Node>>,
    // right: Option<Box<Node>>,
    node_type: ASTNodeType
}

impl ASTNode {
    fn delimeter(token: Token) -> Self {
        ASTNode {
            children: vec![], // Maybe make this option, s.t. there are no vector deallocations on every node drop
            node_type: ASTNodeType::Delimeter(token)
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
    Function(Vec<ASTNode>),
    Empty,
}

pub mod parsers {
    use super::*;

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
    
        parse_powers(&mut tree);
    
        parse_quotients(&mut tree);
    
        parse_products(&mut tree);
    
        parse_sums(&mut tree);
    
        parse_differences(&mut tree);
    
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

    }
    
    pub fn parse_powers(tree: &mut ASTNode) {
        println!("parsing powers");
    }
    
    pub fn parse_quotients(tree: &mut ASTNode) {
        println!("parsing quotients");
    }
    
    pub fn parse_products(tree: &mut ASTNode) {
        println!("parsing products");
    }
    
    pub fn parse_sums(tree: &mut ASTNode) {
        println!("parsing sums");
    }
    
    pub fn parse_differences(tree: &mut ASTNode) {
        println!("parsing differences");
    }
    
    pub fn optimise_tree(_tree: &mut ASTNode) {
        println!("optimising");
    }
}

pub mod walkers {
    use crate::{parser::ASTNodeType, tokenizer::Token};

    use super::ASTNode;

    // TODO: * multiple interfix tokens DONE!
    //       * some way of matching just the token, not the value (possibly not required)
    //       * iterating over indices, not values to access previous vals

    // SOLUTIONS:
    //       * indexed iteration
    //       * chain of responsibility with more abstract steps (hmmm seems it already is a chain of responsibility)
    //       * just a normal set, s.t. Token doesn't have to be Eq, Hash or Ord

    pub fn interfix_walker<F>(tree: &mut ASTNode, interfix_list: &Vec<Token>, create: &F)
        where F : Fn(ASTNode, ASTNode) -> ASTNode {
        // need to iterate over all children to recursively walk over nested parens
        let mut i = 0;
        while i < tree.children.len() {
            match &tree.children[i].node_type {
                ASTNodeType::Delimeter(delimeter) if interfix_list.contains(delimeter) => {
                    if i != 0 && i != tree.children.len() - 1 {
                        // the interfix isn't on the edges
                        if tree.children[i + 1].node_type == ASTNodeType::Empty {
                            // first check if the second argument can be recursively walked and do this now
                            interfix_walker(&mut tree.children[i + 1], interfix_list, create);

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
                ASTNodeType::Empty => {
                    interfix_walker(&mut tree.children[i], interfix_list, create);
                }
                _ => ()
            }
            i += 1;
        }
    }

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
}