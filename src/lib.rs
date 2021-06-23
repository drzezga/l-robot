pub mod tokenizer;
pub mod parser;
pub mod latex;
pub mod resolver;

pub fn resolve_lines(lines: Vec<String>) -> Vec<(usize, resolver::ResolveMessage)> {
    let roots = lines
        .iter().enumerate()
        .map(|(line_num, line)| (line_num + 1, tokenizer::tokenize(line).unwrap()))
        .filter(|(_, tokens)| tokens.len() != 0) // TODO: better way of resolving errors
        .map(|(line_num, tokens)| (line_num, parser::parsers::parse(&tokens).unwrap()))
        .collect();
    // for root in &roots {
    //     println!("{:?}", root);
    // }
    // println!("{}", roots);
    let mut resolver = resolver::Resolver::new();
    resolver.resolve(roots)
}