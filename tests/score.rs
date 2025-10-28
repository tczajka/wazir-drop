use wazir_drop::Score;

#[test]
fn test_score() {
    assert_eq!(Score::IMMEDIATE_WIN.to_string(), "#0");
    assert_eq!(Score::lose_in(3).to_string(), "-#3");
    assert_eq!(Score::from_eval(17).to_string(), "17");

    assert_eq!(Score::win_in(7).next(), Score::win_in(6));
    assert_eq!(Score::lose_in(3).next(), Score::lose_in(4));
    assert_eq!(Score::from_eval(17).next(), Score::from_eval(18));

    assert_eq!(Score::win_in(7).prev(), Score::win_in(8));
    assert_eq!(Score::lose_in(3).prev(), Score::lose_in(2));
    assert_eq!(Score::from_eval(17).prev(), Score::from_eval(16));

    assert_eq!(Score::win_in(7).back(), Score::lose_in(8));
    assert_eq!(Score::lose_in(3).back(), Score::win_in(4));
    assert_eq!(Score::from_eval(17).back(), Score::from_eval(-17));
    assert_eq!(Score::MIN_WIN.back(), -Score::MIN_WIN);
    assert_eq!((-Score::MIN_WIN).back(), Score::MIN_WIN);

    assert_eq!(Score::win_in(7).forward(), Score::lose_in(6));
    assert_eq!(Score::lose_in(3).forward(), Score::win_in(2));
    assert_eq!(Score::from_eval(17).forward(), Score::from_eval(-17));
    assert_eq!(Score::MIN_WIN.forward(), (-Score::MIN_WIN).prev());
    assert_eq!((-Score::MIN_WIN).forward(), Score::MIN_WIN.next());
}
