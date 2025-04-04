macro_rules! assert_from_to_bytes_bidirectional {
    ($thing:ident, $raw:expr) => {{
        let raw = $raw;

        // test binrw
        let parsed_binrw = $thing::read_le(&mut ::std::io::Cursor::new(&raw)).unwrap();
        let mut written_binrw = ::std::io::Cursor::new(Vec::new());
        parsed_binrw.write_le(&mut written_binrw).unwrap();
        assert_eq!(raw, written_binrw.into_inner(), "assert binrw reads and writes.");

        // test FromToBytes
        let mut parsed_buf = ::bytes::BytesMut::new();
        parsed_buf.extend_from_slice(&raw);

        let parsed = $thing::from_bytes(&mut parsed_buf.freeze()).unwrap();
        assert_eq!(buf.remaining(), 0);

        let mut written_buf = ::bytes::BytesMut::new();
        parsed.to_bytes(&mut written_buf).unwrap();

        assert_eq!(&raw, written_buf.as_ref(), "assert fromtobytes matches");

        parsed
    }};
}
