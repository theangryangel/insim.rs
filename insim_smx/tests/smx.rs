use insim_smx::Smx;
use std::path::PathBuf;

#[test]
fn test_smx_decode() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/Autocross_3DH.smx");
    let p = Smx::from_file(&path).expect("Expected SMX file to be parsed");

    assert_eq!(p.num_objects, 1666);
    assert_eq!(p.objects.len(), p.num_objects as usize);
    assert_eq!(p.checkpoint_object_index.len(), p.num_checkpoints as usize);
    assert_eq!(p.track, "Autocross");
}
