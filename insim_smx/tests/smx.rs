use insim_core::binrw::BinWrite;
use insim_smx::Smx;
use std::{
    fs::File,
    io::Seek,
    io::{Cursor, Read, SeekFrom},
    path::PathBuf,
};

fn assert_valid_autocross_3dh(p: &Smx) {
    assert_eq!(p.objects.len(), 1666);
    assert_eq!(p.checkpoint_object_index.len(), 6);
    assert_eq!(p.track, "Autocross");
    assert_eq!(p.track.as_bytes().len(), 9);
}

#[test]
fn test_smx_decode_from_pathbuf() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/Autocross_3DH.smx");
    let p = Smx::from_pathbuf(&path).expect("Expected SMX file to be parsed");

    assert_valid_autocross_3dh(&p);
}

#[test]
fn test_smx_decode_from_file() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/Autocross_3DH.smx");
    let mut file = File::open(path).expect("Expected Autocross_3DH.smx to exist");
    let p = Smx::from_file(&mut file).expect("Expected SMX file to be parsed");

    let pos = file.stream_position().unwrap();
    let end = file.seek(SeekFrom::End(0)).unwrap();

    assert_eq!(pos, end, "Expected the whole file to be completely read");

    assert_valid_autocross_3dh(&p);
}

#[test]
fn test_smx_encode() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/Autocross_3DH.smx");
    let p = Smx::from_pathbuf(&path).expect("Expected SMX file to be parsed");

    let mut file = File::open(path).expect("Expected Autocross_3DH.smx to exist");
    let mut raw: Vec<u8> = Vec::new();
    file.read_to_end(&mut raw)
        .expect("Expected to read whole file");

    let mut writer = Cursor::new(Vec::new());
    p.write(&mut writer)
        .expect("Expected to write the whole file");

    let inner = writer.into_inner();
    assert_eq!(inner, raw);
}
