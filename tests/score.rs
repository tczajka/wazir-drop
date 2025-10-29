use wazir_drop::Score;

#[test]
fn test_score() {
    assert_eq!(Score::win(1).to_string(), "#1");
    assert_eq!(Score::loss(3).to_string(), "-#3");
    assert_eq!(Score::from_eval(17).to_string(), "17");

    assert_eq!(Score::win(7).next(), Score::win(6));
    assert_eq!(Score::loss(3).next(), Score::loss(4));
    assert_eq!(Score::from_eval(17).next(), Score::from_eval(18));

    assert_eq!(Score::win(7).prev(), Score::win(8));
    assert_eq!(Score::loss(3).prev(), Score::loss(2));
    assert_eq!(Score::from_eval(17).prev(), Score::from_eval(16));

    assert_eq!(Score::win(7).back(), Score::loss(8));
    assert_eq!(Score::loss(3).back(), Score::win(4));
    assert_eq!(Score::from_eval(17).back(), Score::from_eval(-17));
    assert_eq!(Score::MIN_WIN.back(), -Score::MIN_WIN);
    assert_eq!((-Score::MIN_WIN).back(), Score::MIN_WIN);

    assert_eq!(Score::win(7).forward(), Score::loss(6));
    assert_eq!(Score::loss(3).forward(), Score::win(2));
    assert_eq!(Score::from_eval(17).forward(), Score::from_eval(-17));
    assert_eq!(Score::MIN_WIN.forward(), (-Score::MIN_WIN).prev());
    assert_eq!((-Score::MIN_WIN).forward(), Score::MIN_WIN.next());
}
