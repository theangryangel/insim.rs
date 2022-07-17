use crate::state::State;
use axum::{
    async_trait,
    http::{HeaderMap, Uri},
};
use axum_live_view::{
    event_data::EventData,
    html,
    live_view::{Updated, ViewHandle},
    Html, LiveView,
};
use std::sync::Arc;
use tokio::sync::Notify;

pub(crate) struct ChatComponent {
    pub(crate) state: Arc<State>,
    pub(crate) tx: Arc<Notify>,
}

#[async_trait]
impl LiveView for ChatComponent {
    type Message = ();
    type Error = std::convert::Infallible;

    async fn mount(
        &mut self,
        _: Uri,
        _: &HeaderMap,
        handle: ViewHandle<Self::Message>,
    ) -> Result<(), Self::Error> {
        tokio::spawn({
            let tx = self.tx.clone();
            async move {
                loop {
                    tx.notified().await;
                    if handle.send(()).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn update(
        mut self,
        _msg: (),
        _data: Option<EventData>,
    ) -> Result<Updated<Self>, Self::Error> {
        Ok(Updated::new(self))
    }

    fn render(&self) -> Html<Self::Message> {
        let messages = self.state.chat();
        html! {
            if messages.is_empty() {
                <p>"Its quiet, too quiet..."</p>
            } else {
                <ul>
                    for msg in messages.iter() {
                        <li>
                            { &msg.at } { &msg.ucid }
                            <div>
                                { &msg.body }
                            </div>
                        </li>
                    }
                </ul>
            }
        }
    }
}
