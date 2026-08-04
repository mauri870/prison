# prison

An iterated prisoner's dilemma (IPD) tournament implementation inspired by Robert Axelrod's [The Evolution of Cooperation](https://ee.stanford.edu/~hellman/Breakthrough/book/pdfs/axelrod.pdf).

Strategies are small agents written in a custom assembly language and run on a purpose-built virtual machine.

## Usage

```
prison match <a.asm> <b.asm> [--rounds N] [--seed N]
prison tournament <dir> [--rounds N] [--seed N]
```

`match` runs two strategies head-to-head. `tournament` runs every strategy in a directory against every other in a round-robin and prints a scored leaderboard. Both commands use a random seed by default; pass `--seed` to make a run reproducible.

## Writing a strategy

Strategies are assembly files targeting the prison ISA. Each round the VM executes the program from the top and expects it to end with `PLAY COOPERATE` or `PLAY DEFECT`. If the program halts or exhausts its instruction budget without a play, it cooperates.

Four registers (`R0`-`R3`), 64 bytes of per-match scratch memory.

Read-only pseudo-registers expose game state via `LOAD Rd, <name>`.

Payoff matrix: CC=(3,3), CD=(0,5), DC=(5,0), DD=(1,1).

Example [joss.asm](strategies/joss.asm):

```asm
; Like tit-for-tat but randomly defects ~10% of the time even when opponent cooperated.
    LOAD R0, LASTOPP
    CMP R0, DEFECT
    JEQ defect
    RDRAND R0
    CMPI R0, #102
    JGT defect
    PLAY COOPERATE

defect:
    PLAY DEFECT
```

The full instruction set with per-instruction documentation is in [`isa.asm`](isa.asm).

## Testing strategies

Add `; Expect: action=Cooperate` (or `Defect`) to a strategy file and it will be picked up automatically by the test harness. Use `seed=N` to fix the RNG for strategies that use `RDRAND`.

```
cargo test
```

## Citing this project

```bibtex
@software{meneguzzo2026prison,
  author  = {{de Souza Meneguzzo}, Mauri},
  title   = {prison},
  year    = {2026},
  url     = {https://github.com/mauri870/prison}
}
```
