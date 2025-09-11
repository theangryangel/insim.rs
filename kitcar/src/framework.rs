//! Framework

use std::{any::TypeId, fmt::Debug};

use insim::{identifiers::ConnectionId, Packet};
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

use crate::ui::{manager::UIManager, node::UINode};

/// Framework TaskContext
#[derive(Debug)]
pub struct TaskContext<S>
where 
    S: Send + Sync
{
    /// events
    pub events: broadcast::Receiver<Packet>,

    /// command sender
    pub commands: mpsc::Sender<Command>,

    /// user state
    pub state: S,
}

impl<S> TaskContext<S> 
where 
    S: Send + Sync {
    pub async fn send<P: Into<Packet>>(&self, packet: P) {
        self.commands.send(Command::SendPacket(packet.into())).await;
    }

    pub async fn set_ui<T: 'static>(&mut self, ucid: &ConnectionId, node: UINode) {
        let tree_id = TypeId::of::<T>();
        self.commands.send(Command::SetUi(tree_id, *ucid, node)).await;
    }

    pub async fn remove_ui<T: 'static>(&mut self, ucid: &ConnectionId) {
        let tree_id = TypeId::of::<T>();
        self.commands.send(Command::RemoveUi(tree_id, *ucid)).await;
    }
}

/// Plugin trait
#[async_trait::async_trait]
pub trait Plugin<S>: Send + Sync + Debug
where 
    S: Send + Sync + Clone
{
    /// Name useful for logging
    fn name(&self) -> &str;
    
    /// Run
    async fn run(&mut self, ctx: TaskContext<S>) -> Result<(), ()>;
}

pub(crate) enum Command {
    SendPacket(Packet),
    SetUi(
        TypeId, ConnectionId, UINode
    ),
    RemoveUi(
        TypeId, ConnectionId
    ),

}

#[derive(Debug)]
/// Framework
pub struct Framework<S> 
where 
    S: Send + Sync + Clone
{
    plugins: Vec<Box<dyn Plugin<S>>>,
}

impl<S> Framework<S>
where 
    S: Send + Sync + Clone + 'static
{
    /// New
    pub async fn new() -> Self {
        Self {
            plugins: vec![]
        }
    }
    
    /// Add plugin
    pub async fn register_plugin(mut self, name: &str, plugin: Box<dyn Plugin<S>>) -> Self {
        info!("Registering plugin: {}", name);
        self.plugins.push(plugin);
        self
    }
    
    /// Run
    pub async fn run(mut self, state: S, mut net: insim::net::tokio_impl::Framed) -> Result<(), insim::Error> {
        let (event_sender, _) = broadcast::channel::<Packet>(1000);
        let (insim_sender, mut insim_receiver) = mpsc::channel::<Command>(1000);

        info!("Starting InSim mini-game framework");
        
        // Start plugin tasks
        let mut plugin_handles = Vec::new();
        for mut plugin in self.plugins.drain(..) {
            let ctx = TaskContext {
                events: event_sender.subscribe(),
                commands: insim_sender.clone(),
                state: state.clone(),
            };
            
            let handle = tokio::spawn(async move {
                info!("Starting plugin: {}", plugin.name());
                if let Err(e) = plugin.run(ctx).await {
                    error!("Plugin {} failed: {:?}", plugin.name(), e);
                }
                warn!("Plugin {} stopped", plugin.name());
            });
            
            plugin_handles.push(handle);
        }
        
        // Main event loop
        let event_sender = event_sender.clone();
        
        loop {
            tokio::select! {
                packet = net.read() => {
                    let _ = event_sender.send(packet?).unwrap();
                },

                Some(command) = insim_receiver.recv() => match command {
                    Command::SendPacket(packet) => net.write(packet).await?,
                    Command::SetUi(type_id, connection_id, uinode) => todo!(),
                    Command::RemoveUi(type_id, connection_id) => todo!(),
                }
            }
        }
        
        // // Wait for all plugin tasks to complete
        // for handle in plugin_handles {
        //     let _ = handle.await;
        // }
        // 
        // sender_handle.abort();
        // 
        // Ok(())
    }
}
