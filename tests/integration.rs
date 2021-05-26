mod tests {
    use literate_robot::{parser::{ASTNode, ASTNodeType, parsers::parse}, tokenizer::{Token, tokenize}};

    #[test]
    fn tokenize_parse() {
        let x = tokenize("x^2");
        let x = parse(&x);
        // println!("{:#?}", &x);
        assert_eq!(
            x.children,
            vec![
                ASTNode::new(ASTNodeType::Power, vec![
                    ASTNode::delimeter(Token::Name("x".into())),
                    ASTNode::delimeter(Token::Number(2.0)),
                ])
            ]
        )
    }
}