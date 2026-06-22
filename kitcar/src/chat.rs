//! [`Handler`] impl for [`insim_extra::chat::ChatParser`].

use std::{any::Any, future::Future, str::FromStr};

pub use insim_extra::chat::{ChatEvent, ChatParser};

use crate::{AppError, Dispatch, ExtractCx, Handler};

impl<S, V, C> Handler<(), S, V> for ChatParser<C>
where
    S: Send + Sync + 'static,
    V: crate::ui::View + 'static,
    C: FromStr + Any + Send + Sync + Clone + 'static,
{
    fn call(self, cx: &ExtractCx<'_, S, V>) -> impl Future<Output = Result<(), AppError>> + Send {
        let maybe_packet = if let Dispatch::Packet(p) = cx.dispatch {
            Some(p.clone())
        } else {
            None
        };
        let sender = cx.sender.clone();
        async move {
            if let Some(p) = maybe_packet
                && let Some(msg) = self.parse(&p)
            {
                let _ = sender.event(msg);
            }
            Ok(())
        }
    }
}
