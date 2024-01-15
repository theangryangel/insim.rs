use std::time::Duration;

use bevy_ecs::prelude::{Event, Events, Schedule, System, World, EventWriter, EventReader};
use bevy_ecs::schedule::{IntoSystemConfigs, ScheduleLabel, Schedules};
use bevy_ecs::storage::Resources;
use bevy_ecs::system::{Resource, Res};
use tokio::time::sleep;

#[derive(Debug, Clone, Event)]
struct InsimConnected;

#[derive(Debug, Clone, Event)]
struct InsimDisconnected;

#[derive(Debug, Clone, Event)]
struct InsimShutdown;

#[derive(Debug, Clone, Event)]
struct InsimPacket(insim::packet::Packet);

#[derive(Resource, Clone, Debug)]
pub struct InsimPluginRx(flume::Receiver<insim::connection::Event>);

pub(crate) struct Plugin {
    pub(crate) config: crate::config::connection::ConnectionConfig,
}

impl crate::ecs::Plugin for Plugin {
    fn name(&self) -> &'static str {
        "Insim"
    }

    fn register(&self, ecs: &mut crate::ecs::Ecs) {
        let (tx, rx) = flume::unbounded();

        let mut conn = self.config.into_connection();

        tokio::spawn(async move {
            loop {
                match conn.poll().await {
                    Ok(e) => tx.send(e).unwrap(),
                    _ => {
                        tracing::info!("unhandled, sleeping");
                        sleep(Duration::from_secs(1)).await;
                    }
                };
            }
        });

        ecs.add_resource(InsimPluginRx(rx));
        ecs.add_system(crate::ecs::PreTick, process_insim_event);
        ecs.add_system(crate::ecs::PreTick, process_insim_connected);
        ecs.add_event::<InsimConnected>();
        ecs.add_event::<InsimDisconnected>();
        ecs.add_event::<InsimShutdown>();
        ecs.add_event::<InsimPacket>();
        ecs.add_event::<insim::insim::Mci>();
    }
}

fn process_insim_event(
    world: &mut World,
) {
    loop {

        if let Some(rx) = world.get_resource::<InsimPluginRx>() {
            match rx.0.try_recv() {
                Ok(insim::connection::Event::Data(insim::packet::Packet::MultiCarInfo(p), _)) => {
                    if let Some(mut moved) = world.get_resource_mut::<Events<insim::insim::Mci>>() {
                        moved.send(p);
                    }
                },
                Ok(insim::connection::Event::Connected(_)) => {
                    if let Some(mut moved) = world.get_resource_mut::<Events<InsimConnected>>() {
                        moved.send(InsimConnected);
                    }
                },
                Ok(insim::connection::Event::Disconnected(_)) => {
                    if let Some(mut moved) = world.get_resource_mut::<Events<InsimDisconnected>>() {
                        moved.send(InsimDisconnected);
                    }
                },
                Ok(insim::connection::Event::Shutdown(_)) => {
                    if let Some(mut moved) = world.get_resource_mut::<Events<InsimShutdown>>() {
                        moved.send(InsimShutdown);
                    }
                },
                Ok(insim::connection::Event::Error(e, _)) => {
                    panic!("{}", e);
                },
                Ok(_) => {},

                Err(flume::TryRecvError::Disconnected) => {
                    panic!("FUCK")
                },
                Err(flume::TryRecvError::Empty) => {
                    break;
                }
            }
        }
    }
}

fn process_insim_connected(mut rx: EventReader<InsimConnected>) {
    for e in rx.read() {
        println!("{:?}", e)
    }
}


pub(crate) fn process_insim_mci(mut rx: EventReader<insim::insim::Mci>) {
    for e in rx.read() {
        println!("WOOT! {:?}", e)
    }
}


