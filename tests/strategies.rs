use prison::vm::Action;

fn run_strategy(path: &std::path::Path) -> datatest_stable::Result<()> {
    let src = std::fs::read_to_string(path)?;
    let expects = parse_expects(&src);
    if expects.checks.is_empty() {
        return Ok(());
    }

    let action = prison::run(&src, expects.seed).expect("run failed");
    let file = path.display().to_string();

    for (key, val) in &expects.checks {
        check(key, val, &action, &file);
    }

    Ok(())
}

struct Expects {
    seed: u64,
    checks: Vec<(String, String)>,
}

// Parse `; Expect: KEY=VALUE ...` lines from assembly source.
// The `seed` key sets the VM seed; all other keys become assertions.
fn parse_expects(src: &str) -> Expects {
    let mut seed = 1u64;
    let mut checks = Vec::new();

    for line in src.lines() {
        if let Some(rest) = line.trim().strip_prefix("; Expect:") {
            for token in rest.split_whitespace() {
                if let Some((key, val)) = token.split_once('=') {
                    if key == "seed" {
                        seed = val.parse().unwrap_or_else(|_| panic!("bad seed: {val}"));
                    } else {
                        checks.push((key.to_string(), val.to_string()));
                    }
                }
            }
        }
    }

    Expects { seed, checks }
}

fn check(key: &str, val: &str, action: &Action, file: &str) {
    match key {
        "action" => {
            let expected = match val {
                "Cooperate" => Action::Cooperate,
                "Defect" => Action::Defect,
                _ => panic!("{file}: unknown action value '{val}'"),
            };
            assert_eq!(*action, expected, "{file}: action = {action:?}, expected {expected:?}");
        }
        _ => panic!("{file}: unknown expect key '{key}'"),
    }
}

datatest_stable::harness!(run_strategy, "strategies", r".*\.asm$");
