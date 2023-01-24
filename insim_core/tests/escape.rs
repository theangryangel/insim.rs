use insim_core::string::{escape, unescape};

#[test]
fn test_escaping() {
    let input = String::from("^|*:\\/?\"<>#12345");
    let output = String::from("^^^v^a^c^d^s^q^t^l^r^h12345");

    assert_eq!(escape(input.as_bytes()), output.as_bytes(),);
}

#[test]
fn test_unescaping() {
    let output = String::from("^|*:\\/?\"<>#12345");
    let input = String::from("^^^v^a^c^d^s^q^t^l^r^h12345");

    assert_eq!(unescape(input.as_bytes()), output.as_bytes(),);
}
