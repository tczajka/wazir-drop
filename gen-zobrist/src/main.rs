use rand::{Rng, SeedableRng, rngs::StdRng};
use wazir_drop::{Color, NUM_CAPTURED_INDEXES, Piece, Square};

fn main() {
    let mut rng = StdRng::from_os_rng();

    println!("#[rustfmt::skip]");
    println!("pub static TO_MOVE: EnumMap<Color, u64> = EnumMap::from_array([");
    generate("    ", Color::COUNT, &mut rng);
    println!("]);");
    println!();

    println!("#[rustfmt::skip]");
    println!(
        "pub static PIECE_SQUARE: EnumMap<Piece, EnumMap<Square, u64>> = EnumMap::from_array(["
    );
    for _ in 0..Piece::COUNT {
        println!("    EnumMap::from_array([");
        generate("        ", Square::COUNT, &mut rng);
        println!("    ]),");
    }
    println!("]);");
    println!();

    println!("#[rustfmt::skip]");
    println!("static CAPTURED: [u64; NUM_CAPTURED_INDEXES] = [");
    generate("    ", NUM_CAPTURED_INDEXES, &mut rng);
    println!("];");
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
