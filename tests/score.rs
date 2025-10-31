use wazir_drop::{Score, ScoreExpanded};

#[test]
fn test_score_to_string() {
    assert_eq!(ScoreExpanded::Win(1).to_string(), "#1");
    assert_eq!(ScoreExpanded::Loss(3).to_string(), "-#3");
    assert_eq!(ScoreExpanded::Eval(17).to_string(), "17");
}

#[test]
fn test_score_to_absolute() {
    assert_eq!(
        ScoreExpanded::Eval(17).to_absolute(3),
        ScoreExpanded::Eval(17)
    );
    assert_eq!(ScoreExpanded::Win(0).to_absolute(3), ScoreExpanded::Win(3));
    assert_eq!(
        ScoreExpanded::Win(100).to_absolute(3),
        ScoreExpanded::Win(103)
    );
    assert_eq!(
        ScoreExpanded::Loss(0).to_absolute(3),
        ScoreExpanded::Loss(3)
    );
    assert_eq!(
        ScoreExpanded::Loss(100).to_absolute(3),
        ScoreExpanded::Loss(103)
    );
}

#[test]
fn test_score_to_relative() {
    assert_eq!(
        ScoreExpanded::Eval(17).to_relative(3),
        ScoreExpanded::Eval(17)
    );
    assert_eq!(ScoreExpanded::Win(0).to_relative(3), ScoreExpanded::Win(0));
    assert_eq!(
        ScoreExpanded::Win(100).to_relative(3),
        ScoreExpanded::Win(97)
    );
    assert_eq!(
        ScoreExpanded::Loss(0).to_relative(3),
        ScoreExpanded::Loss(0)
    );
    assert_eq!(
        ScoreExpanded::Loss(100).to_relative(3),
        ScoreExpanded::Loss(97)
    );
}

#[test]
fn test_score_offset() {
    assert_eq!(ScoreExpanded::Eval(17).offset(3), ScoreExpanded::Eval(20));
    assert_eq!(ScoreExpanded::Win(10).offset(3), ScoreExpanded::Win(10));
    assert_eq!(ScoreExpanded::Loss(10).offset(3), ScoreExpanded::Loss(10));
}

#[test]
fn test_score_prev_next() {
    assert_eq!(
        Score::from(ScoreExpanded::Eval(17)).prev(),
        Score::from(ScoreExpanded::Eval(16))
    );
    assert_eq!(
        Score::from(ScoreExpanded::Eval(17)).next(),
        Score::from(ScoreExpanded::Eval(18))
    );
    assert_eq!(
        Score::from(ScoreExpanded::Win(10)).prev(),
        Score::from(ScoreExpanded::Win(11))
    );
    assert_eq!(
        Score::from(ScoreExpanded::Win(10)).next(),
        Score::from(ScoreExpanded::Win(9))
    );
    assert_eq!(
        Score::from(ScoreExpanded::Loss(10)).prev(),
        Score::from(ScoreExpanded::Loss(9))
    );
    assert_eq!(
        Score::from(ScoreExpanded::Loss(10)).next(),
        Score::from(ScoreExpanded::Loss(11))
    );
}
