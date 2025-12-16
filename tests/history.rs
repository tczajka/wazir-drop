use wazir_drop::History;

#[test]
fn test_history() {
    let mut history = History::new(1);
    history.push(2);
    history.push(3);
    history.push(4);
    history.pop();
    assert_eq!(history.find_repetition(), None);
    history.push(2);
    assert_eq!(history.find_repetition(), Some(1));
    history.push_irreversible(4);
    assert_eq!(history.find_repetition(), None);
    history.pop();
    assert_eq!(history.find_repetition(), Some(1));
}
