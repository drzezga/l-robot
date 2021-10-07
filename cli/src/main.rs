use std::fs;
use std::io::{BufRead, Write};

use l_robot::{parser::parsers::parse, resolve_lines, tokenizer::tokenize};

use l_robot::resolver::{resolve_message::{ResolveMessage, ResolveMessageType}, Resolver};

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
        .subcommand(App::new("interactive")
            .about("Opens an interactive shell."))
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
        Some(("interactive", _)) => {
            println!("{}", "l-robot interactive mode.".blue());
            println!("Press ctrl + C to exit.\n");
            print!("{}", "you > ");
            std::io::stdout().flush().unwrap();

            let mut resolver = Resolver::new();
            
            for line in std::io::stdin().lock().lines() {
                if let Ok(str) = line {
                    let tokens = tokenize(&str).unwrap();
                    let tree = parse(&tokens).unwrap();
                    let output = resolver.resolve_line(tree);
                    for message in output {
                        match message.msg_type {
                            ResolveMessageType::Error => println!("err : {}", message.content.red()),
                            ResolveMessageType::Info => println!("inf : {}", message.content.bright_blue()),
                            ResolveMessageType::Output => println!("out : {}", message.content.bright_black()),
                        }
                    }
                } else {
                    println!("err : {}", "Input error".red());
                }

                print!("{}", "you > ");
                std::io::stdout().flush().unwrap();
            }
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
