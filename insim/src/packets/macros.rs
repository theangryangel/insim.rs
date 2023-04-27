/// Internal macro to define conversion implementations between a Packet enum variant and an inner
/// value of the enum variant.
#[macro_export]
macro_rules! impl_packet_traits {
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


        impl RequestIdentifiable for $inner {
            fn request_identifier(&self) -> RequestId {
                self.reqi
            }

            fn set_request_identifier(&mut self, reqi: RequestId) {
                self.reqi = reqi;
            }
        }
        )+
    }
}
