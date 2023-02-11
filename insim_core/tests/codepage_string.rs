use insim_core::string::codepages;

#[test]
fn test_codepage_from_string_lossy() {
    let output = codepages::to_lossy_bytes("Hello");

    assert_eq!(output, "Hello".as_bytes(),);
}
