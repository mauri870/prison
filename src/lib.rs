pub mod assembler;
pub mod decoder;
pub mod match_runner;
pub mod tournament;
pub mod vm;

pub fn run(src: &str, seed: u64) -> Result<vm::Action, String> {
    let bytes = assembler::assemble(src)?;
    let program = decoder::decode(&bytes).map_err(|e| format!("{e:?}"))?;
    let obs = vm::Observation::default();
    vm::Vm::new(seed).run_round(&program, &obs).map_err(|e| format!("{e:?}"))
}
