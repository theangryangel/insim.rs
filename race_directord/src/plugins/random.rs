use bevy_ecs::prelude::{Event, EventReader, EventWriter};
use crate::ecs;

// This is our event that we will send and receive in systems
#[derive(Event)]
pub(crate) struct RandomChanceEvent {
    pub message: String,
    pub random_value: f32,
}

pub(crate) struct Plugin;

impl crate::ecs::Plugin for Plugin {
    fn name(&self) -> &'static str {
        "MyEvent"
    }

    fn register(&self, ecs: &mut ecs::Ecs) {
        ecs.add_event::<RandomChanceEvent>();

        ecs.add_system(ecs::PostTick, sending_system);
        ecs.add_system(ecs::Tick, receiving_system);
    }
}

// In every frame we will send an event with a 50/50 chance
fn sending_system(mut event_writer: EventWriter<RandomChanceEvent>) {
    let random_value: f32 = rand::random();
    if random_value > 0.5 {
        event_writer.send(RandomChanceEvent {
            message: "A random event with value > 0.5".to_string(),
            random_value,
        });
    }
}

// This system listens for events of the type MyEvent
// If an event is received it will be printed to the console
fn receiving_system(mut event_reader: EventReader<RandomChanceEvent>) {
    for my_event in event_reader.read() {
        println!(
            "Received message {:?}, with random value of {}",
            my_event.message, my_event.random_value
        );
    }
}

