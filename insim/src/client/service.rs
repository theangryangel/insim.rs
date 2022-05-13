use super::{Command, Event};
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
pub trait Service {
    async fn call(&mut self, event: Event);
}

pub trait ServiceTrait: Service + Sync + Sized {}

#[derive(Debug, Clone)]
pub struct DebugService;

#[async_trait]
impl Service for DebugService {
    async fn call(&mut self, event: Event) {
        println!("{:?}", event);
    }
}

impl DebugService {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub struct SleepService;

#[async_trait]
impl Service for SleepService {
    async fn call(&mut self, _event: Event) {
        let mut i = 0;

        while i < 10 {
            println!("Sleeping zzz {:?}", i);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            println!("Woke up! {:?}", i);

            i += 1;
        }
    }
}

impl SleepService {
    pub fn new() -> Self {
        Self
    }
}
