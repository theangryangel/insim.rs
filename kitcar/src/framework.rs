//! Framework

use std::fmt::Debug;

use insim::Packet;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

/// Framework TaskContext
#[derive(Debug)]
pub struct TaskContext<S>
where 
    S: Send + Sync
{
    /// events
    pub events: broadcast::Receiver<Packet>,
    /// insim sender
    pub insim_sender: mpsc::Sender<Packet>,

    /// user state
    pub state: S,
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
    pub async fn new() -> Self {
        Self {
            plugins: vec![]
        }
    }
    
    pub async fn register_plugin(mut self, name: &str, plugin: Box<dyn Plugin<S>>) -> Self {
        info!("Registering plugin: {}", name);
        self.plugins.push(plugin);
        self
    }
    
    pub async fn run(mut self, state: S, mut net: insim::net::tokio_impl::Framed) -> Result<(), insim::Error> {
        let (event_sender, _) = broadcast::channel::<Packet>(1000);
        let (insim_sender, mut insim_receiver) = mpsc::channel::<Packet>(1000);

        info!("Starting InSim mini-game framework");
        
        // Start plugin tasks
        let mut plugin_handles = Vec::new();
        for mut plugin in self.plugins.drain(..) {
            let ctx = TaskContext {
                events: event_sender.subscribe(),
                insim_sender: insim_sender.clone(),
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

                Some(packet) = insim_receiver.recv() => {
                    net.write(packet).await?;
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
