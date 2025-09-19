use std::str::FromStr;
use wazir_drop::{ParseError, Piece, Position, RegularMove, Square};

#[test]
fn test_display_from_str() {
    let s = "\
regular
red
AF
f
.W.A.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
";

    let position = Position::from_str(&s).unwrap();
    assert_eq!(position.to_string(), s);

    // Too many ferzes.
    let s = "\
regular
red
AFF
f
.W.A.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
";

    assert_eq!(Position::from_str(&s), Err(ParseError));

    // Too few ferzes.
    let s = "\
regular
red
A
f
.W.A.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
";
    assert_eq!(Position::from_str(&s), Err(ParseError));

    // Too many lines ferzes.
    let s = "\
regular
red
A
f
.W.A.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
........
";
    assert_eq!(Position::from_str(&s), Err(ParseError));
}

#[test]
fn test_parse_regular_move() {
    let position = Position::from_str(
        "\
regular
red
AF
f
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

    // Jump.
    let mov = position.parse_regular_move("a2a3").unwrap();
    assert_eq!(
        mov,
        RegularMove {
            piece: Piece::Wazir,
            captured: None,
            from: Some(Square::A2),
            to: Square::A3,
        }
    );
    assert_eq!(position.colored_regular_move(mov).to_string(), "a2a3");

    // Capture.
    let mov = position.parse_regular_move("a2b2").unwrap();
    assert_eq!(
        mov,
        RegularMove {
            piece: Piece::Wazir,
            captured: Some(Piece::Alfil),
            from: Some(Square::A2),
            to: Square::B2,
        }
    );

    // Can't jump with non-existent piece.
    assert_eq!(position.parse_regular_move("a1a2"), Err(ParseError));
    // Can't capture own piece.
    assert_eq!(position.parse_regular_move("a6a4"), Err(ParseError));

    // Drop.
    let mov = position.parse_regular_move("Aa1").unwrap();
    assert_eq!(
        mov,
        RegularMove {
            piece: Piece::Alfil,
            captured: None,
            from: None,
            to: Square::A1,
        }
    );
    assert_eq!(position.colored_regular_move(mov).to_string(), "Aa1");

    // Can't drop on an existing piece.
    assert_eq!(position.parse_regular_move("Ab2"), Err(ParseError));

    // Can't drop a piece we don't have.
    assert_eq!(position.parse_regular_move("Da1"), Err(ParseError));

    // Can't drop opponent's piece.
    assert_eq!(position.parse_regular_move("fa1"), Err(ParseError));
}
