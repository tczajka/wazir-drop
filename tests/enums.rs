use wazir_drop::{
    enums::{EnumMap, SimpleEnumExt},
    unsafe_simple_enum,
};

#[repr(u8)]
enum Foo {
    A,
    B,
    C,
}

unsafe_simple_enum!(Foo, 3);

#[test]
fn test_all() {
    let mut all = Foo::all();
    assert!(matches!(all.next(), Some(Foo::A)));
    assert!(matches!(all.next(), Some(Foo::B)));
    assert!(matches!(all.next(), Some(Foo::C)));
    assert!(all.next().is_none());
}

#[test]
fn test_index() {
    let mut map: EnumMap<Foo, usize> = EnumMap::from_fn(Foo::index);
    assert_eq!(map[Foo::A], 0);
    assert_eq!(map[Foo::B], 1);
    assert_eq!(map[Foo::C], 2);
    map[Foo::A] = 4;
    assert_eq!(map[Foo::A], 4);
}

#[test]
fn test_iter() {
    let map: EnumMap<Foo, usize> = EnumMap::from_fn(Foo::index);
    let mut iter = map.iter();
    assert!(matches!(iter.next(), Some((Foo::A, &0))));
    assert!(matches!(iter.next(), Some((Foo::B, &1))));
    assert!(matches!(iter.next(), Some((Foo::C, &2))));
    assert!(iter.next().is_none());
}

#[test]
fn test_iter_mut() {
    let mut map: EnumMap<Foo, usize> = EnumMap::from_fn(Foo::index);
    let mut iter = map.iter_mut();
    assert!(matches!(iter.next(), Some((Foo::A, &mut 0))));
    assert!(matches!(iter.next(), Some((Foo::B, &mut 1))));
    assert!(matches!(iter.next(), Some((Foo::C, &mut 2))));
    assert!(iter.next().is_none());
}
