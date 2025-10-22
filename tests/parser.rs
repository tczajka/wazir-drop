use wazir_drop::parser::{self, Parser, ParserExt};

#[test]
fn test_parse_all() {
    let p = parser::byte();
    assert_eq!(p.parse_all(b"a"), Ok(b'a'));
    assert!(p.parse_all(b"ab").is_err());
}

#[test]
fn test_empty() {
    let p = parser::empty();
    let result = p.parse(b"abc").unwrap();
    assert_eq!(result.remaining, b"abc");
}

#[test]
fn test_end() {
    let p = parser::end();
    let result = p.parse(b"").unwrap();
    assert_eq!(result.remaining, b"");

    assert!(p.parse(b"a").is_err());
}

#[test]
fn test_byte() {
    let p = parser::byte();
    let result = p.parse(b"abc").unwrap();
    assert_eq!(result.value, b'a');
    assert_eq!(result.remaining, b"bc");

    assert!(p.parse(b"").is_err());
}

#[test]
fn test_exact() {
    let p = parser::exact(b"abc");

    let result = p.parse(b"abcdef").unwrap();
    assert_eq!(result.remaining, b"def");

    assert!(p.parse(b"abdef").is_err());
}

#[test]
fn test_endl() {
    let p = parser::endl();
    let result = p.parse(b"\nabc").unwrap();
    assert_eq!(result.remaining, b"abc");

    assert!(p.parse(b"abc\n").is_err());
}

#[test]
fn test_and() {
    let p = parser::byte().and(parser::byte());

    let result = p.parse(b"abcdef").unwrap();
    assert_eq!(result.value, (b'a', b'b'));
    assert_eq!(result.remaining, b"cdef");

    assert!(p.parse(b"a").is_err());
}

#[test]
fn test_and_then() {
    let p = parser::byte().and_then(|b| {
        let n = usize::from(b - b'0');
        parser::byte().repeat(n..=n)
    });

    let result = p.parse(b"3bcdef").unwrap();
    assert_eq!(result.value, vec![b'b', b'c', b'd']);
    assert_eq!(result.remaining, b"ef");

    assert!(p.parse(b"3ab").is_err());
}

#[test]
fn test_or() {
    let p = parser::exact(b"abc").or(parser::exact(b"def"));

    let result = p.parse(b"abcxyz").unwrap();
    assert_eq!(result.remaining, b"xyz");

    let result = p.parse(b"defxyz").unwrap();
    assert_eq!(result.remaining, b"xyz");

    assert!(p.parse(b"xxx").is_err());
}

#[test]
fn test_map() {
    let p = parser::byte().map(|b| b + 1);

    let result = p.parse(b"abc").unwrap();
    assert_eq!(result.value, b'b');
    assert_eq!(result.remaining, b"bc");

    assert!(p.parse(b"").is_err());
}

#[test]
fn test_then_ignore() {
    let result = parser::byte()
        .then_ignore(parser::byte())
        .parse(b"abc")
        .unwrap();
    assert_eq!(result.value, b'a');
    assert_eq!(result.remaining, b"c");
}

#[test]
fn test_ignore_then() {
    let result = parser::byte()
        .ignore_then(parser::byte())
        .parse(b"abc")
        .unwrap();
    assert_eq!(result.value, b'b');
    assert_eq!(result.remaining, b"c");
}

#[test]
fn test_repeat() {
    let p = parser::byte().repeat(1..=3);
    let result = p.parse(b"abcdef").unwrap();
    assert_eq!(result.value, vec![b'a', b'b', b'c']);
    assert_eq!(result.remaining, b"def");

    assert!(p.parse(b"").is_err());

    let result = p.parse(b"ab").unwrap();
    assert_eq!(result.value, vec![b'a', b'b']);
    assert_eq!(result.remaining, b"");
}
