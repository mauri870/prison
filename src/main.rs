mod assembler;
mod decoder;
mod vm;

fn run(src: &str) -> Result<vm::Action, String> {
    let bytes = assembler::assemble(src)?;
    let program = decoder::decode(&bytes).map_err(|e| format!("{e:?}"))?;
    vm::Vm::new().run_round(&program).map_err(|e| format!("{e:?}"))
}

fn main() {
    println!("coop:   {:?}", run(include_str!("../strategies/coop.asm")).unwrap());
    println!("defect: {:?}", run(include_str!("../strategies/defect.asm")).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coop_always_cooperates() {
        assert_eq!(run(include_str!("../strategies/coop.asm")).unwrap(), vm::Action::Cooperate);
    }

    #[test]
    fn defect_always_defects() {
        assert_eq!(run(include_str!("../strategies/defect.asm")).unwrap(), vm::Action::Defect);
    }
}
