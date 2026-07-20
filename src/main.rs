use std::path::Path;
use prison::{assembler, decoder, match_runner, tournament};

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
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--rounds" => rounds = iter.next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| die("--rounds requires a positive integer")),
            "--seed" => seed = iter.next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| die("--seed requires a positive integer")),
            _ => positional.push(arg.as_str()),
        }
    }

    if positional.len() != 2 {
        die("usage: prison match <a.asm> <b.asm> [--rounds N] [--seed N]");
    }

    let path_a = positional[0];
    let path_b = positional[1];
    let program_a = compile(path_a);
    let program_b = compile(path_b);

    let result = match_runner::run_match(&program_a, &program_b, rounds, seed)
        .unwrap_or_else(|e| die(&format!("{e:?}")));

    let name_a = Path::new(path_a).file_name().unwrap().to_string_lossy();
    let name_b = Path::new(path_b).file_name().unwrap().to_string_lossy();

    println!("{name_a} vs {name_b} ({rounds} round{}, seed {seed})", if rounds == 1 { "" } else { "s" });
    println!("{name_a:<20} {}", result.total_a);
    println!("{name_b:<20} {}", result.total_b);
}

fn cmd_tournament(args: &[String]) {
    let mut rounds = 1u32;
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let mut positional = Vec::new();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--rounds" => rounds = iter.next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| die("--rounds requires a positive integer")),
            "--seed" => seed = iter.next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| die("--seed requires a positive integer")),
            _ => positional.push(arg.as_str()),
        }
    }

    if positional.len() != 1 {
        die("usage: prison tournament <dir> [--rounds N] [--seed N]");
    }

    let dir = positional[0];
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| die(&format!("{dir}: {e}")));

    let mut strategies = Vec::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| die(&format!("{e}")));
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("asm") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let program = compile(&path.to_string_lossy());
        strategies.push((name, program));
    }

    if strategies.is_empty() {
        die(&format!("{dir}: no .asm files found"));
    }

    strategies.sort_by(|a, b| a.0.cmp(&b.0));

    let result = tournament::run_tournament(&strategies, rounds, seed)
        .unwrap_or_else(|e| die(&format!("{e:?}")));

    let n = strategies.len();
    println!("Tournament: {n} strategies, {rounds} round{} each, seed {seed}",
        if rounds == 1 { "" } else { "s" });
    println!();
    println!("{:<24} {}", "Strategy", "Score");
    println!("{}", "-".repeat(32));
    for (name, score) in &result.standings {
        println!("{name:<24} {score}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("match")      => cmd_match(&args[2..]),
        Some("tournament") => cmd_tournament(&args[2..]),
        _ => die("usage: prison <match|tournament> ..."),
    }
}
