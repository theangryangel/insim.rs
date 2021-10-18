#[macro_export]
macro_rules! into_packet_variant {
    ($from:ty, $to:ident) => {
        impl From<$from> for crate::protocol::Packet {
            fn from(item: $from) -> Self {
                Self::$to(item)
            }
        }
    };
}
