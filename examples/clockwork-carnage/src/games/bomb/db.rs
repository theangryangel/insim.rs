use super::state::ActiveRun;

pub(super) async fn persist_bomb_run(
    db: &Option<(crate::db::Pool, i64)>,
    run: &ActiveRun,
    survival_ms: i64,
) {
    if let Some((pool, event_id)) = db {
        if let Err(e) = crate::db::insert_bomb_run(
            pool,
            *event_id,
            &run.uname,
            "",
            run.checkpoints,
            survival_ms,
        )
        .await
        {
            tracing::warn!("Failed to persist bomb run: {e}");
        }
    }
}
