use clap::{Arg, App};

use literate_robot::{parser::parsers::parse, tokenizer::tokenize};

fn main() {
    let matches = App::new("literate-robot")
        .version("0.1")
        .author("drzezga")
        .about("Parses and resolves mathematical expressions.")
        .arg(Arg::new("INPUT")
            .index(1))
        .arg(Arg::new("file")
            .short('f'))
        .subcommand(App::new("latex")
            .about("Generates latex from the mathematical expression.")
            .arg(Arg::new("INPUT")
                .index(1)))
        .get_matches();

    match matches.subcommand() {
        Some(("latex", sub_matches)) => {
            // generate latex
            // let str: String = "F_g=G*(m_1*m_2)/r^2".into();
            let str = sub_matches.value_of("INPUT").unwrap();
            let tokens = tokenize(&str).unwrap();
            let tree = parse(&tokens).unwrap();
            let latex = tree.to_latex();
            println!("{}", latex);
        }
        // no subcommands
        _ => {
            let str = matches.value_of("INPUT").unwrap();
            let tokens = tokenize(&str).unwrap();
            let tree = parse(&tokens).unwrap();
            println!("{:#?}", tree);
            // let latex = tree.to_latex();
        }
    }
}