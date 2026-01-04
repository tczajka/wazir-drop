use std::str::FromStr;

use wazir_drop::{
    movegen::{
        any_move_from_short_move, attacked_by, captures, captures_boring, captures_check_threats,
        captures_checks, captures_of_wazir, check_evasions_capture_attacker, double_move_bitboard,
        drops, drops_boring, drops_check_threats, drops_checks, in_check, jumps, jumps_boring,
        jumps_check_threats, jumps_checks, move_bitboard, pseudocaptures, pseudojumps, setup_moves,
        triple_move_bitboard, validate_from_to,
    },
    Color, Move, Piece, Position, ShortMove, Square,
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
fn test_double_move_bitboard() {
    // From a corner.
    assert_eq!(
        double_move_bitboard(Piece::Knight, Square::E4).to_string(),
        "\
.x.x.x..
x.x.x.x.
...x...x
x.x.x.x.
.x.x.x.x
x.x.x.x.
...x...x
x.x.x.x.
"
    );
}

#[test]
fn test_triple_move_bitboard() {
    // From a corner.
    assert_eq!(
        triple_move_bitboard(Piece::Ferz, Square::E4).to_string(),
        "\
........
x.x.x.x.
........
x.x.x.x.
........
x.x.x.x.
........
x.x.x.x.
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
    let mov = any_move_from_short_move(&position, ShortMove::from_str("AWNAADADAFFAADDA").unwrap())
        .unwrap();
    assert_eq!(mov.to_string(), "AWNAADADAFFAADDA");

    assert!(
        any_move_from_short_move(&position, ShortMove::from_str("AWNAADADAFFAADDN").unwrap())
            .is_err()
    );
    assert!(
        any_move_from_short_move(&position, ShortMove::from_str("awnaadadaffaadda").unwrap())
            .is_err()
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

    let mov = any_move_from_short_move(&position, ShortMove::from_str("Aa1").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "A@a1");
    let mov = any_move_from_short_move(&position, ShortMove::from_str("a2a3").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "Wa2-a3");
    let mov = any_move_from_short_move(&position, ShortMove::from_str("a2b2").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "Wa2xab2");

    assert!(
        any_move_from_short_move(&position, ShortMove::from_str("AWNAADADAFFAADDA").unwrap())
            .is_err()
    );
    assert!(any_move_from_short_move(&position, ShortMove::from_str("fa1").unwrap()).is_err());
    assert!(any_move_from_short_move(&position, ShortMove::from_str("a2c2").unwrap()).is_err());
    assert!(any_move_from_short_move(&position, ShortMove::from_str("b3a4").unwrap()).is_err());
    assert!(any_move_from_short_move(&position, ShortMove::from_str("Na1").unwrap()).is_err());
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
AF
.W.A.D.D
AfFA.DAD
f.A.A.A.
....d.A.
...a.Nad
.....n..
a.a...a.
add.w..a
",
    )
    .unwrap();

    let moves: Vec<String> = pseudocaptures(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["Ab7xdd5", "Ac5xae7", "Ne6xag7", "Wa2xfb2"]);

    let moves: Vec<String> = captures(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(&moves, &["Ab7xdd5", "Ac5xae7", "Ne6xag7"]);

    let moves: Vec<String> = captures_checks(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["Ne6xag7"]);

    let moves: Vec<String> = captures_check_threats(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["Ab7xdd5"]);

    let moves: Vec<String> = captures_boring(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["Ac5xae7"]);
}

#[test]
fn test_captures_of_wazir() {
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

    let moves: Vec<String> = captures_of_wazir(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["Ab4xwd6", "Ab8xwd6", "Db6xwd6", "Nf7xwd6"]);
}

#[test]
fn test_check_evasion_capture_attacker() {
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

    let moves: Vec<String> = check_evasions_capture_attacker(&position)
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

    let moves: Vec<Move> = check_evasions_capture_attacker(&position).collect();
    assert!(moves.is_empty());
}

#[test]
fn test_jumps() {
    let position = Position::from_str(
        "\
regular
4
AAAAAAAAddFf
.W...D..
..Ff..D.
......A.
........
...a..ad
..d..nN.
a.a...a.
add...wa
",
    )
    .unwrap();

    let moves: Vec<String> = pseudojumps(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(
        &moves,
        &[
            "Ac7-a5", "Ac7-e5", "Da6-a4", "Da6-a8", "Da6-c6", "Db7-b5", "Db7-d7", "Fb3-a4",
            "Fb3-c2", "Fb3-c4", "Nf7-d6", "Nf7-d8", "Nf7-e5", "Nf7-g5", "Nf7-h6", "Wa2-a1",
            "Wa2-a3", "Wa2-b2",
        ]
    );

    let moves: Vec<String> = jumps(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(
        &moves,
        &[
            "Ac7-a5", "Ac7-e5", "Da6-a4", "Da6-a8", "Da6-c6", "Db7-b5", "Db7-d7", "Fb3-a4",
            "Fb3-c2", "Fb3-c4", "Nf7-d6", "Nf7-d8", "Nf7-e5", "Nf7-g5", "Nf7-h6", "Wa2-a1",
            "Wa2-b2",
        ]
    );

    let moves: Vec<String> = jumps_checks(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(&moves, &["Nf7-g5"]);

    let moves: Vec<String> = jumps_check_threats(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["Db7-d7"]);

    let moves: Vec<String> = jumps_boring(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(
        &moves,
        &[
            "Ac7-a5", "Ac7-e5", "Da6-a4", "Da6-a8", "Da6-c6", "Db7-b5", "Fb3-a4", "Fb3-c2",
            "Fb3-c4", "Nf7-d6", "Nf7-d8", "Nf7-e5", "Nf7-h6", "Wa2-a1", "Wa2-b2",
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
add..w.a
",
    )
    .unwrap();

    let moves: Vec<String> = drops(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(
        &moves,
        &[
            "A@a3", "A@a5", "A@a7", "A@b5", "A@c1", "A@c2", "A@c4", "A@c6", "A@c8", "A@d1", "A@d2",
            "A@d3", "A@d4", "A@d5", "A@d6", "A@d8", "A@e1", "A@e2", "A@e3", "A@e5", "A@e6", "A@f1",
            "A@f2", "A@f4", "A@f5", "A@f8", "A@g2", "A@g4", "A@g5", "A@g6", "A@g8", "A@h4", "A@h5",
            "A@h7",
        ]
    );

    let moves: Vec<String> = drops_checks(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(&moves, &["A@f4", "A@f8"]);

    let moves: Vec<String> = drops_check_threats(&position)
        .map(|mov| mov.to_string())
        .collect();
    assert_eq!(&moves, &["A@d2", "A@d6"]);

    let moves: Vec<String> = drops_boring(&position).map(|mov| mov.to_string()).collect();
    assert_eq!(
        &moves,
        &[
            "A@a3", "A@a5", "A@a7", "A@b5", "A@c1", "A@c2", "A@c4", "A@c6", "A@c8", "A@d1", "A@d3",
            "A@d4", "A@d5", "A@d8", "A@e1", "A@e2", "A@e3", "A@e5", "A@e6", "A@f1", "A@f2", "A@f5",
            "A@g2", "A@g4", "A@g5", "A@g6", "A@g8", "A@h4", "A@h5", "A@h7",
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
