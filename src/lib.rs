pub mod assembler;
pub mod decoder;
pub mod match_runner;
pub mod vm;

pub fn run(src: &str, seed: u64) -> Result<vm::Action, String> {
    let bytes = assembler::assemble(src)?;
    let program = decoder::decode(&bytes).map_err(|e| format!("{e:?}"))?;
    let obs = vm::Observation::default();
    let mut mem = vm::StrategyMemory::new();
    vm::Vm::new(seed).run_round(&program, &obs, &mut mem).map_err(|e| format!("{e:?}"))
}
