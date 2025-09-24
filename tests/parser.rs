use wazir_drop::{
    either::Either,
    parser::{self, Parser, ParserExt},
};

#[test]
fn test_parse_all() {
    assert_eq!(parser::Byte.parse_all(b"a"), Ok(b'a'));
    assert!(parser::Byte.parse_all(b"ab").is_err());
}

#[test]
fn test_end() {
    let result = parser::End.parse(b"").unwrap();
    assert_eq!(result.remaining, b"");

    assert!(parser::End.parse(b"a").is_err());
}

#[test]
fn test_byte() {
    let result = parser::Byte.parse(b"abc").unwrap();
    assert_eq!(result.value, b'a');
    assert_eq!(result.remaining, b"bc");

    assert!(parser::Byte.parse(b"").is_err());
}

#[test]
fn test_exact() {
    let p = parser::exact(b"abc");

    let result = p.parse(b"abcdef").unwrap();
    assert_eq!(result.remaining, b"def");

    assert!(p.parse(b"abdef").is_err());
}

#[test]
fn test_then() {
    let p = parser::Byte.then(parser::Byte);

    let result = p.parse(b"abcdef").unwrap();
    assert_eq!(result.value, (b'a', b'b'));
    assert_eq!(result.remaining, b"cdef");

    assert!(p.parse(b"a").is_err());
}

#[test]
fn test_or() {
    let p = parser::exact(b"abc").or(parser::exact(b"def"));

    let result = p.parse(b"abcxyz").unwrap();
    assert_eq!(result.value, Either::Left(()));
    assert_eq!(result.remaining, b"xyz");

    let result = p.parse(b"defxyz").unwrap();
    assert_eq!(result.value, Either::Right(()));
    assert_eq!(result.remaining, b"xyz");

    assert!(p.parse(b"xxx").is_err());
}

#[test]
fn test_map() {
    let p = parser::Byte.map(|b| b + 1);

    let result = p.parse(b"abc").unwrap();
    assert_eq!(result.value, b'b');
    assert_eq!(result.remaining, b"bc");

    assert!(p.parse(b"").is_err());
}

#[test]
fn test_then_ignore() {
    let result = parser::Byte
        .then_ignore(parser::Byte)
        .parse(b"abc")
        .unwrap();
    assert_eq!(result.value, b'a');
    assert_eq!(result.remaining, b"c");
}

#[test]
fn test_ignore_then() {
    let result = parser::Byte
        .ignore_then(parser::Byte)
        .parse(b"abc")
        .unwrap();
    assert_eq!(result.value, b'b');
    assert_eq!(result.remaining, b"c");
}

#[test]
fn test_repeat() {
    let p = parser::Byte.repeat(1..=3);
    let result = p.parse(b"abcdef").unwrap();
    assert_eq!(result.value, vec![b'a', b'b', b'c']);
    assert_eq!(result.remaining, b"def");

    assert!(p.parse(b"").is_err());

    let result = p.parse(b"ab").unwrap();
    assert_eq!(result.value, vec![b'a', b'b']);
    assert_eq!(result.remaining, b"");
}
