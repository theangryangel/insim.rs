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

pub(crate) struct ConnectionComponent {
    pub(crate) state: Arc<State>,
    pub(crate) tx: Arc<Notify>,
}

#[async_trait]
impl LiveView for ConnectionComponent {
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
        let connections = self.state.get_connections();
        html! {
            if connections.is_empty() {
                <p>"Its quiet, too quiet..."</p>
            } else {
                <ul>
                    for (ucid, connection) in connections.iter() {
                        <li>
                            { &ucid }
                            "="
                            { format!("{:?}", &connection) }
                        </li>
                    }
                </ul>
            }
        }
    }
}
