# prison

An iterated prisoner's dilemma (IPD) tournament implementation inspired by Robert Axelrod's The Evolution of Cooperation.

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

Read-only pseudo-registers expose game state via `LOAD Rd, <name>`. See `isa.asm` for the full list and semantics.

Payoff matrix: CC=(3,3), CD=(0,5), DC=(5,0), DD=(1,1).

Example [tit_for_tat.asm](strategies/tit_for_tat.asm):

```asm
; Cooperate on round 0, then mirror the opponent's last move.
    LOAD R0, LASTOPP
    CMP R0, DEFECT
    JEQ defect
    PLAY COOPERATE

defect:
    PLAY DEFECT

```

Read-only pseudo-registers expose game state:

| Name            | Value                          |
|-----------------|--------------------------------|
| `LASTOPP`       | Opponent's last action (0=C, 1=D) |
| `LASTSELF`      | Your last action               |
| `ROUND`         | Current round number           |
| `SCORESELF`     | Your cumulative score          |
| `SCOREOPP`      | Opponent's cumulative score    |
| `LASTPAYOFFSELF`| Your payoff last round         |
| `OPPID`         | Opponent index in tournament   |

The full instruction set with per-instruction documentation is in `isa.asm`.

## Testing strategies

Add `; Expect: action=Cooperate` (or `Defect`) to a strategy file and it will be picked up automatically by the test harness. Use `seed=N` to fix the RNG for strategies that use `RDRAND`.

```
cargo test
```
