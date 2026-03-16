use std::{
    io::{self, BufRead},
    process::exit,
};

use abacat_eval::{document::Document, eval::Eval};
use abacat_parser::parse;

fn main() -> io::Result<()> {
    ctrlc::set_handler(|| {
        println!("BYE BYE (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧");
        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let mut doc = Document::new_with_default_constants();
    let mut expr_buffer = String::with_capacity(2048);
    println!("HULLO!!!");
    loop {
        let mut stdin = io::stdin().lock();
        stdin.read_line(&mut expr_buffer)?;
        let parsed = parse(&expr_buffer);
        if let Result::Ok(expr) = parsed {
            match doc.next(expr) {
                Ok(Eval::Value(value)) => println!("> {}", value),
                Ok(Eval::ValueAssignment(ident, _)) => println!("> Assigned '{}'", ident),
                Err(_) => println!("> Failed to eval the statement"),
            }
        } else {
            println!("Could not parse!");
        }
        expr_buffer.clear();
    }
}
