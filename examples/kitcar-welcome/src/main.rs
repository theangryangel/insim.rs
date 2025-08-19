//! A basic kitcar example

use std::time::Duration;

use insim::insim::Mtc;
use kitcar::Engine;

// User defines their own tags for type safety.
#[derive(Debug, Clone, Copy)]
enum TimerKind {
    Welcome(insim::identifiers::PlayerId),
}

struct Welcome;

impl Engine for Welcome {
    type Error = insim::Error;
    type TimerTag = TimerKind;

    fn connected(
        &mut self,
        ctx: &mut kitcar::EngineContext<Self::TimerTag>,
    ) -> Result<(), Self::Error> {
        println!("Kitcar connected!");
        Ok(())
    }

    fn disconnected(
        &mut self,
        _ctx: &mut kitcar::EngineContext<Self::TimerTag>,
    ) -> Result<(), Self::Error> {
        println!("Kitcar disconnected!");
        Ok(())
    }

    fn tick(
        &mut self,
        _ctx: &mut kitcar::EngineContext<Self::TimerTag>,
        delta: std::time::Duration,
    ) -> Result<(), Self::Error> {
        // println!("Kitcar tick! {delta:?}");
        Ok(())
    }

    fn timer(
        &mut self,
        ctx: &mut kitcar::EngineContext<Self::TimerTag>,
        tag: Self::TimerTag,
    ) -> Result<(), Self::Error> {
        match tag {
            TimerKind::Welcome(player_id) => {
                ctx.connection.write(Mtc {
                    plid: player_id,
                    text: format!("Welcome {player_id:?}"),
                    ..Default::default()
                })?;
            },
        }

        println!("Kitcar timer!");
        Ok(())
    }

    fn packet(
        &mut self,
        ctx: &mut kitcar::EngineContext<Self::TimerTag>,
        event: &insim::Packet,
    ) -> Result<(), Self::Error> {
        println!("{event:?}");
        match event {
            insim::Packet::Npl(npl) => {
                println!("Adding welcome timer");
                ctx.timers
                    .add(Duration::from_secs(5), TimerKind::Welcome(npl.plid));
            },
            _ => {},
        }
        Ok(())
    }
}

pub fn main() -> Result<(), insim::Error> {
    let con = insim::tcp("172.24.64.1:29999");
    kitcar::ignite(con, Welcome)
}
