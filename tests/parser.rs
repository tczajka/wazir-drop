use wazir_drop::parser::{self, Parser, ParserExt};

#[test]
fn test_parse_all() {
    let result = parser::Byte.parse_all(b"a");
    assert_eq!(result, Ok(b'a'));

    assert!(parser::Byte.parse_all(b"ab").is_err());
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
