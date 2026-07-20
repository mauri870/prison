use std::path::Path;
use prison::{assembler, decoder, match_runner};

fn compile(path: &str) -> Vec<prison::vm::Opcode> {
    let src = std::fs::read_to_string(path)
        .unwrap_or_else(|e| die(&format!("{path}: {e}")));
    let bytes = assembler::assemble(&src)
        .unwrap_or_else(|e| die(&format!("{path}: {e}")));
    decoder::decode(&bytes)
        .unwrap_or_else(|e| die(&format!("{path}: {e:?}")))
}

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1);
}

fn cmd_match(args: &[String]) {
    let mut positional = Vec::new();
    let mut rounds = 1u32;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        if arg == "--rounds" {
            rounds = iter.next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| die("--rounds requires a positive integer"));
        } else {
            positional.push(arg.as_str());
        }
    }

    if positional.len() != 2 {
        die("usage: prison match <a.asm> <b.asm> [--rounds N]");
    }

    let path_a = positional[0];
    let path_b = positional[1];
    let program_a = compile(path_a);
    let program_b = compile(path_b);

    let result = match_runner::run_match(&program_a, &program_b, rounds, 1)
        .unwrap_or_else(|e| die(&format!("{e:?}")));

    let name_a = Path::new(path_a).file_name().unwrap().to_string_lossy();
    let name_b = Path::new(path_b).file_name().unwrap().to_string_lossy();

    println!("{name_a} vs {name_b} ({rounds} round{})", if rounds == 1 { "" } else { "s" });
    println!("{name_a:<20} {}", result.total_a);
    println!("{name_b:<20} {}", result.total_b);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("match") => cmd_match(&args[2..]),
        _ => die("usage: prison match <a.asm> <b.asm> [--rounds N]"),
    }
}
