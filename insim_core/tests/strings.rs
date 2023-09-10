use std::borrow::Cow;

use insim_core::string::{codepages, escape, unescape};

#[test]
fn test_escaping_and_unescaping() {
    let original: Cow<str> = "^|*:\\/?\"<>#123^945".into();

    let escaped = escape(original.clone());
    let unescaped = unescape(escaped.clone());

    assert_eq!(escaped, "^^^v^a^c^d^s^q^t^l^r^h123^945");
    assert_eq!(unescaped, original);
}

#[test]
fn test_codepage_hello_world() {
    let output = codepages::to_lossy_bytes("Hello");

    assert_eq!(output, "Hello".as_bytes(),);
}

// sample utf-8 strings from https://www.cl.cam.ac.uk/~mgk25/ucs/examples/quickbrown.txt

#[test]
fn test_codepage_to_hungarian() {
    // flood-proof mirror-drilling machine
    let as_bytes = codepages::to_lossy_bytes("Árvíztűrő tükörfúrógép");

    assert_eq!(
        codepages::to_lossy_string(&as_bytes),
        "Árvízt?r? tükörfúrógép",
    );
}

#[test]
fn test_codepage_to_mixed() {
    // flood-proof mirror-drilling machine
    let as_bytes = codepages::to_lossy_bytes("TEST Árvíztűrő tükörfúrógép");

    assert_eq!(
        codepages::to_lossy_string(&as_bytes),
        "TEST Árvízt?r? tükörfúrógép",
    );
}
