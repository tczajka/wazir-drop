use std::str::FromStr;

use wazir_drop::{
    movegen::{
        attacked_by, captures, check_evasion_captures, check_evasion_jumps, drops, in_check,
        move_bitboard, move_from_short_move, pseudojumps, setup_moves, validate_from_to,
        wazir_captures,
    },
    Color, Piece, Position, RegularMove, ShortMove, Square,
};

#[test]
fn test_move_bitboard() {
    assert_eq!(
        move_bitboard(Piece::Alfil, Square::D4).to_string(),
        "\
........
.x...x..
........
........
........
.x...x..
........
........
"
    );

    assert_eq!(
        move_bitboard(Piece::Dabbaba, Square::D4).to_string(),
        "\
........
...x....
........
.x...x..
........
...x....
........
........
"
    );

    assert_eq!(
        move_bitboard(Piece::Ferz, Square::D4).to_string(),
        "\
........
........
..x.x...
........
..x.x...
........
........
........
"
    );

    assert_eq!(
        move_bitboard(Piece::Knight, Square::D4).to_string(),
        "\
........
..x.x...
.x...x..
........
.x...x..
..x.x...
........
........
"
    );

    assert_eq!(
        move_bitboard(Piece::Wazir, Square::D4).to_string(),
        "\
........
........
...x....
..x.x...
...x....
........
........
........
"
    );

    // From a corner.
    assert_eq!(
        move_bitboard(Piece::Knight, Square::H8).to_string(),
        "\
........
........
........
........
........
......x.
.....x..
........
"
    );
}

#[test]
fn test_validate_from_to() {
    assert!(validate_from_to(Piece::Alfil, Square::D4, Square::F6).is_ok());
    assert!(validate_from_to(Piece::Alfil, Square::D4, Square::D5).is_err());
    assert!(validate_from_to(Piece::Knight, Square::A4, Square::B2).is_ok());
    assert!(validate_from_to(Piece::Knight, Square::A4, Square::C6).is_err());
}

#[test]
fn test_move_from_short_move() {
    let position = Position::from_str(
        "\
setup
0

........
........
........
........
........
........
........
........
",
    )
    .unwrap();
    let mov =
        move_from_short_move(&position, ShortMove::from_str("AWNAADADAFFAADDA").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "AWNAADADAFFAADDA");

    assert!(
        move_from_short_move(&position, ShortMove::from_str("AWNAADADAFFAADDN").unwrap()).is_err()
    );
    assert!(
        move_from_short_move(&position, ShortMove::from_str("awnaadadaffaadda").unwrap()).is_err()
    );

    let position = Position::from_str(
        "\
regular
4
AFf
.W.A.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
",
    )
    .unwrap();

    let mov = move_from_short_move(&position, ShortMove::from_str("Aa1").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "A@a1");
    let mov = move_from_short_move(&position, ShortMove::from_str("a2a3").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "Wa2-a3");
    let mov = move_from_short_move(&position, ShortMove::from_str("a2b2").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "Wa2xab2");

    assert!(
        move_from_short_move(&position, ShortMove::from_str("AWNAADADAFFAADDA").unwrap()).is_err()
    );
    assert!(move_from_short_move(&position, ShortMove::from_str("fa1").unwrap()).is_err());
    assert!(move_from_short_move(&position, ShortMove::from_str("a2c2").unwrap()).is_err());
    assert!(move_from_short_move(&position, ShortMove::from_str("b3a4").unwrap()).is_err());
    assert!(move_from_short_move(&position, ShortMove::from_str("Na1").unwrap()).is_err());
}

#[test]
fn test_setup_moves() {
    let mut count: u32 = 0;
    for mov in setup_moves(Color::Red) {
        mov.validate_pieces().unwrap();
        count += 1;
    }
    // 16! / (8! 4! 2! 1! 1!) = 10810800
    assert_eq!(count, 10810800);
}

#[test]
fn test_captures() {
    let position = Position::from_str(
        "\
regular
4
AFf
.W.A.D.D
AfFA.DDA
..A.A.A.
......A.
...a..ad
..d..nN.
a.a...a.
add.w..a
",
    )
    .unwrap();

    let moves: Vec<String> = captures(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(&moves, &["Ac5xae7", "Nf7xah8", "Wa2xfb2"]);
}

#[test]
fn test_pseudojumps() {
    let position = Position::from_str(
        "\
regular
4
AAAAAAAAddFf
.W...DD.
.fF.....
......A.
........
...a..ad
..d..nN.
a.a...a.
add.w..a
",
    )
    .unwrap();

    let moves: Vec<String> = pseudojumps(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(
        &moves,
        &[
            "Ac7-a5", "Ac7-e5", "Da6-a4", "Da6-a8", "Da6-c6", "Da7-a5", "Fb3-a4", "Fb3-c2",
            "Fb3-c4", "Nf7-d6", "Nf7-d8", "Nf7-e5", "Nf7-g5", "Nf7-h6", "Wa2-a1", "Wa2-a3"
        ]
    );
}

#[test]
fn test_drops() {
    let position = Position::from_str(
        "\
regular
4
Af
FW.A.D.D
AfFA.DDA
..A.A.A.
......A.
...a..ad
..d..nN.
a.a...a.
add.w..a
",
    )
    .unwrap();

    let moves: Vec<String> = drops(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(
        &moves,
        &[
            "A@a3", "A@a5", "A@a7", "A@b5", "A@c1", "A@c2", "A@c4", "A@c6", "A@c8", "A@d1", "A@d2",
            "A@d3", "A@d4", "A@d5", "A@d6", "A@d8", "A@e1", "A@e2", "A@e3", "A@e5", "A@e6", "A@f1",
            "A@f2", "A@f4", "A@f5", "A@f8", "A@g2", "A@g4", "A@g5", "A@g6", "A@g8", "A@h4", "A@h6",
            "A@h7",
        ]
    );
}

#[test]
fn test_attacked_by() {
    let position = Position::from_str(
        "\
regular
4
Af
FW.A.D.D
AfFA.DDA
..A.A.A.
......A.
...a..ad
..d..nN.
a.a...a.
add.w..a
",
    )
    .unwrap();

    assert_eq!(
        attacked_by(&position, Square::D6, Color::Red).to_string(),
        "\
........
...x.x.x
........
........
........
......x.
........
........
"
    );
}

#[test]
fn test_in_check() {
    let position = Position::from_str(
        "\
regular
4
Af
FW.A.D.D
fAFA.DDA
..A.A.A.
......A.
...a..ad
..d..nN.
a.a...a.
add.w..a
",
    )
    .unwrap();

    assert!(in_check(&position, Color::Red));
    assert!(!in_check(&position, Color::Blue));
}

#[test]
fn test_wazir_captures() {
    let position = Position::from_str(
        "\
regular
4
Af
FW.A.D.D
AfFA.DDA
..A.A.A.
.....wA.
...a..ad
..d..nN.
a.a...a.
add....a
",
    )
    .unwrap();

    let moves: Vec<String> = wazir_captures(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["Ab4xwd6", "Ab8xwd6", "Db6xwd6", "Nf7xwd6"]);
}

#[test]
fn test_check_evasion_captures() {
    let position = Position::from_str(
        "\
regular
4
Af
FW.A.D.D
AFfAD.DA
..A...A.
....A.A.
...a..ad
..d..nN.
a.a...a.
addw...a
",
    )
    .unwrap();

    let moves: Vec<String> = check_evasion_captures(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["Ad5xfb3", "Db5xfb3"]);

    // Double check.
    let position = Position::from_str(
        "\
regular
4
Af
FW.A.D.D
AFfAD.DA
.dA...A.
....A.A.
...a..ad
..d..nN.
a.a...a.
a.dw...a
",
    )
    .unwrap();

    let moves: Vec<RegularMove> = check_evasion_captures(&position).collect();
    assert!(moves.is_empty());
}

#[test]
fn test_check_evasion_jumps() {
    let position = Position::from_str(
        "\
regular
4
Af
FW.A.D.D
AFfAD.DA
..A...A.
....A.A.
...a..ad
..d..nN.
a.a...a.
addw...a
",
    )
    .unwrap();

    let moves: Vec<String> = check_evasion_jumps(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["Wa2-a3"]);
}
