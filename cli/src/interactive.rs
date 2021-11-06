use std::io::{BufRead, Write};

use colored::Colorize;

use l_robot::git_hash;
use l_robot::tokenizer::tokenize;
use l_robot::parser::parsers::parse;
use l_robot::resolver::{Resolver, resolve_message::ResolveMessageType};

pub fn start_interactive() {
    println!("{} {}", "l-robot".blue(), git_hash().bright_black());
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
                    ResolveMessageType::Error => println!("{} : {}", "err".red(), message.content),
                    ResolveMessageType::Info => println!("{} : {}", "inf".bright_blue(), message.content),
                    ResolveMessageType::Output => println!("{} : {}", "out".bright_black(), message.content.bright_black()),
                }
            }
        } else {
            println!("{} : {}", "err".red(), "Input error");
        }

        print!("{}", "you > ");
        std::io::stdout().flush().unwrap();
    }
}