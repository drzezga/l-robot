use clap::{Arg, App};

use colored::Colorize;
use l_robot::{parser::parsers::parse, resolver::{ResolveMessageType, Resolver}, tokenizer::tokenize};

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
        // no subcommands or unknown
        _ => {
            let str = matches.value_of("INPUT").unwrap();
            let tokens = tokenize(&str).unwrap();
            let tree = parse(&tokens).unwrap();
            // println!("{:#?}", tree);
            let mut resolver = Resolver::new();
            let output = resolver.resolve(vec![(1, tree)]);

            for (line_num, (msg_type, message)) in output {
                let message = match msg_type {
                    ResolveMessageType::Error => message.red(),
                    ResolveMessageType::Info => message.blue(),
                    ResolveMessageType::Output => message.white(),
                };
                println!(
                    "{} {}: {}",
                    line_num,
                    match msg_type {
                        ResolveMessageType::Error => "err",
                        ResolveMessageType::Info => "inf",
                        ResolveMessageType::Output => "out",
                    },
                    message
                );
            }
            // let latex = tree.to_latex();
        }
    }
}