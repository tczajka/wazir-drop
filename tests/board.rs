use std::str::FromStr;

use wazir_drop::{Board, Color, ColoredPiece, Square};

#[test]
fn test_display_from_str() {
    let s = "\
.W.A.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
";
    let board = Board::from_str(s).unwrap();
    assert_eq!(board.to_string(), s);
}

#[test]
fn test_place_remove_piece() {
    let mut board = Board::from_str(
        "\
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

    board
        .place_piece(Square::A1, ColoredPiece::BlueDabbaba)
        .unwrap();
    board
        .remove_piece(Square::B1, ColoredPiece::RedAlfil)
        .unwrap();

    assert_eq!(
        board.to_string(),
        "\
dW.A.D.D
.aFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
"
    );

    assert!(board
        .place_piece(Square::A1, ColoredPiece::RedWazir)
        .is_err());
    assert!(board
        .remove_piece(Square::A1, ColoredPiece::RedWazir)
        .is_err());
    assert!(board
        .remove_piece(Square::B1, ColoredPiece::RedWazir)
        .is_err());
}

#[test]
fn test_square() {
    let board = Board::from_str(
        "\
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

    assert_eq!(board.square(Square::A1), None);
    assert_eq!(board.square(Square::A2), Some(ColoredPiece::RedWazir));
}

#[test]
fn test_occupied_by() {
    let board = Board::from_str(
        "\
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

    assert_eq!(
        board.occupied_by(Color::Red).to_string(),
        "\
.x.x.x.x
x.xx.xxx
..x.x.x.
......x.
........
......x.
........
........
"
    );

    assert_eq!(
        board.occupied_by(Color::Blue).to_string(),
        "\
........
.x......
........
........
...x.x.x
..x..x..
x.x...x.
xxx.x..x
"
    );

    assert_eq!(
        board.empty_squares().to_string(),
        "\
x.x.x.x.
....x...
xx.x.x.x
xxxxxx.x
xxx.x.x.
xx.xx..x
.x.xxx.x
...x.xx.
"
    );
}

#[test]
fn test_occupied_by_piece() {
    let board = Board::from_str(
        "\
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

    assert_eq!(
        board
            .occupied_by_piece(ColoredPiece::RedDabbaba)
            .to_string(),
        "\
.....x.x
.....xx.
........
........
........
........
........
........
"
    );
}
