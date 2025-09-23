use wazir_drop::{
    either::Either,
    parser::{self, Parser, ParserExt},
};

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
    let result = parser::exact(b"abc").parse(b"abcdef").unwrap();
    assert_eq!(result.remaining, b"def");

    assert!(parser::exact(b"abc").parse(b"abdef").is_err());
}

#[test]
fn test_pair() {
    let result = parser::Byte.then(parser::Byte).parse(b"abcdef").unwrap();
    assert_eq!(result.value, (b'a', b'b'));
    assert_eq!(result.remaining, b"cdef");

    assert!(parser::Byte
        .then(parser::exact(b"abc"))
        .parse(b"xxx")
        .is_err());
}

#[test]
fn test_or() {
    let parser = parser::exact(b"abc").or(parser::exact(b"def"));

    let result = parser.parse(b"abcxyz").unwrap();
    assert_eq!(result.value, Either::Left(()));
    assert_eq!(result.remaining, b"xyz");

    let result = parser.parse(b"defxyz").unwrap();
    assert_eq!(result.value, Either::Right(()));
    assert_eq!(result.remaining, b"xyz");

    assert!(parser.parse(b"xxx").is_err());
}

#[test]
fn test_map() {
    let result = parser::Byte.map(|b| b + 1).parse(b"abc").unwrap();
    assert_eq!(result.value, b'b');
    assert_eq!(result.remaining, b"bc");

    assert!(parser::Byte.map(|b| b + 1).parse(b"").is_err());
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
    let result = parser::Byte.repeat(1..=3).parse(b"abc").unwrap();
    assert_eq!(result.value, vec![b'a', b'b', b'c']);
    assert_eq!(result.remaining, b"");

    assert!(parser::Byte.repeat(1..=3).parse(b"").is_err());
    assert!(parser::Byte.repeat(1..=3).parse(b"abcde").is_err());
}
