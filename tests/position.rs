use std::str::FromStr;
use wazir_drop::{Move, Outcome, Position, Stage};

#[test]
fn test_outcome_display_round_trip() {
    for o in [Outcome::RedWin, Outcome::Draw, Outcome::BlueWin] {
        assert_eq!(Outcome::from_str(&o.to_string()).unwrap(), o);
    }
}

#[test]
fn test_stage_display_round_trip() {
    for s in [
        Stage::Setup,
        Stage::Regular,
        Stage::End(Outcome::RedWin),
        Stage::End(Outcome::Draw),
        Stage::End(Outcome::BlueWin),
    ] {
        assert_eq!(Stage::from_str(&s.to_string()).unwrap(), s);
    }
}

#[test]
fn test_initial() {
    assert_eq!(
        Position::initial().to_string(),
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
"
    );
}

#[test]
fn test_display_from_str() {
    // Opening.
    let s = "\
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
";
    let position = Position::from_str(s).unwrap();
    assert_eq!(position.to_string(), s);

    // Red has placed something before setup.
    let s = "\
setup
0

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
1

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
1

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
";

    let position = Position::from_str(s).unwrap();
    assert_eq!(position.to_string(), s);

    // Too many ferzes.
    let s = "\
regular
4
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
4
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
4
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

    // Ended, red wazir captured.
    let s = "\
end blue_win
4
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
end blue_win
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
";
    assert!(Position::from_str(s).is_err());
}

#[test]
fn test_make_move() {
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

    let position2 = position
        .make_move(Move::from_str("AWNAADADAFFAADDA").unwrap())
        .unwrap();
    assert_eq!(
        position2.to_string(),
        "\
setup
1

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
2

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

    assert!(position
        .make_move(Move::from_str("AWNAADADAFFAADDA").unwrap())
        .is_err());

    let position2 = position.make_move(Move::from_str("A@a1").unwrap()).unwrap();
    assert_eq!(
        position2.to_string(),
        "\
regular
5
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
5
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
5
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
5
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
end blue_win
6
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

    let position = Position::from_str(
        "\
regular
101
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

    let position2 = position
        .make_move(Move::from_str("fg7-h6").unwrap())
        .unwrap();
    assert_eq!(
        position2.to_string(),
        "\
end draw
102
AFf
.W.A.D.D
AaFA.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a.....
add.wf.a
"
    );
}
