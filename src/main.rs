fn main() {
    let seed = 1;
    println!("coop:   {:?}", prison::run(include_str!("../strategies/coop.asm"), seed).unwrap());
    println!("defect: {:?}", prison::run(include_str!("../strategies/defect.asm"), seed).unwrap());
    println!("random: {:?}", prison::run(include_str!("../strategies/random.asm"), seed).unwrap());
}
