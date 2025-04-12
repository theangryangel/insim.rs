macro_rules! assert_from_to_bytes {
    ($thing:ident, $raw:expr, $fn:expr) => {{
        let raw = $raw;

        // test binrw
        let parsed_binrw =
            <$thing as ::insim_core::binrw::BinRead>::read_le(&mut ::std::io::Cursor::new(&raw))
                .unwrap();
        let mut written_binrw = ::std::io::Cursor::new(Vec::new());
        <$thing as ::insim_core::binrw::BinWrite>::write_le(&parsed_binrw, &mut written_binrw)
            .unwrap();
        let written_binrw_inner = written_binrw.into_inner();
        assert_eq!(
            written_binrw_inner, raw,
            "assert binrw reads and writes. left=actual, right=expected"
        );

        $fn(parsed_binrw);

        // test FromToBytes
        let mut parsed_buf = ::bytes::BytesMut::new();
        parsed_buf.extend_from_slice(&raw);

        let mut parsed_buf = parsed_buf.freeze();

        let parsed = <$thing as ::insim_core::FromToBytes>::from_bytes(&mut parsed_buf).unwrap();
        let remaining = <::bytes::Bytes as ::bytes::Buf>::remaining(&parsed_buf);
        assert_eq!(
            remaining, 0,
            "expected 0 remaining bytes, found {}",
            remaining
        );

        let mut written_buf = ::bytes::BytesMut::new();
        <$thing as ::insim_core::FromToBytes>::to_bytes(&parsed, &mut written_buf).unwrap();

        assert_eq!(
            written_buf.as_ref(),
            raw,
            "assert reads and writes. left=actual, right=expected"
        );
        $fn(parsed);
    }};
}
