use wazir_drop::{
    either::Either,
    parser::{self, Parser, ParserExt},
};

#[test]
fn test_end() {
    let result = parser::End.parse(b"").unwrap();
    assert_eq!(result.value, ());
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
    assert_eq!(result.value, ());
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
