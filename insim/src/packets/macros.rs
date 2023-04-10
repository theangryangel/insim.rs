/// Internal macro to define conversion implementations between a Packet enum variant and an inner
/// value of the enum variant.
#[macro_export]
macro_rules! impl_packet_from {
    (
        $(
            $inner:ty => $variant:ident$(,)?
        )+
    ) => {
        // Implement From for all our variants so that we can use do insim::Init().into() to get a
        // Packet::Init(..) variant
        $(
        impl From<$inner> for Packet {
            fn from(item: $inner) -> Self {
                Packet::$variant(item)
            }
        }
        )+
    }
}
