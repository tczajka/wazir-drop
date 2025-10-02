use std::str::FromStr;
use wazir_drop::{parser::ParseError, Color, Piece, Position, Square};

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

    let position = Position::from_str(s).unwrap();
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

    assert_eq!(Position::from_str(s), Err(ParseError));

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
    assert_eq!(Position::from_str(s), Err(ParseError));

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
    assert_eq!(Position::from_str(s), Err(ParseError));
}
