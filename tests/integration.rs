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
        )
    }
}