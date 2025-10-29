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

    assert_eq!(Score::from_eval(17).to_relative(3), Score::from_eval(17));
    assert_eq!(Score::win(0).to_relative(3), Score::win(0));
    assert_eq!(Score::win(100).to_relative(3), Score::win(97));
    assert_eq!(Score::loss(0).to_relative(3), Score::loss(0));
    assert_eq!(Score::loss(100).to_relative(3), Score::loss(97));

    assert_eq!(Score::from_eval(17).to_absolute(3), Score::from_eval(17));
    assert_eq!(Score::win(0).to_absolute(3), Score::win(3));
    assert_eq!(Score::win(100).to_absolute(3), Score::win(103));
    assert_eq!(Score::loss(0).to_absolute(3), Score::loss(3));
    assert_eq!(Score::loss(100).to_absolute(3), Score::loss(103));
}
