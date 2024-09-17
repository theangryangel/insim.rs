macro_rules! impl_typical_with_request_id {
    ($thing:ident) => {
        impl crate::WithRequestId for $thing {
            fn with_request_id<R: Into<crate::identifiers::RequestId>>(
                mut self,
                reqi: R,
            ) -> impl Into<crate::Packet> + std::fmt::Debug {
                self.reqi = reqi.into();
                self
            }
        }
    };
}
