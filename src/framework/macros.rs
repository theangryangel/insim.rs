#[macro_export]
macro_rules! event_handler {
    (
        $( #[$attr:meta] )*
        $vis:vis trait $name:ident for $client:ty, $enum:ident {
            $($variant:ident($inner:ty) => $fn:ident,)*
        }
    ) => {
        // emit the trait declaration
        $( #[$attr] )*
        $vis trait $name: Send + Sync {
            fn on_raw(&self, client: &Client, data: &$enum) {}

            fn on_startup(&self) {}
            fn on_shutdown(&self) {}

            fn on_connect(&self, client: &$client) {}
            fn on_disconnect(&self, client: &$client) {}
            fn on_timeout(&self, client: &$client) {}

            $(
                fn $fn(&self, client: &$client, data: &$inner) {}
            )*
        }

        impl $client {
            pub fn on_packet(&self, data: &$enum) {
                match data {
                    $(
                        $enum::$variant(ref inner) => {
                            for event_handler in self.config.event_handlers.iter() {
                                event_handler.$fn(&self, inner);
                            }
                        },
                    )*
                    _ => {},
                }
            }
        }
    };
}
