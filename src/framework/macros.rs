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
            $(
                pub fn $fn(&mut self, inner_func: fn(Ctx<$state>, &$inner)) {
                    let boxed_fn = Box::new(move |ctx: Ctx<$state>, packet: &$enum| {
                        if let $enum::$variant(inner_packet) = packet {
                            inner_func(ctx, inner_packet);
                        }
                    });

                    let key = $enum::name_into_id(stringify!($variant)).unwrap();

                    if let Some(handlers) = self.handlers.on_packet_handlers.lock().unwrap().get_mut(&key) {
                        handlers.push(boxed_fn);
                    } else {
                        self.handlers.on_packet_handlers.lock().unwrap().insert(key, vec![boxed_fn]);
                    }

                }
            )*

        }

    }
}
