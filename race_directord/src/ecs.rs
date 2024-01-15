use bevy_ecs::prelude::{Event, Events, Schedule, System, World};
use bevy_ecs::schedule::{IntoSystemConfigs, ScheduleLabel, Schedules};
use bevy_ecs::system::Resource;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Startup;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreTick;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tick;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostTick;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shutdown;

pub(crate) trait Plugin {
    fn name(&self) -> &'static str;
    fn register(&self, ecs: &mut Ecs);
}

pub(crate) struct Ecs {
    world: World,
}

impl Ecs {
    pub(crate) fn new() -> Self {
        let mut world = World::new();
        world.add_schedule(Schedule::new(Startup));
        world.add_schedule(Schedule::new(PreTick));
        world.add_schedule(Schedule::new(Tick));
        world.add_schedule(Schedule::new(PostTick));
        world.add_schedule(Schedule::new(Shutdown));

        Self { world }
    }

    pub(crate) fn add_system<M>(
        &mut self,
        label: impl ScheduleLabel,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        let schedule = label.intern();
        let mut schedules = self.world.resource_mut::<Schedules>();

        if let Some(schedule) = schedules.get_mut(schedule) {
            schedule.add_systems(systems);
        } else {
            let mut new_schedule = Schedule::new(schedule);
            new_schedule.add_systems(systems);
            schedules.insert(new_schedule);
        }

        self
    }

    pub(crate) fn add_event<T>(&mut self) -> &mut Self
    where
        T: Event,
    {
        if !self.world.contains_resource::<Events<T>>() {
            self.world.init_resource::<Events<T>>();
            self.add_system(
                PreTick,
                bevy_ecs::event::event_update_system::<T>
                    .run_if(bevy_ecs::event::event_update_condition::<T>),
            );
        }
        self
    }

    pub(crate) fn add_plugin(&mut self, p: impl Plugin) -> &mut Self {
        p.register(self);
        println!("registering {:?}", p.name());
        self
    }

    pub(crate) fn add_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    fn run(&mut self, label: impl ScheduleLabel) {
        // FIXME
        self.world.try_run_schedule(label).unwrap();
    }

    pub(crate) fn startup(&mut self) {
        self.run(Startup);
    }

    pub(crate) fn shutdown(&mut self) {
        self.run(Shutdown);
    }

    pub(crate) fn tick(&mut self) {
        self.run(PreTick);
        self.run(Tick);
        self.run(PostTick);
    }
}
