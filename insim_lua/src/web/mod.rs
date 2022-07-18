use crate::state::State;
use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
};
use axum_live_view::{html, live_view, Html, LiveViewUpgrade};
use std::collections::HashMap;
use std::sync::Arc;

mod components;

fn layout<T>(inner: Html<T>) -> Html<T> {
    html! {
        <!DOCTYPE html>
        <html>
          <head>
            <style>
            r#"
            @import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono&display=swap');
            body {
                font-family: 'JetBrains Mono', monospace;
            }
            "#
            </style>
          </head>
          <body>
            <main>
              { inner }
            </main>
            <script src="/bundle.js"></script>
          </body>
        </html>
    }
}

pub(crate) async fn index(
    Extension(state): Extension<HashMap<String, Arc<State>>>,
) -> impl IntoResponse {
    let messages = state
        .keys()
        .map(|key| key.to_string())
        .collect::<Vec<String>>();

    layout::<()>(html! {
        if messages.is_empty() {
            <p>"Its quiet, too quiet..."</p>
        } else {
            <ul>
                for msg in messages {
                    <li>
                        { &msg }
                    </li>
                }
            </ul>
        }
    })
}

pub(crate) async fn server_index(
    live: LiveViewUpgrade,
    Path(server): Path<String>,
    Extension(state): Extension<HashMap<String, Arc<State>>>,
) -> impl IntoResponse {
    let value = state.get(&server);
    // TODO throw 404
    let value = value.unwrap();

    let connections = components::connections::ConnectionComponent {
        state: value.clone(),
        tx: value.notify_on_connection(),
    };

    let messages = components::chat::ChatComponent {
        state: value.clone(),
        tx: value.notify_on_chat(),
    };

    let players = components::players::PlayersComponent {
        state: value.clone(),
        tx: value.notify_on_player(),
    };

    let combined = live_view::combine(
        (connections, messages, players),
        |connections, messages, players| {
            html! {
                <div>
                    <div>{players}</div>
                    <div>
                        {connections}
                    </div>
                    <div>
                        {messages}
                    </div>
                </div>
            }
        },
    );

    live.response(move |embed| layout(embed.embed(combined)))
}
