use std::str::FromStr;
use wazir_drop::{enums::SimpleEnumExt, Position, ShortMove, Stage};

#[test]
fn test_stage_display_round_trip() {
    for stage in Stage::all() {
        let name = stage.to_string();
        assert_eq!(Stage::from_str(&name), Ok(stage));
    }
}

#[test]
fn test_display_from_str() {
    // Opening.
    let s = "\
opening
red


........
........
........
........
........
........
........
........
";
    let position = Position::from_str(s).unwrap();
    assert_eq!(position.to_string(), s);

    // Red has placed something before opening.
    let s = "\
opening
red


W.......
........
........
........
........
........
........
........
";
    assert!(Position::from_str(s).is_err());

    let s = "\
opening
blue


WNFFDDDD
AAAAAAAA
........
........
........
........
........
........
";
    let position = Position::from_str(s).unwrap();
    assert_eq!(position.to_string(), s);

    // Invalid placement in opening.
    let s = "\
opening
blue


WNFFDDDD
AAAAAAA.
.......A
........
........
........
........
........
";
    assert!(Position::from_str(s).is_err());

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

    assert!(Position::from_str(s).is_err());

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
    assert!(Position::from_str(s).is_err());

    // Too many lines.
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
    assert!(Position::from_str(s).is_err());

    // Ended, no red wazir.
    let s = "\
end
red
AF
f
...A.D.D
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

    // Ended, but red wazir still there.
    let s = "\
end
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
    assert!(Position::from_str(s).is_err());
}

#[test]
fn test_move_from_short_move() {
    let s = "\
opening
red


........
........
........
........
........
........
........
........
";
    let position = Position::from_str(s).unwrap();
    let mov = position.move_from_short_move(ShortMove::from_str("AWNAADADAFFAADDA").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "AWNAADADAFFAADDA");

    assert!(position.move_from_short_move(ShortMove::from_str("AWNAADADAFFAADDN").unwrap()).is_err());
    assert!(position.move_from_short_move(ShortMove::from_str("awnaadadaffaadda").unwrap()).is_err());

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

    let mov = position.move_from_short_move(ShortMove::from_str("Aa1").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "A@a1");
    let mov = position.move_from_short_move(ShortMove::from_str("a2a3").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "Wa2-a3");
    let mov = position.move_from_short_move(ShortMove::from_str("a2b2").unwrap()).unwrap();
    assert_eq!(mov.to_string(), "Wa2xab2");

    assert!(position.move_from_short_move(ShortMove::from_str("fa1").unwrap()).is_err());
    assert!(position.move_from_short_move(ShortMove::from_str("a2c2").unwrap()).is_err());
    assert!(position.move_from_short_move(ShortMove::from_str("b3a4").unwrap()).is_err());
}