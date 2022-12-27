pub(crate) mod templating;

use crate::state::{chat::Chat, Connection, Game};
use axum::{
    extract::Path,
    response::{sse, Html, IntoResponse, Response},
    Extension,
};
use insim::file::pth::Pth;
use serde::Serialize;

use miette::Result;
use minijinja::context;
use std::sync::Arc;

use crate::insim_manager::Manager;

pub(crate) async fn servers_index(
    tmpl: Extension<templating::Engine>,
    manager: Extension<Arc<Manager>>,
) -> impl IntoResponse {
    #[allow(clippy::map_clone)]
    let servers = manager
        .instances
        .keys()
        .map(|e| e.clone())
        .collect::<Vec<String>>();

    let res = tmpl
        .render(
            "hello.html",
            context! {
                name => servers
            },
        )
        .unwrap(); // FIXME
    Html(res)
}

pub(crate) async fn servers_show(
    Path(server): Path<String>,
    tmpl: Extension<templating::Engine>,
    manager: Extension<Arc<Manager>>,
) -> impl IntoResponse {
    let servers = &manager.instances.get(&server).unwrap().state;

    let res = tmpl
        .render(
            "servers_show.html",
            context! {
                players => servers.get_players(true),
                connections => servers.get_connections(),
                name => &server,
                chat => servers.chat().iter().cloned().collect::<Vec<Chat>>(),
                player_count => servers.get_player_count(),
                connection_count => servers.get_connection_count(),
                game => servers.game(),
            },
        )
        .unwrap(); // FIXME
    Html(res)
}

#[derive(Serialize)]
struct Live {
    pub game: Game,
    pub players: Vec<Connection>,
}

pub(crate) async fn servers_live(
    Path(server): Path<String>,
    tmpl: Extension<templating::Engine>,
    manager: Extension<Arc<Manager>>,
) -> sse::Sse<impl futures::stream::Stream<Item = Result<sse::Event, std::convert::Infallible>>> {
    let s = &manager.instances.get(&server).unwrap().state;
    let s = s.clone();

    let stream = async_stream::stream! {
        loop {

            let notify_on_player = s.notify_on_player();
            let notify_on_chat = s.notify_on_chat();

            tokio::select! {
                _ = notify_on_player.notified() => {
                    let res = tmpl
                    .render("servers_info.html", context! {
                        players => (s.get_players(true)),
                        connections => (s.get_connections()),
                        name => "",
                    }).unwrap();

                    yield Ok(sse::Event::default().event("players").data(res));

                    let game = Live {
                        game: s.game(),
                        players: s.get_players(true),
                    };

                    yield Ok(
                        sse::Event::default().event("players_json").json_data(&game).unwrap()
                    );
                },

                _ = notify_on_chat.notified() => {

                    let res = tmpl
                    .render("servers_chat.html", context! {
                        chat => s.chat().iter().cloned().collect::<Vec<Chat>>(),
                    }).unwrap();

                    yield Ok(sse::Event::default().event("message").data(res));
                }

            }

        }
    };

    sse::Sse::new(stream)
}

const DEFAULT_SCALE: f32 = 65536.0;

pub(crate) async fn track_map(
    Path(server): Path<String>,
    manager: Extension<Arc<Manager>>,
) -> impl IntoResponse {
    let s = &manager.instances.get(&server).unwrap().state;

    let mut document = svg::Document::new();

    let mut all_pth_nodes = Vec::new();

    let mut viewbox_x: (f32, f32) = (0.0, 0.0);
    let mut viewbox_y: (f32, f32) = (0.0, 0.0);

    let background_colour = String::from("#3D9970");
    let track_colour = String::from("#111111");
    let viewbox_padding = 10.0;
    let resolution = 2.0;

    // FIXME this is all shit
    //
    let track = s.game().track.unwrap();

    let path_glob = std::path::Path::new(
        // FIXME this should maybe be build time?
        "./assets",
    )
    .join(format!("{}*.pth", track.family(),))
    .as_path()
    .display()
    .to_string();

    // FIXME, quick hack to force the *current* track to be last

    let mut paths: Vec<std::path::PathBuf> = glob::glob(&path_glob)
        .unwrap()
        .map(|c| c.expect("wheres the path"))
        .collect();

    paths.sort_by(|a, _b| {
        if a.file_stem().unwrap().to_string_lossy() == track.code {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });

    for i in paths {
        let p = Pth::from_file(&i).unwrap();

        // wrap around the nodes to avoid missing "notches" in the track drawing
        let mut nodes = p.nodes.clone();
        nodes.insert(0, *nodes.last().unwrap());
        nodes.iter_mut().for_each(|c| {
            c.center = c.center.flipped();
            c.direction = c.direction.flipped();
        });

        all_pth_nodes.push(nodes);
    }

    // draw all the track limits first, a single polygon per PTH
    // to avoid the "gaps" issue
    for nodes in all_pth_nodes.iter() {
        let mut fwd = Vec::with_capacity(nodes.len() * 2);
        let mut bck = Vec::with_capacity(nodes.len());

        let mut i = 1.0;

        for node in nodes.iter() {
            let limits = node.get_outer_limit(Some(DEFAULT_SCALE));

            viewbox_x.0 = viewbox_x.0.min(limits.0.x);
            viewbox_x.0 = viewbox_x.0.min(limits.1.x);

            viewbox_x.1 = viewbox_x.1.max(limits.0.x);
            viewbox_x.1 = viewbox_x.1.max(limits.1.x);

            viewbox_y.0 = viewbox_y.0.min(limits.0.y);
            viewbox_y.0 = viewbox_y.0.min(limits.1.y);

            viewbox_y.1 = viewbox_y.1.max(limits.0.y);
            viewbox_y.1 = viewbox_y.1.max(limits.1.y);

            if i % resolution == 0.0 {
                fwd.push((limits.0.x, limits.0.y));
                bck.push((limits.1.x, limits.1.y));
            }

            i += 1.0;
        }

        fwd.extend(bck.iter().rev());

        let poly = svg::node::element::Polygon::new()
            .set("style", format!("fill: {}", background_colour))
            .set(
                "points",
                fwd.iter()
                    .map(|i| format!("{},{}", i.0, i.1))
                    .collect::<Vec<String>>()
                    .join(" "),
            );

        document = document.add(poly);
    }

    // draw all the roads next
    for nodes in all_pth_nodes.iter() {
        let mut fwd = Vec::with_capacity(nodes.len() * 2);
        let mut bck = Vec::with_capacity(nodes.len());
        let mut i = 1.0;

        for node in nodes.iter() {
            if i % resolution == 0.0 {
                let limits = node.get_road_limit(Some(DEFAULT_SCALE));
                fwd.push((limits.0.x, limits.0.y));
                bck.push((limits.1.x, limits.1.y));
            }

            i += 1.0;
        }

        fwd.extend(bck.iter().rev());

        let points = fwd
            .iter()
            .map(|i| format!("{},{}", i.0, i.1))
            .collect::<Vec<String>>()
            .join(" ");

        let poly = svg::node::element::Polygon::new()
            .set("style", format!("fill: {}", track_colour))
            .set("points", points);

        document = document.add(poly);
    }

    document = document.set(
        "viewBox",
        (
            viewbox_x.0 - viewbox_padding,
            viewbox_y.0 - viewbox_padding,
            (viewbox_x.1 + viewbox_padding) - (viewbox_x.0 - viewbox_padding),
            (viewbox_y.1 + viewbox_padding) - (viewbox_y.0 - viewbox_padding),
        ),
    );

    Response::builder()
        .header("Content-Type", "image/svg+xml")
        .body(document.to_string())
        .unwrap()
}
