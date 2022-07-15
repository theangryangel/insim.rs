use super::{task::Message, State};
use axum::{
    async_trait,
    extract::{Extension, Path},
    http::{HeaderMap, Uri},
    response::IntoResponse,
};
use axum_live_view::{
    event_data::EventData,
    html,
    live_view::{Updated, ViewHandle},
    Html, LiveView, LiveViewUpgrade,
};
use bounded_vec_deque::BoundedVecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub(crate) async fn index(Extension(state): Extension<State>) -> String {
    let db = &state.read().unwrap();

    db.keys()
        .map(|key| key.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

pub(crate) async fn server_index(
    live: LiveViewUpgrade,
    Path(server): Path<String>,
    Extension(state): Extension<State>,
) -> impl IntoResponse {
    let db = &state.read().unwrap();

    let value = db.get(&server);
    // TODO throw 404
    let value = value.unwrap();

    let messages = MessagesList {
        messages: value.chat.clone(),
        tx: value.change.clone(),
    };

    live.response(move |embed| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                </head>
                <body>
                    { embed.embed(messages) }
                    <script src="/bundle.js"></script>
                </body>
            </html>
        }
    })
}

struct MessagesList {
    messages: Arc<Mutex<BoundedVecDeque<Message>>>,
    tx: broadcast::Sender<()>,
}

#[async_trait]
impl LiveView for MessagesList {
    type Message = ();
    type Error = std::convert::Infallible;

    async fn mount(
        &mut self,
        _: Uri,
        _: &HeaderMap,
        handle: ViewHandle<Self::Message>,
    ) -> Result<(), Self::Error> {
        let mut rx = self.tx.subscribe();
        tokio::spawn(async move {
            while let Ok(()) = rx.recv().await {
                if handle.send(()).await.is_err() {
                    break;
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
        let messages = self.messages.lock().unwrap();
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
