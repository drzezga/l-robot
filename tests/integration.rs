mod tests {
    use literate_robot::{parser::{ASTNode, ASTNodeType, parsers::parse}, tokenizer::{Token, tokenize}};

    #[test]
    fn tokenize_parse() {
        let x = tokenize("x^2").unwrap();
        let x = parse(&x).unwrap();
        assert_eq!(
            x,
            ASTNode::new(ASTNodeType::Power, vec![
                ASTNode::delimeter(Token::Name("x".into())),
                ASTNode::delimeter(Token::Number(2.0)),
            ])
        );

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
}