use insim_core::binrw::BinWrite;
use insim_pth::Pth;
use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    path::PathBuf,
};

fn assert_valid_as1_pth(p: &Pth) {
    assert_eq!(p.version, 0);
    assert_eq!(p.revision, 0);
    assert_eq!(p.finish_line_node, 250);
}

#[test]
fn test_pth_decode_from_pathbuf() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1.pth");
    let p = Pth::from_pathbuf(&path).expect("Expected PTH file to be parsed");

    assert_valid_as1_pth(&p)
}

#[test]
fn test_pth_decode_from_file() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1.pth");
    let mut file = File::open(path).expect("Expected Autocross_3DH.smx to exist");
    let p = Pth::from_file(&mut file).expect("Expected PTH file to be parsed");

    let pos = file.stream_position().unwrap();
    let end = file.seek(SeekFrom::End(0)).unwrap();

    assert_eq!(pos, end, "Expected the whole file to be completely read");

    assert_valid_as1_pth(&p)
}

#[test]
fn test_pth_encode() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1.pth");
    let p = Pth::from_pathbuf(&path).expect("Expected SMX file to be parsed");

    let mut file = File::open(path).expect("Expected AS1.pth to exist");
    let mut raw: Vec<u8> = Vec::new();
    file.read_to_end(&mut raw)
        .expect("Expected to read whole file");

    let mut writer = Cursor::new(Vec::new());
    p.write(&mut writer)
        .expect("Expected to write the whole file");

    let inner = writer.into_inner();
    assert_eq!(inner, raw);
}
