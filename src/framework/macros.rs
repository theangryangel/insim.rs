/// Internal macro generate event handlers for the high level framework API.

#[macro_export]
macro_rules! packet_handlers {
    (
        $client:ident<$state:ident> for $enum:ident
        {
            $(
                $(#[$variant_attr:meta])*
                $variant:ident($inner:ty) => $fn:ident,
            )*
        }
    ) => {

        impl<$state> $client<$state>
        where
            State: Clone + Send + Sync + 'static,
        {
            // TODO: Is there anything we can do to speed this up? With large numbers of packet
            // handlers this could get out of hand. Is something like a HashMap instead of a Vec
            // something workable? Would that be better or worse? Lets benchmark it.
            $(
                pub fn $fn(&mut self, inner_func: fn(Ctx<$state>, &$inner)) {
                    self.on_packet_handlers.push(
                        Box::new(move |ctx: Ctx<$state>, packet: &$enum| {
                            if let $enum::$variant(inner_packet) = packet {
                                inner_func(ctx, inner_packet);
                            }
                        }),
                    );
                }
            )*

        }

    }
}
