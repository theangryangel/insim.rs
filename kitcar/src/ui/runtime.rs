use std::collections::HashMap;

use insim::{identifiers::ConnectionId, insim::Btn, Packet};
use tokio::{sync::{broadcast, mpsc, watch}, task::JoinHandle};

use crate::ui::{ClickIdPool, Element, Ui};

#[derive(Debug, Clone)]
pub enum UiOutput {
    Diff(ConnectionId, UiDiff),
    Click(ConnectionId, String), // ConnectionId, key
}

/// Commands sent to the UI manager task
enum UiCommand<P> {
    Mount {
        ucid: ConnectionId,
        id_pool: ClickIdPool,
        render_fn: Box<dyn Fn(&P) -> Option<Element> + Send + 'static>,
        initial_props: P,
    },
    Update {
        ucid: ConnectionId,
        props: P,
    },
    Unmount {
        ucid: ConnectionId,
    },
}

pub struct UiManager<P>
where
    P: Clone + PartialEq + Send + 'static,
{
    command_sender: mpsc::UnboundedSender<UiCommand<P>>,
    click_receiver: mpsc::UnboundedReceiver<(String, Btn)>, // key, btn
    task_handle: JoinHandle<()>,
}

impl<P> UiManager<P>
where
    P: Clone + PartialEq + Send + 'static,
{
    pub fn new<S: UserState>(ctx: Context<S>) -> Self {
        // FIXME: no unbounded
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (click_sender, click_receiver) = mpsc::unbounded_channel();
        
        let task_handle = tokio::spawn(Self::run_manager_task(
            ctx,
            command_receiver, 
            click_sender
        ));
        
        Self {
            command_sender,
            click_receiver,
            task_handle,
        }
    }

    async fn run_manager_task<S: UserState>(
        ctx: Context<S>,
        mut command_receiver: mpsc::UnboundedReceiver<UiCommand<P>>,
        click_sender: mpsc::UnboundedSender<(String, Btn)>,
    ) {
        let mut uis: HashMap<ConnectionId, (Ui<Box<dyn Fn(&P) -> Option<Element> + Send>, P>, watch::Sender<P>)> = HashMap::new();
        let mut prop_receivers: HashMap<ConnectionId, watch::Receiver<P>> = HashMap::new();
        let mut packet_receiver = ctx.subscribe_to_packets();
        
        loop {
            tokio::select! {
                // Handle commands from the public API
                Some(command) = command_receiver.recv() => {
                    match command {
                        UiCommand::Mount { ucid, id_pool, render_fn, initial_props } => {
                            let (props_sender, props_receiver) = watch::channel(initial_props.clone());
                            let mut ui = Ui::new(id_pool, ucid, render_fn);
                            
                            // Do initial render
                            if let Some(diff) = ui.render(&initial_props) {
                                for packet in diff.into_merged() {
                                    ctx.send_packet(packet).await;
                                }
                            }
                            
                            uis.insert(ucid, (ui, props_sender));
                            prop_receivers.insert(ucid, props_receiver);
                        }
                        
                        UiCommand::Update { ucid, props } => {
                            if let Some((_, props_sender)) = uis.get(&ucid) {
                                let _ = props_sender.send(props);
                            }
                        }
                        
                        UiCommand::Unmount { ucid } => {
                            uis.remove(&ucid);
                            prop_receivers.remove(&ucid);
                        }
                    }
                }
                
                // Handle incoming packets
                Ok(packet) = packet_receiver.recv() => {
                    if_chain::if_chain! {
                        if let Packet::Btn(btn) = packet;
                        if let Some((ui, _)) = uis.get(&btn.ucid);
                        if let Some(key) = ui.click_id_to_key(&btn.clickid);
                        then {
                            let _ = click_sender.send(
                                (
                                    key.to_string(),
                                    btn,
                                )
                            );
                        }
                    }
                }
                
                // Check for prop changes across all UIs
                _ = async {
                    let ucids: Vec<ConnectionId> = prop_receivers.keys().cloned().collect();
                    
                    for ucid in ucids {
                        if_chain::if_chain! {
                            if let Some(receiver) = prop_receivers.get_mut(&ucid);
                            if receiver.has_changed().unwrap_or(false);
                            if let Some((ui, _)) = uis.get_mut(&ucid);
                            then {
                                let props = receiver.borrow_and_update().clone();
                                if let Some(diff) = ui.render(&props) {
                                    for packet in diff.into_merged() {
                                        ctx.send_packet(packet).await;
                                    }
                                }
                            }
                        }
                    }
                } => {}
            }
        }
    }

    pub fn mount<F>(&self, ucid: ConnectionId, id_pool: ClickIdPool, render_fn: F, initial_props: P)
    where
        F: Fn(&P) -> Option<Element> + Send + 'static,
    {
        let command = UiCommand::Mount {
            ucid,
            id_pool,
            render_fn: Box::new(render_fn),
            initial_props,
        };
        let _ = self.command_sender.send(command);
    }

    pub fn update(&self, ucid: ConnectionId, props: P) {
        let command = UiCommand::Update { ucid, props };
        let _ = self.command_sender.send(command);
    }

    pub fn unmount(&self, ucid: ConnectionId) {
        let command = UiCommand::Unmount { ucid };
        let _ = self.command_sender.send(command);
    }

    pub async fn events(&mut self) -> Result<UiOutput, broadcast::error::RecvError> {
        self.click_receiver.recv().await
    }

    pub fn subscribe_to_output(&self) -> broadcast::Receiver<UiOutput> {
        self.click_receiver.resubscribe()
    }

    pub fn abort(&self) {
        self.task_handle.abort();
    }

    pub async fn join(self) -> Result<(), tokio::task::JoinError> {
        self.task_handle.await
    }
}
