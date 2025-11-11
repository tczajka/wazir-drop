use crate::history::History;

#[test]
fn test_history() {
    let mut history = History::new();
    history.clear(10);
    history.push(1);
    history.push(2);
    history.push(3);
    history.push(4);
    history.push(3);
    history.pop();
    assert_eq!(history.find(10), None);
    assert_eq!(history.find(3), Some(12));
    assert_eq!(history.find(1), Some(10));
}
