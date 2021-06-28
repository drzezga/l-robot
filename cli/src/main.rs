use std::fs;

use l_robot::{parser::parsers::parse, resolve_lines, resolver::{ResolveMessage, ResolveMessageType}, tokenizer::tokenize};

use clap::{Arg, App};
use colored::Colorize;

fn main() {
    let matches = App::new("literate-robot")
        .version("0.1")
        .author("drzezga")
        .about("Parses and resolves mathematical expressions.")
        .arg(Arg::new("file")
            .takes_value(true)
            .short('f'))
        .arg(Arg::new("INPUT")
            .index(1))
        .subcommand(App::new("latex")
            .about("Generates latex from the mathematical expression.")
            .arg(Arg::new("INPUT")
                .index(1)))
        .subcommand(App::new("debug")
            .about("Debugs.")
            .arg(Arg::new("INPUT")
                .index(1)))
        .get_matches();

    match matches.subcommand() {
        Some(("debug", sub_matches)) => {
            // generate latex
            // let str: String = "F_g=G*(m_1*m_2)/r^2".into();
            let str = sub_matches.value_of("INPUT").unwrap();
            let tokens = tokenize(&str).unwrap();
            let tree = parse(&tokens).unwrap();
            // let latex = tree.to_latex();
            println!("{:#?}", tree);
        }
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
            let output = if let Some(filename) = matches.value_of("file") {
                if let Ok(content) = fs::read_to_string(filename) {
                    resolve_lines(content.lines().map(|x| x.to_string()).collect())
                } else {
                    panic!("Could not load file");
                }
            } else {
                let str = matches.value_of("INPUT").unwrap();
                resolve_lines(str.split(';').map(|x| x.to_string()).collect())
                // let tokens = tokenize(&str).unwrap();
                // let tree = parse(&tokens).unwrap();
                // let mut resolver = Resolver::new();
                // let output = resolver.resolve(vec![(1, tree)]);
            };

            for (line_num, ResolveMessage { msg_type, content: message}) in output {
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
        }
    }
}
