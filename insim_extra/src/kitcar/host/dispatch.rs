//! Handing inputs to the director and services, and carrying out their
//! effects.

use std::{collections::VecDeque, time::Instant};

use insim::Packet;

use super::{Loop, Target, context};
use crate::{
    kitcar::{
        Effects,
        director::RaceDirectorMsg,
        effects::Effect,
        service::{AnyMsg, Context, Input},
    },
    timers,
    ui::Delivery,
    world::Applied,
};

impl Loop {
    pub(super) async fn on_packet(&mut self, packet: Packet) {
        let Applied { events, requests } = self.world.apply(&packet);
        // Requests the world needs to stay in step.
        for request in requests {
            if let Err(error) = self.connection.send(request).await {
                tracing::warn!(%error, "failed to send a request for the world");
            }
        }
        // The director goes first, so every service reads where it got to.
        for target in self.targets() {
            if !self.live(target) {
                continue;
            }
            self.deliver(target, Input::Packet(packet.clone())).await;
            for event in &events {
                if !self.live(target) {
                    break;
                }
                self.deliver(target, Input::World(event.clone())).await;
            }
        }
        if let Packet::Cnl(cnl) = &packet {
            // Click ids are per connection, so a reused UCID starts fresh.
            let _ = self.pools.remove(&cnl.ucid);
        }
    }

    pub(super) async fn deliver(&mut self, target: Target, input: Input<AnyMsg>) {
        let now = timers::now();
        let effects = {
            let world = &self.world;
            let cx = Context::new(world, now);
            match target {
                Target::Director => {
                    let Some(directing) = self.director.as_mut() else {
                        return;
                    };
                    let input = input.map(|msg| {
                        *msg.downcast::<RaceDirectorMsg>()
                            .expect("the host only delivers the director its own messages")
                    });
                    directing
                        .director
                        .update(input, &cx)
                        .map(|msg| Box::new(msg) as AnyMsg)
                },
                Target::Service(index) => {
                    let cx = context(world, self.director.as_ref(), now);
                    self.services[index]
                        .service
                        .update(input, &cx, &mut self.pools)
                },
            }
        };
        self.apply(target, effects, now).await;
        self.direct(now).await;
    }

    /// Hand a component's task or timer result to its screen.
    pub(super) async fn deliver_ui(&mut self, index: usize, delivery: Delivery) {
        let now = timers::now();
        let effects = {
            let cx = context(&self.world, self.director.as_ref(), now);
            self.services[index].service.deliver(delivery, &cx)
        };
        self.apply(Target::Service(index), effects, now).await;
        self.direct(now).await;
    }

    /// Hand the director the commands services just gave it.
    pub(super) async fn direct(&mut self, now: Instant) {
        for command in std::mem::take(&mut self.commands) {
            if !self.live(Target::Director) {
                tracing::warn!(?command, "no director to take this command");
                continue;
            }
            let effects = {
                let world = &self.world;
                let cx = Context::new(world, now);
                self.director
                    .as_mut()
                    .expect("checked live")
                    .director
                    .command(command, &cx)
                    .map(|msg| Box::new(msg) as AnyMsg)
            };
            self.apply(Target::Director, effects, now).await;
        }
    }

    pub(super) async fn apply(&mut self, target: Target, effects: Effects<AnyMsg>, now: Instant) {
        let mut queue: VecDeque<_> = effects.items.into();
        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::Packet(packet) => {
                    if let Err(error) = self.connection.send(packet).await {
                        tracing::warn!(%error, "failed to send a packet from a service");
                    }
                },
                Effect::Job(job) => self.slot_mut(target).timers.run(job, now),
                Effect::Send { ucid, key, msg } => {
                    // The director has no view, so nothing to send to.
                    let Target::Service(index) = target else {
                        continue;
                    };
                    let more = {
                        let cx = context(&self.world, self.director.as_ref(), now);
                        self.services[index].service.send(ucid, &*key, &*msg, &cx)
                    };
                    // Component replies run before the remaining effects.
                    for effect in more.items.into_iter().rev() {
                        queue.push_front(effect);
                    }
                },
                Effect::Stop => self.slot_mut(target).stopping = true,
                Effect::Director(command) => self.commands.push(command),
            }
        }
    }
}
