use std::str::FromStr;
use wazir_drop::{enums::SimpleEnumExt, Move, Position, Stage};

#[test]
fn test_stage_display_round_trip() {
    for stage in Stage::all() {
        let name = stage.to_string();
        assert_eq!(Stage::from_str(&name), Ok(stage));
    }
}

#[test]
fn test_initial() {
    assert_eq!(
        Position::initial().to_string(),
        "\
setup
red

........
........
........
........
........
........
........
........
"
    );
}

#[test]
fn test_display_from_str() {
    // Opening.
    let s = "\
setup
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

    // Red has placed something before setup.
    let s = "\
setup
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
setup
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

    // Invalid placement in setup.
    let s = "\
setup
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
AFf
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
AFFf
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
Af
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
Af
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
AFfw
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
AFf
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
fn test_make_move() {
    let position = Position::from_str(
        "\
setup
red

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

    let position2 = position
        .make_move(Move::from_str("AWNAADADAFFAADDA").unwrap())
        .unwrap();
    assert_eq!(
        position2.to_string(),
        "\
setup
blue

AWNAADAD
AFFAADDA
........
........
........
........
........
........
"
    );

    assert!(position
        .make_move(Move::from_str("awnaadadaffaadda").unwrap())
        .is_err());

    let position3 = position2
        .make_move(Move::from_str("awnaadadaffaadda").unwrap())
        .unwrap();
    assert_eq!(
        position3.to_string(),
        "\
regular
red

AWNAADAD
AFFAADDA
........
........
........
........
awnaadad
affaadda
"
    );

    let position = Position::from_str(
        "\
regular
red
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

    assert!(position
        .make_move(Move::from_str("AWNAADADAFFAADDA").unwrap())
        .is_err());

    let position2 = position.make_move(Move::from_str("A@a1").unwrap()).unwrap();
    assert_eq!(
        position2.to_string(),
        "\
regular
blue
Ff
AW.A.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
"
    );

    assert!(position.make_move(Move::from_str("A@a2").unwrap()).is_err());
    assert!(position.make_move(Move::from_str("A@b2").unwrap()).is_err());
    assert!(position.make_move(Move::from_str("N@a1").unwrap()).is_err());
    assert!(position.make_move(Move::from_str("f@a1").unwrap()).is_err());

    let position2 = position
        .make_move(Move::from_str("Wa2-a3").unwrap())
        .unwrap();
    assert_eq!(
        position2.to_string(),
        "\
regular
blue
AFf
..WA.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
"
    );
    assert!(position
        .make_move(Move::from_str("Wa2-c2").unwrap())
        .is_err());
    assert!(position
        .make_move(Move::from_str("Fb3-a4").unwrap())
        .is_err());

    let position2 = position
        .make_move(Move::from_str("Wa2xab2").unwrap())
        .unwrap();
    assert_eq!(
        position2.to_string(),
        "\
regular
blue
AAFf
...A.D.D
AWFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
"
    );
    assert!(position
        .make_move(Move::from_str("Wa2xnb2").unwrap())
        .is_err());

    let position = Position::from_str(
        "\
regular
blue
AFf
.W.A.D.D
AaFA.DDA
n.A.A.A.
......A.
...a.a.d
..d...N.
a.a...f.
add.w..a
",
    )
    .unwrap();

    let position2 = position
        .make_move(Move::from_str("nc1xWa2").unwrap())
        .unwrap();
    assert_eq!(
        position2.to_string(),
        "\
end
red
AFfw
.n.A.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d...N.
a.a...f.
add.w..a
",
    );
}
