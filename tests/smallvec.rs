use wazir_drop::smallvec::SmallVec;

#[test]
fn test_smallvec() {
    let mut vec = SmallVec::<u32, 3>::new();
    vec.push(1);
    vec.push(2);
    assert_eq!(vec.len(), 2);
    assert_eq!(&vec[..], &[1, 2]);
    assert_eq!(vec[1], 2);
    assert_eq!(vec.pop(), Some(2));
    assert_eq!(vec.pop(), Some(1));
    assert_eq!(vec.pop(), None);
    assert!(vec.is_empty());
}

#[test]
fn test_from_iter() {
    let vec = SmallVec::<u32, 3>::from_iter([1, 2, 3]);
    assert_eq!(&vec[..], &[1, 2, 3]);
}

#[test]
fn test_into_iter() {
    let vec = SmallVec::<String, 3>::from_iter(["a".to_string(), "b".to_string()]);
    let v: Vec<String> = vec.into_iter().collect();
    assert_eq!(v, vec!["a", "b"]);
}
