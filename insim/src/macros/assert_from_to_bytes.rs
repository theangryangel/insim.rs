macro_rules! assert_from_to_bytes {
    ($thing:ident, $raw:expr, $fn:expr) => {{
        let raw = $raw;

        // test ReadWriteBuf
        let mut parsed_buf = ::bytes::BytesMut::new();
        parsed_buf.extend_from_slice(&raw);

        let mut parsed_buf = parsed_buf.freeze();

        let parsed = <$thing as ::insim_core::Decode>::decode(&mut parsed_buf).unwrap();
        let remaining = <::bytes::Bytes as ::bytes::Buf>::remaining(&parsed_buf);
        assert_eq!(
            remaining, 0,
            "expected 0 remaining bytes, found {}",
            remaining
        );

        let mut written_buf = ::bytes::BytesMut::new();
        <$thing as ::insim_core::Encode>::encode(&parsed, &mut written_buf).unwrap();

        assert_eq!(
            written_buf.as_ref(),
            raw,
            "assert reads and writes. left=actual, right=expected"
        );
        $fn(parsed);
    }};
}
