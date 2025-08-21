use nix_parser::NixParser;
use std::fs;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple CLI for now - can be enhanced with clap later
    let args: Vec<String> = std::env::args().collect();

    let source = if args.len() > 1 {
        if args[1] == "-" {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        } else {
            fs::read_to_string(&args[1])?
        }
    } else {
        eprintln!("Usage: nix-parse <file.nix>");
        eprintln!("       nix-parse - (read from stdin)");
        std::process::exit(1);
    };

    // Parse
    let mut parser = NixParser::new()?;
    let result = parser.parse(&source)?;

    // Check for errors
    if !result.diagnostics().is_empty() {
        for diag in result.diagnostics() {
            eprintln!("Error: {:?}", diag);
        }
        std::process::exit(1);
    }

    // Get AST and print
    match result.expression()? {
        Some(ast) => println!("{:#?}", ast),
        None => eprintln!("No expression found in parse result"),
    }

    Ok(())
}
