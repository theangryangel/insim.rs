use insim_pth::Pth;

#[test]
fn test_pth_decode() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1.pth");
    let p = Pth::from_file(&path).expect("Expected PTH file to be parsed");

    assert!(p.nodes.len() == p.num_nodes as usize);
}
