use rand::{Rng, SeedableRng, rngs::StdRng};
use wazir_drop::{
    Color, ColoredPiece, NUM_CAPTURED_INDEXES, Square,
    constants::{PLY_AFTER_SETUP, PLY_DRAW},
};

fn main() {
    let mut rng = StdRng::from_os_rng();

    println!("#[rustfmt::skip]");
    println!("pub static TO_MOVE: EnumMap<Color, u64> = EnumMap::from_array([");
    generate("    ", Color::COUNT, &mut rng);
    println!("]);");
    println!();

    println!("#[rustfmt::skip]");
    println!(
        "pub static COLORED_PIECE_SQUARE: EnumMap<ColoredPiece, EnumMap<Square, u64>> = EnumMap::from_array(["
    );
    for _ in 0..ColoredPiece::COUNT {
        println!("    EnumMap::from_array([");
        generate("        ", Square::COUNT, &mut rng);
        println!("    ]),");
    }
    println!("]);");
    println!();

    println!("#[rustfmt::skip]");
    println!(
        "static CAPTURED: EnumMap<Color, [u64; NUM_CAPTURED_INDEXES]> = EnumMap::from_array(["
    );
    for _ in 0..Color::COUNT {
        println!("    [");
        generate("        ", NUM_CAPTURED_INDEXES, &mut rng);
        println!("    ],");
    }
    println!("]);");
    println!();

    println!("#[rustfmt::skip]");
    println!("pub static NULL_MOVE_COUNTER: [u64; (PLY_DRAW - PLY_AFTER_SETUP + 1) as usize] = [");
    generate(
        "    ",
        usize::from(PLY_DRAW - PLY_AFTER_SETUP + 1),
        &mut rng,
    );
    println!("];");
    println!();

    println!("#[rustfmt::skip]");
    println!("pub static PLY: [u64; (PLY_DRAW + 1) as usize] = [");
    generate("    ", usize::from(PLY_DRAW + 1), &mut rng);
    println!("];");
    println!();
}

fn generate(indent: &str, count: usize, rng: &mut StdRng) {
    print!("{indent}");
    for i in 0..count {
        let x: u64 = rng.random();
        print!("0x{x:016x},");
        if i == count - 1 {
            println!();
        } else if i % 5 == 4 {
            println!();
            print!("{indent}");
        } else {
            print!(" ");
        }
    }
}
