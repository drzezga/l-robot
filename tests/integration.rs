mod tests {
    use l_robot::{parser::{node::{ASTNode, ASTNodeType}, parsers::parse}, resolver::{Resolver, resolve_message::{ResolveMessage, ResolveMessageType}}, tokenizer::{Token, tokenize}};

    #[test]
    fn tokenize_parse_x_squared() {
        let x = tokenize("x^2").unwrap();
        let x = parse(&x).unwrap();
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Power, vec![
                ASTNode::delimeter(Token::Name("x".into())),
                ASTNode::delimeter(Token::Number(2.0)),
            ])
        );
    }

    #[test]
    fn tokenize_parse_gravitational_force() {
        // F_g=G*(m_1*m_2)/r^2

        let x = tokenize("F_g=G*(m_1*m_2)/r^2").unwrap();
        let x = parse(&x).unwrap();

        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Equality, vec![
                ASTNode::delimeter(Token::Name("F_g".into())),
                ASTNode::new(ASTNodeType::Product, vec![
                    ASTNode::delimeter(Token::Name("G".into())),
                    ASTNode::new(ASTNodeType::Quotient, vec![
                        ASTNode::new(ASTNodeType::Product, vec![
                            ASTNode::delimeter(Token::Name("m_1".into())),
                            ASTNode::delimeter(Token::Name("m_2".into())),
                        ]),
                        ASTNode::new(ASTNodeType::Power, vec![
                            ASTNode::delimeter(Token::Name("r".into())),
                            ASTNode::delimeter(Token::Number(2.)),
                        ])
                    ])
                ]),
            ])
        );
    }
    #[test]
    fn tokenize_parse_implied_mult_div_literal() {
        let x = tokenize("100(10 + 3)^3/4").unwrap();
        let x = parse(&x).unwrap();

        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Product, vec![
                ASTNode::number(100.),
                ASTNode::new(ASTNodeType::Power, vec![
                    ASTNode::new(ASTNodeType::Sum, vec![
                        ASTNode::number(10.),
                        ASTNode::number(3.),
                    ]),
                    ASTNode::new(ASTNodeType::Quotient, vec![
                        ASTNode::number(3.),
                        ASTNode::number(4.),
                    ])
                ]),
            ])
        );
    }
    #[test]
    fn tokenize_parse_many_sums() {
        let x = tokenize("1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10").unwrap();
        let x = parse(&x).unwrap();

        assert_eq!(
            x, // all hail the great staircase
            ASTNode::new(ASTNodeType::Sum, vec![
                ASTNode::new(ASTNodeType::Sum, vec![
                    ASTNode::new(ASTNodeType::Sum, vec![
                        ASTNode::new(ASTNodeType::Sum, vec![
                            ASTNode::new(ASTNodeType::Sum, vec![
                                ASTNode::new(ASTNodeType::Sum, vec![
                                    ASTNode::new(ASTNodeType::Sum, vec![
                                        ASTNode::new(ASTNodeType::Sum, vec![
                                            ASTNode::new(ASTNodeType::Sum, vec![
                                                ASTNode::number(1.),
                                                ASTNode::number(2.),
                                            ]),
                                            ASTNode::number(3.),
                                        ]),
                                        ASTNode::number(4.),
                                    ]),
                                    ASTNode::number(5.),
                                ]),
                                ASTNode::number(6.),
                            ]),
                            ASTNode::number(7.),
                        ]),
                        ASTNode::number(8.),
                    ]),
                    ASTNode::number(9.),
                ]),
                ASTNode::number(10.),
            ]),
        );
    }

    #[test]
    fn full_egyptian_triangle() {
        // Egyptian triangle
        let x = vec![
            "x = 3",
            "y = 4",
            "z = (x^2 + y^2)^1/2",
        ];
        let x = x.iter()
            .map(|a| tokenize(a).unwrap())
            .map(|a| parse(&a).unwrap())
            .enumerate();
        let mut resolver = Resolver::new();

        let output = resolver.resolve(x.collect());

        // We are indexing from 0, so 3rd is actually 2nd
        assert_eq!(output.last().unwrap(), &(2usize, ResolveMessage::output("z = 5")));
    }

    #[test]
    fn full_fraction_subtraction() {
        let x = vec![
            "x = 0.0002",
            "y = 0.0001",
            "x - y",
        ];
        let x = x.iter()
            .map(|a| tokenize(a).unwrap())
            .map(|a| parse(&a).unwrap())
            .enumerate();
        let mut resolver = Resolver::new();

        let output = resolver.resolve(x.collect());

        assert_eq!(output.last().unwrap(), &(2usize, ResolveMessage::output("? = 0.0001")));
    }

    #[test]
    fn resolve_with_partial_fn_declaration_throws_error() {
        let x = "let f()";
        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(parse(&tokenize(x).unwrap()).unwrap());

        assert_eq!(output.len(), 1);

        assert!(matches!(output.first().unwrap().msg_type, ResolveMessageType::Error));
    }

    #[test]
    fn resolve_with_missing_side_of_equation_throws_error() {
        let x = "10 =";
        let mut resolver = Resolver::new();

        let output = resolver.resolve_line(parse(&tokenize(x).unwrap()).unwrap());

        println!("{:?}", output);

        assert_eq!(output.len(), 2);

        assert!(matches!(output.first().unwrap().msg_type, ResolveMessageType::Error));
        assert!(matches!(output[1].msg_type, ResolveMessageType::Error));
    }

    #[test]
    fn resolve_fn_assignment_with_missing_parts_does_not_panic() {
        let x = "let f(x, y) = x^(y + 1)";
        for i in 0..x.len() {
            let mut resolver = Resolver::new();
            let line = x[0..=i].into();
            if let Ok(tokenized) = tokenize(line) {
                if let Ok(parsed) = parse(&tokenized) {
                    resolver.resolve_line(parsed);
                }
            }
        }
    }

    #[test]
    fn resolve_fn_assignment_with_another_line_does_not_panic() {
        let x = vec![
            "let f(x) = 10",
            "f",
            "f()",
            "f(10)"
        ];
        let mut resolver = Resolver::new();

        for x in x {
            resolver.resolve_line(parse(&tokenize(x).unwrap()).unwrap());
        }
    }
}