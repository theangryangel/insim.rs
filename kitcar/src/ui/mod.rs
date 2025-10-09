//! An implementation of a retained immediate mode UI.
//! A hybrid approach that combines the programming model of immediate mode with the
//! performance optimizations of retained mode.
//! Functions are called every frame, but we diff the output to minimise the rendering
//! requirements.
//! Each plugin will be responsible for it's own set of Ui's. Nothing shared except for the id_pool.
//! `Ui` represents the ui for a single connection.
pub mod id_pool;
pub mod renderer;
pub mod styled;
pub mod vdom;

use std::collections::HashMap;

pub use id_pool::ClickIdPool;
use insim::{
    identifiers::ConnectionId,
    insim::{Bfn, BfnType, Mso, TinyType},
    Packet, WithRequestId,
};
pub use renderer::{UiRenderer, UiRendererDiff};
pub use styled::Styled;
use tokio::{sync::watch, task::JoinHandle};
pub use vdom::Element;

/// Trait for users to implement a Ui for a connection
pub trait Ui: Send + 'static {
    type State: Default + Sync + Send + Clone;
    type Signals: Clone + Sync + Send + 'static;
    type Controller: Clone + Sync + Send + 'static;

    fn render(state: &Self::State, signals: &watch::Receiver<Self::Signals>) -> Option<Element>;
    fn on_click(state: &mut Self::State, click_id: &str, controller: &Self::Controller) -> bool;
    fn on_mso(_state: &mut Self::State, _mso: &Mso, _controller: &Self::Controller) -> bool {
        false
    }
}

/// Manager to implement Ui
#[derive(Debug)]
pub struct UiManager;

impl UiManager {
    pub fn spawn<U: Ui>(
        signals: watch::Receiver<U::Signals>,
        controller: U::Controller,
        insim: insim::builder::SpawnedHandle,
    ) -> JoinHandle<insim::Result<()>> {
        tokio::spawn(async move {
            let mut packet_rx = insim.subscribe();
            let mut active = HashMap::new();

            let _ = insim.send(TinyType::Ncn.with_request_id(1)).await?;

            while let Ok(packet) = packet_rx.recv().await {
                match packet {
                    Packet::Ncn(ncn) => {
                        let _ = active.entry(ncn.ucid).or_insert_with(|| {
                            Self::spawn_player_ui::<U>(
                                ncn.ucid,
                                signals.clone(),
                                controller.clone(),
                                insim.clone(),
                            )
                        });
                    },
                    Packet::Cnl(cnl) => {
                        if let Some(handle) = active.remove(&cnl.ucid) {
                            handle.abort();
                        }
                    },
                    _ => {},
                }
            }

            // FIXME: masking the error if one occurs
            Ok(())
        })
    }

    fn spawn_player_ui<U: Ui>(
        ucid: ConnectionId,
        mut signals: watch::Receiver<U::Signals>,
        controller: U::Controller,
        insim: insim::builder::SpawnedHandle,
    ) -> JoinHandle<insim::Result<()>> {
        tokio::spawn(async move {
            let mut state = U::State::default();
            let mut renderer = UiRenderer::new(ClickIdPool::new());
            // Honor when a user blocks/requests buttons
            let mut blocked = false;

            // Initial render
            if !blocked {
                let element = U::render(&state, &signals);
                if let Some(diff) = renderer.render(element, &ucid) {
                    insim.send_all(diff.into_merged()).await?;
                }
            }

            let mut packet_rx = insim.subscribe();

            loop {
                tokio::select! {
                    // Handle button clicks
                    Ok(packet) = packet_rx.recv() => {
                        let should_render = match packet {
                            Packet::Mso(mso) => {
                                if mso.ucid != ucid {
                                    false
                                } else {
                                    U::on_mso(&mut state, &mso, &controller)
                                }
                            },

                            Packet::Btc(btc) => {
                                if_chain::if_chain! {
                                    if btc.ucid == ucid;
                                    if !blocked;
                                    if let Some(click_id) = renderer.click_id_to_key(&btc.clickid);
                                    then {
                                        U::on_click(&mut state, &click_id, &controller)
                                    } else {
                                        false
                                    }
                                }
                            },
                            Packet::Bfn(bfn) => {
                                if bfn.ucid != ucid {
                                    false
                                }
                                else if matches!(bfn.subt, BfnType::Clear | BfnType::UserClear) {
                                    blocked = true;
                                    renderer.clear();
                                    false
                                }
                                else if matches!(bfn.subt, BfnType::BtnRequest) {
                                    blocked = false;
                                    true
                                } else {
                                    false
                                }
                            },

                            _ => {
                                false
                            }
                        };

                        if !blocked && should_render {
                            let element = U::render(&state, &signals);
                            if let Some(diff) = renderer.render(element, &ucid) {
                                insim.send_all(diff.into_merged()).await?;
                            }
                        }

                    },

                    // Handle signal changes
                    _ = signals.changed() => {
                        if blocked {
                            continue;
                        }
                        let element = U::render(&state, &signals);
                        if let Some(diff) = renderer.render(element, &ucid) {
                            insim.send_all(diff.into_merged()).await?;
                        }
                    }
                }
            }
        })
    }
}
