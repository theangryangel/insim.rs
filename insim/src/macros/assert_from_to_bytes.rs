macro_rules! assert_from_to_bytes {
    ($thing:ident, $raw:expr, $fn:expr) => {{
        let raw = $raw;

        // test ReadWriteBuf
        let mut parsed_buf = ::bytes::BytesMut::new();
        parsed_buf.extend_from_slice(&raw);

        let mut parsed_buf = parsed_buf.freeze();
        let mut decode_ctx = ::insim_core::DecodeContext::new(&mut parsed_buf);

        let parsed = <$thing as ::insim_core::Decode>::decode(&mut decode_ctx).unwrap();
        let remaining = <::bytes::Bytes as ::bytes::Buf>::remaining(decode_ctx.buf);
        assert_eq!(
            remaining, 0,
            "expected 0 remaining bytes, found {}",
            remaining
        );

        let mut written_buf = ::bytes::BytesMut::new();
        let mut encode_ctx = ::insim_core::EncodeContext::new(&mut written_buf);
        <$thing as ::insim_core::Encode>::encode(&parsed, &mut encode_ctx).unwrap();

        assert_eq!(
            written_buf.as_ref(),
            raw,
            "assert reads and writes. left=actual, right=expected"
        );
        $fn(parsed);
    }};
}
