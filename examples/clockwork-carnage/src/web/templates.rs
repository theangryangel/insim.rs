use askama::Template;
use insim::core::track::Track;

use crate::db;
use crate::db::{Session, SessionMode, SessionStatus};
use crate::web::state::PageCtx;
use crate::web::filters;

pub struct RoundResults {
    pub round: i64,
    pub results: Vec<db::MetronomeResult>,
}

pub fn group_metronome_rounds(results: Vec<db::MetronomeResult>) -> Vec<RoundResults> {
    let mut rounds: Vec<RoundResults> = Vec::new();
    for result in results {
        if let Some(last) = rounds.last_mut() {
            if last.round == result.round {
                last.results.push(result);
                continue;
            }
        }
        rounds.push(RoundResults { round: result.round, results: vec![result] });
    }
    rounds
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub page: PageCtx,
    pub active: Option<Session>,
    pub upcoming: Vec<Session>,
}

#[derive(Template)]
#[template(path = "sessions.html")]
pub struct SessionsTemplate {
    pub page: PageCtx,
    pub sessions: Vec<Session>,
}

#[derive(Template)]
#[template(path = "session_detail.html")]
pub struct SessionDetailTemplate {
    pub page: PageCtx,
    pub session: Session,
    pub metronome_standings: Vec<db::MetronomeStanding>,
    pub metronome_rounds: Vec<RoundResults>,
    pub round_results: Vec<(i64, Vec<db::MetronomeResult>)>,
    pub shortcut_best_times: Vec<db::ShortcutTime>,
    pub shortcut_all_times: Vec<db::ShortcutTime>,
    pub bomb_best_runs: Vec<db::BombRun>,
    pub bomb_all_runs: Vec<db::BombRun>,
}

#[derive(Template)]
#[template(path = "session_new.html")]
pub struct SessionNewTemplate {
    pub page: PageCtx,
    pub tracks: &'static [Track],
}

#[derive(Template)]
#[template(path = "session_edit.html")]
pub struct SessionEditTemplate {
    pub page: PageCtx,
    pub session: db::Session,
    pub tracks: &'static [Track],
}

#[derive(Template)]
#[template(path = "partials/session_actions.html")]
pub struct SessionActionsFragment {
    pub page: PageCtx,
    pub session: Session,
}

#[derive(Template)]
#[template(path = "partials/metronome_standings_content.html")]
pub struct MetronomeStandingsContent {
    pub metronome_standings: Vec<db::MetronomeStanding>,
}

#[derive(Template)]
#[template(path = "partials/metronome_round_content.html")]
pub struct MetronomeRoundContent {
    pub round_results: Vec<db::MetronomeResult>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_standings.html")]
pub struct ShortcutStandingsFragment {
    pub session: Session,
    pub shortcut_best_times: Vec<db::ShortcutTime>,
    pub shortcut_all_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_best_times_content.html")]
pub struct ShortcutBestTimesContent {
    pub shortcut_best_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_all_times_content.html")]
pub struct ShortcutAllTimesContent {
    pub shortcut_all_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "partials/bomb_standings.html")]
pub struct BombStandingsFragment {
    pub session: Session,
    pub bomb_best_runs: Vec<db::BombRun>,
    pub bomb_all_runs: Vec<db::BombRun>,
}

#[derive(Template)]
#[template(path = "partials/bomb_best_runs_content.html")]
pub struct BombBestRunsContent {
    pub bomb_best_runs: Vec<db::BombRun>,
}

#[derive(Template)]
#[template(path = "partials/bomb_all_runs_content.html")]
pub struct BombAllRunsContent {
    pub bomb_all_runs: Vec<db::BombRun>,
}
