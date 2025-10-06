//! 20s league
use std::{collections::HashMap, time::Duration};

use insim::{identifiers::{ConnectionId, PlayerId}, insim::{Mst, Mtc, TinyType}, Packet, WithRequestId};
use kitcar::{leaderboard::{Leaderboard, LeaderboardHandle}, presence::{Presence, PresenceHandle}, time::countdown::Countdown, ui::{ClickIdPool, Element, Ui, UiDiff}, utils::NoVote, Service, State as _};
use tokio::{sync::{broadcast, mpsc}, time::sleep};
use anyhow::Result;

use crate::components::countdown;

const ROUNDS_PER_GAME: usize = 5;
const ROUND_DURATION: u64 = 60;
const TARGET_TIME: f32 = 20.0;

mod components;

#[tokio::main]
async fn main() -> Result<()> {
    let mut insim = insim::tcp("172.24.64.1:29999").connect_async().await?;
    
    let (packet_tx, _) = broadcast::channel(100);
    let (send_packet_tx, mut send_packet_rx) = mpsc::channel::<Packet>(100);
    
    // Spawn library handles
    let presence = Presence::spawn(packet_tx.subscribe());
    let leaderboard = Leaderboard::spawn(packet_tx.subscribe());
    NoVote::spawn(packet_tx.subscribe(), send_packet_tx.clone());
    
    let _ = insim.write(TinyType::Ncn.with_request_id(1)).await;
    let _ = insim.write(TinyType::Npl.with_request_id(2)).await;
    
    let mut game = GameState::new(leaderboard, presence, send_packet_tx);

    let packet_tx2 = packet_tx.clone();

    // Spawn packet reader
    // TODO: move into kitcar?
    let _ = tokio::spawn(async move {
        loop {
            tokio::select! {
                packet = insim.read() => match packet {
                    Ok(packet) => { 
                        // FIXME: handle result
                        let _ = packet_tx2.send(packet);
                    }
                    Err(e) => { eprintln!("Error: {}", e); break; }
                },
                packet = send_packet_rx.recv() => match packet {
                    Some(packet) => {
                        // FIXME: handle result
                        let _ = insim.write(packet).await;
                    },
                    None => {
                        // FIXME
                    }
                }
            }
        }
    });
    
    println!("20 Second League started!");
    
    loop {
        game.wait_for_players().await?;
        game.run(packet_tx.clone()).await?;
        game.show_leaderboard(true).await?;
        
        sleep(Duration::from_secs(30)).await;
        game.reset().await?;
    }
}

struct GameState {
    round_scores: HashMap<PlayerId, Duration>,
    send_packet_tx: mpsc::Sender<Packet>,

    leaderboard: LeaderboardHandle,
    presence: PresenceHandle,

    /// TODO: should be a service handle
    views: HashMap<ConnectionId, Ui<fn(&Duration) -> Option<Element>, Duration>>,
}

impl GameState {
    fn new(
        leaderboard: LeaderboardHandle,
        presence: PresenceHandle,
        send_packet_tx: mpsc::Sender<Packet>,
    ) -> Self {
        Self {
            round_scores: HashMap::new(),
            send_packet_tx,
            leaderboard,
            presence,
            views: HashMap::new(),
        }
    }
    
    async fn reset(&mut self) -> Result<()> {
        self.leaderboard.clear().await;
        self.round_scores.clear();
        Ok(())
    }
    
    async fn wait_for_players(
        &self,
    ) -> Result<()> {
        loop {
            let count = self.presence.player_count().await;
            
            if count >= 1 {
                // ui.show_message_all("Starting in 10s...", Duration::from_secs(10)).await?;
                self.message_all("Starting in 10s").await;
                sleep(Duration::from_secs(2)).await;
                return Ok(());
            }
            
            self.message_all("Waiting for players...").await;
            // ui.show_message_all(&format!("Waiting ({}/2)", count), Duration::from_secs(5)).await?;
            sleep(Duration::from_secs(5)).await;
        }
    }
    
    async fn run(
        &mut self,
        game_tx: broadcast::Sender<Packet>,
    ) -> Result<()> {
        for round in 1..=ROUNDS_PER_GAME {
            let mut game_rx = game_tx.subscribe();

            self.command(&format!("/restart")).await;

            println!("Starting round {}/{}", round, ROUNDS_PER_GAME);
            
            self.round_scores.clear();

            loop {
                match game_rx.recv().await {
                    Ok(Packet::Rst(_)) => {
                        // TODO: how do we prevent this from causing issues
                        // its convenient. do we care?
                        sleep(Duration::from_secs(11)).await;
                        break;
                    },
                    p => {
                        println!("{:?}", p);
                    }
                }
            }

            // ui.show_message_all(
            //     &format!("Round {}/{} - Get close to 20s!", round, ROUNDS_PER_GAME),
            //     Duration::from_secs(5),
            // ).await?;
            self.message_all(&format!("Round {}/{} - Get close to 20s!", round, ROUNDS_PER_GAME)).await;
            
            self.run_round(&mut game_rx).await?;
            self.score_round(10).await;
            
            // ui.show_message_all(&format!("Round {} complete!", round), Duration::from_secs(3)).await?;
            self.message_all(&format!("Round {} complete!", round)).await;
            self.show_leaderboard(false).await?;
        }
        
        Ok(())
    }
    
    async fn run_round(
        &mut self,
        game_rx: &mut broadcast::Receiver<Packet>,
    ) -> Result<()> {
        let mut countdown = Countdown::new(Duration::from_secs(1), 30);

        loop {
            tokio::select! {
                remaining = countdown.tick() => match remaining {
                    Some(_) => {
                        self.message_all(&format!("{:?}s remaining!", countdown.remaining_duration().await)).await;
                        // FIXME: proof of concept
                        for conninfo in self.presence.connections().await.unwrap_or_default() {
                            for btn in self.ui(&conninfo.ucid, &countdown.remaining_duration().await).await.unwrap_or_default().into_merged() {
                                println!("{:?}", btn);
                                let _ = self.send_packet_tx.send(btn).await;
                            }
                        }
                        // ui.show_message_all(&format!("{}s remaining!", secs), Duration::from_secs(1)).await?;
                    },
                    None => {
                        break;
                    }
                },
                packet = game_rx.recv() => match packet {
                    Ok(packet) => { println!("{:?}", packet); self.handle_packet(&packet).await?; },
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_packet(
        &mut self,
        packet: &Packet,
    ) -> Result<()> {
        match packet {
            Packet::Fin(fin) => {
                let _ = self.round_scores.insert(
                    fin.plid,
                    fin.ttime,
                );
            }
            Packet::Ncn(ncn) => {
                self.message(&ncn.ucid, "Welcome to 20 Second League! Get as close to 20s as possible.").await;
                // ui.show_message(
                //     join.plid,
                //     "Welcome to 20 Second League! Get as close to 20s as possible.",
                //     Duration::from_secs(10),
                // ).await?;
            },
            Packet::Pll(pll) => {
                // FIXME: probably unfair, but fuck it for now
                let _ = self.round_scores.remove(&pll.plid);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn score_round(&mut self, max: usize) {
        let mut ordered = self.round_scores.drain().map(|(k, v)| {
            (k, Duration::from_secs(TARGET_TIME as u64).abs_diff(v))
        }).collect::<Vec<(PlayerId, Duration)>>();

        ordered.sort_by(|a,b| {
            a.1.cmp(&b.1)
        });

        for (i, (plid, delta)) in ordered.into_iter().take(max).enumerate() {
            let points = max - i;
            let _ = self.leaderboard.add_player_score(&plid, points as i32).await;
            println!("Player {} scored {} points (delta: {:?})", plid, points, delta);
        }
    }
    
    async fn show_leaderboard(&self, finished: bool) -> Result<()> {
        let rankings = self.leaderboard.ranking(Some(10)).await;

        self.message_all("=== Leaderboard ===").await;
        for (i, (plid, score)) in rankings.iter().enumerate() {
            // TODO: shoudl really collect the playerinfo up front, but whatever
            if let Some(playerinfo) = self.presence.player(plid).await {
                self.message_all(
                    &format!("{}. Player {} - {} pts\n", i + 1, playerinfo.pname, score)
                ).await;
            }
        }

        if finished {
            if let Some((winner_plid, winner_score)) = rankings.first() {
                self.message_all(
                    &format!("Winner: Player {} with {} points!", winner_plid, winner_score),
                ).await;
            }
        }

        // ui.show_message_all(&leaderboard_text, Duration::from_secs(5)).await?;
        
        Ok(())
    }

    async fn message_all(&self, msg: &str) {
        println!("{}", msg);
        let _ = self.send_packet_tx.send(Mtc {
            ucid: ConnectionId::ALL,
            text: msg.to_string(),
            ..Default::default()
        }.into()).await;
    }

    async fn message(&self, ucid: &ConnectionId, msg: &str) {
        println!("{}", msg);
        let _ = self.send_packet_tx.send(Mtc {
            ucid: *ucid,
            text: msg.to_string(),
            ..Default::default()
        }.into()).await;
    }

    async fn command(&self, command: &str) {
        println!("{}", command);

        let _ = self.send_packet_tx.send(Mst {
            msg: command.to_string(),
            ..Default::default()
        }.into()).await;
    }

    async fn ui(&mut self, ucid: &ConnectionId, duration: &Duration) -> Option<UiDiff> {
        let interface = self.views.entry(*ucid).or_insert(
            Ui::new(ClickIdPool::new(), *ucid, countdown)
        );

        interface.render(&duration)
    }
}
