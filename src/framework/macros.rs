/// Internal macro generate event handlers for the high level framework API.
#[macro_export]
macro_rules! event_handler {
    (
        $( #[$trait_attr:meta] )*
        $vis:vis trait $name:ident for $client:ty, $enum:ident {
            $(
                $(#[$variant_attr:meta])*
                $variant:ident($inner:ty) => $fn:ident,
            )*
        }
    ) => {
        // emit the trait declaration
        /// Core trait for handling events from [Client].
        $( #[$trait_attr] )*
        $vis trait $name: Send + Sync {
            /// Called whenever any [Packet](super::protocol::Packet) is received.
            fn on_raw(&self, client: &Client, data: &$enum) {}

            /// Called when a [Client] is built.
            fn on_startup(&self) {}

            /// Called when a [Client] is shutdown.
            fn on_shutdown(&self) {}

            /// Called when a [Client] is successfully connected to the server.
            fn on_connect(&self, client: &$client) {}

            /// Called when a [Client] is disconnected from the server.
            fn on_disconnect(&self, client: &$client) {}

            /// Called when a [Client] timeout occurs.
            fn on_timeout(&self, client: &$client) {}

            $(
                $(#[$variant_attr])*
                fn $fn(&self, client: &$client, data: &$inner) {}
            )*
        }

        impl $client {
            pub(crate) fn on_packet(&self, data: &$enum) {
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
