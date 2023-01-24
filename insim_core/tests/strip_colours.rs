use insim_core::string::colours;

#[test]
fn test_strip_colours_only() {
    assert_eq!(colours::strip("^1^2^3^4^5^6^7^8^9"), "");
}

#[test]
fn test_strip_colours() {
    assert_eq!(colours::strip("^1234^56789"), "2346789");
}

#[test]
fn test_strip_colours_escaped() {
    assert_eq!(colours::strip("^^1234^56789"), "^^12346789");
}
