use insim_core::string::unescape;

#[test]
fn test_unescaping() {
    let output = String::from("^|*:\\/?\"<>#12345");
    let input = String::from("^^^v^a^c^d^s^q^t^l^r^h12345");

    assert_eq!(unescape(input.as_bytes()), output.as_bytes(),);
}
