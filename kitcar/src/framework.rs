//! Framework

use std::{any::TypeId, collections::HashMap, fmt::Debug, time::Duration};

use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::TinyType,
    Packet, WithRequestId,
};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
    plugin::{Plugin, PluginContext, UserState},
    state::{ConnectionInfo, GameInfo, PlayerInfo, State, Ui},
    ui::node::UINode,
};

pub(crate) enum Command {
    SendPacket(Packet),

    SetUi(TypeId, ConnectionId, UINode),
    RemoveUi(TypeId, ConnectionId),

    // Game state shit
    GetPlayer(PlayerId, oneshot::Sender<Option<PlayerInfo>>),
    GetPlayers(oneshot::Sender<HashMap<PlayerId, PlayerInfo>>),
    GetConnection(ConnectionId, oneshot::Sender<Option<ConnectionInfo>>),
    GetConnections(oneshot::Sender<HashMap<ConnectionId, ConnectionInfo>>),
    GetGame(oneshot::Sender<GameInfo>),

    Shutdown,
}

#[derive(Debug, thiserror::Error)]
/// Framework Error
pub enum FrameworkError {
    /// Insim error
    #[error("An insim error occured")]
    Insim(#[from] insim::Error),
}

/// Framework
pub struct Framework<S>
where
    S: Send + Sync + Clone + Debug + 'static,
{
    plugins: Vec<(String, Box<dyn Plugin<S>>)>,
    packet_channel_capacity: usize,
    command_channel_capacity: usize,
    ui_tick_rate: Duration,
}

impl<S> Debug for Framework<S>
where
    S: Send + Sync + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We can't print the plugins directly, but we can print their names.
        let plugin_names: Vec<&str> = self.plugins.iter().map(|(name, _)| name.as_str()).collect();

        f.debug_struct("Framework")
            .field("plugins", &plugin_names)
            .field("packet_channel_capacity", &self.packet_channel_capacity)
            .field("command_channel_capacity", &self.command_channel_capacity)
            .finish()
    }
}

impl<S> Framework<S>
where
    S: UserState,
{
    /// New
    pub fn new() -> Self {
        Self {
            plugins: vec![],
            packet_channel_capacity: 1000,
            command_channel_capacity: 64,
            ui_tick_rate: Duration::from_millis(500),
        }
    }

    /// Add plugin
    pub fn with_plugin<P: Plugin<S> + 'static>(mut self, plugin: P) -> Self {
        let plugin_name = std::any::type_name::<P>().to_string();
        self.plugins.push((plugin_name, Box::new(plugin)));
        self
    }

    /// Consume Framework and run
    pub async fn run(
        self,
        user_state: S,
        mut net: insim::net::tokio_impl::Framed,
    ) -> Result<(), FrameworkError> {
        // Request state from LFS
        for (i, subt) in [TinyType::Sst, TinyType::Ism, TinyType::Ncn, TinyType::Npl]
            .into_iter()
            .enumerate()
        {
            net.write(subt.with_request_id((i + 1) as u8)).await?;
        }

        let (event_sender, _) = broadcast::channel::<Packet>(self.packet_channel_capacity);
        let (command_sender, mut command_receiver) =
            mpsc::channel::<Command>(self.command_channel_capacity);

        let cancellation_token = CancellationToken::new();
        let mut state = State::default();
        let mut ui = Ui::new(self.ui_tick_rate);

        info!("Starting framework");

        // Start plugin tasks
        let mut plugin_handles = Vec::new();
        for (plugin_name, plugin) in self.plugins {
            let ctx = PluginContext {
                events: event_sender.clone(),
                commands: command_sender.clone(),
                user_state: user_state.clone(),
                cancellation_token: cancellation_token.child_token(),
            };

            let handle = tokio::spawn(async move {
                info!("Starting plugin: {:?}", plugin_name);
                if let Err(e) = plugin.run(ctx).await {
                    error!("Plugin {:?} failed: {:?}", plugin_name, e);
                }
                warn!("Plugin {:?} stopped", plugin_name);
            });

            plugin_handles.push(handle);
        }

        // Main event loop
        loop {
            tokio::select! {

                // UI update packets to send
                to_update = ui.tick() => {
                    println!("UI TICKING");
                    for p in to_update {
                        let _ = net.write(p).await?;
                    }
                },

                // packet from LFS
                packet = net.read() => {
                    let packet = packet?;

                    // we must always update our state and ui first
                    state.handle_packet(&packet);
                    ui.handle_packet(&packet);

                    if event_sender.receiver_count() > 0 {
                        let _ = event_sender.send(packet);
                    }
                },

                // commands
                Some(command) = command_receiver.recv() => match command {
                    Command::Shutdown => {
                        break;
                    },
                    Command::SendPacket(packet) => net.write(packet).await?,
                    Command::GetPlayer(player_id, sender) => {
                        let _ = sender.send(state.players.get(&player_id).cloned());
                    },
                    Command::GetPlayers(sender) => {
                        let _ = sender.send(state.players.clone());
                    },
                    Command::GetConnection(connection_id, sender) => {
                        let _ = sender.send(state.connections.get(&connection_id).cloned());
                    },
                    Command::GetConnections(sender) => {
                        let _ = sender.send(state.connections.clone());
                    },
                    Command::GetGame(sender) => {
                        let _ = sender.send(state.game.clone());
                    },
                    Command::SetUi(type_id, connection_id, view) => {
                        if let Some(mgr) = ui.inner.get_mut(&connection_id) {
                            let _ = mgr.set_tree(type_id, view);
                        }
                    },
                    Command::RemoveUi(type_id, connection_id) => {
                        if let Some(mgr) = ui.inner.get_mut(&connection_id) {
                            let _ = mgr.remove_tree(type_id);
                        }
                    }
                }
            }
        }

        cancellation_token.cancel();

        drop(event_sender);

        // Wait for all plugin tasks to complete
        for handle in plugin_handles {
            let _ = handle.await;
        }

        Ok(())
    }
}

// Where we want to be:
//
// struct AnnouncerPlugin;
//
// #[async_trait]
// impl<S> Plugin<S> for AnnouncerPlugin {
//     async fn run(mut self: Box<Self>, _ctx: TaskContext<S>) {
//         info!("Announcer Plugin started and finished its job!");
//     }
// }
//
// async fn chatterbox(mut ctx: TaskContext<()>) {
//     info!("Chatterbox plugin started!");
// }
//
// let framework = Framework::new(())
//     .with_plugin(AnnouncerPlugin)
//     .with_plugin(admin_bot)
//     .with_chat_command("!test", |_ctx: TaskContext<()>)| {
//          info!("Woot!");
//     });
//
// info!("Framework built. Running application...");
//
// let net = insim:tcp(...)...
//
// framework.run(net).await?;
