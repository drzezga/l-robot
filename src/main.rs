use literate_robot::{parser::parsers::parse, tokenizer::tokenize};

fn main() {
    // println!("guacamole");
    // 
    let str: String = "F_g=G*(m_1*m_2)/r^2".into();
    let tokens = tokenize(&str);
    let tree = parse(&tokens);
    let latex = tree.to_latex();
    println!("{}", latex);
}