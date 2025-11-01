/*
https://github.com/refx-online/recalculate - cleaned up
and i still hasnt handled score_status.
*/

use crate::{
    context::Context,
    models::score,
    utils::mods::{GameMods, parse_mods},
};

use anyhow::Result;
use futures::stream::StreamExt;
use redis::AsyncCommands;
use refx_pp::model::mode::GameMode;
use sqlx::Row;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

const UNRESTRICTED: i32 = 1 << 0;

#[derive(Debug)]
pub struct RecalculateMessage {
    pub user_id: i32,
}

impl RecalculateMessage {
    pub fn parse(message: &str) -> Result<Self> {
        let user_id: i32 = message
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid user_id format"))?;

        Ok(Self { user_id })
    }
}

pub struct PubSubHandler {
    ctx: Arc<Context>,
}

impl PubSubHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn start_listener(&self) -> Result<()> {
        let mut pubsub = self.ctx.redis_pubsub.lock().await;

        pubsub.subscribe("omajinai:recalculate").await?;
        info!("Subscribed to omajinai:recalculate channel");

        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            let payload: String = msg.get_payload()?;
            match RecalculateMessage::parse(&payload) {
                Ok(r) => {
                    if let Err(e) = self.handle_recalculation(r).await {
                        error!("Recalculation failed: {}", e);
                    }
                },
                Err(e) => warn!("'{}': {}", payload, e),
            }
        }
        Ok(())
    }

    async fn handle_recalculation(&self, msg: RecalculateMessage) -> Result<()> {
        info!(
            "Starting full recalculation triggered by user {}",
            msg.user_id
        );

        let mut conn = self.ctx.redis_conn.lock().await;

        for mode in [0, 1, 2, 3, 4, 5, 6, 8, 12, 16, 20] {
            if let Err(e) = self.recalculate_mode(mode).await {
                error!("Failed to recalculate mode {}: {}", mode, e);
            }
        }

        info!("Completed full recalculation.");

        // Tell bancho that recalc is done
        let _: () = conn
            .publish("refx:recalculate", msg.user_id.to_string())
            .await?;

        Ok(())
    }

    async fn recalculate_mode(&self, mode: i32) -> Result<()> {
        self.recalculate_scores(mode).await?;
        self.recalculate_users(mode).await?;

        Ok(())
    }

    async fn recalculate_scores(&self, mode: i32) -> Result<()> {
        let mut scores = sqlx::query_as::<_, score::Score>(
            // we dont talk about lazer check
            r#"
            SELECT 
                scores.score, scores.id, scores.mode, scores.mods, scores.map_md5,
                scores.pp, scores.acc, scores.max_combo,
                scores.ngeki, scores.n300, scores.nkatu, scores.n100, scores.n50, scores.nmiss,
                scores.userid,
                maps.id AS map_id,
                lazer_scores.mods_json,
                CASE 
                    WHEN lazer_scores.score_id IS NOT NULL THEN TRUE 
                    ELSE FALSE 
                END AS lazer
            FROM scores
            INNER JOIN maps ON scores.map_md5 = maps.md5
            LEFT JOIN lazer_scores ON lazer_scores.score_id = scores.id
            WHERE scores.status = 2 
              AND scores.mode = ?
            ORDER BY scores.pp DESC
            "#,
        )
        .bind(mode as u8)
        .fetch(&self.ctx.database);

        let mut updated = 0;
        while let Some(score) = scores.next().await.transpose()? {
            if let Err(e) = self.update_score_pp(&score).await {
                warn!("Failed to update score {}: {}", score.id, e);
            } else {
                updated += 1;
            }
        }

        info!("Updated {} scores for mode {}", updated, mode);
        Ok(())
    }

    async fn recalculate_users(&self, mode: i32) -> Result<()> {
        let mut users = sqlx::query("SELECT id FROM users").fetch(&self.ctx.database);

        let mut updated = 0;
        while let Some(row) = users.next().await.transpose()? {
            let user_id: i32 = row.get("id");
            if let Err(e) = self.update_user_stats(user_id, mode).await {
                warn!("Failed to update user {}: {}", user_id, e);
            } else {
                updated += 1;
            }
        }

        info!("Updated {} users for mode {}", updated, mode);
        Ok(())
    }

    async fn update_score_pp(&self, score: &score::Score) -> Result<()> {
        let beatmap = self.ctx.beatmap_service.get_beatmap(score.map_id).await?;

        let mut calculator = beatmap
            .performance()
            .combo(score.max_combo as u32)
            .n300(score.n300 as u32)
            .n100(score.n100 as u32)
            .n50(score.n50 as u32)
            .misses(score.nmiss as u32)
            .n_geki(score.ngeki as u32)
            .n_katu(score.nkatu as u32)
            .lazer(score.lazer);

        if !score.lazer {
            calculator = calculator.legacy_total_score(i64::from(score.score));
        }

        self.apply_mods_to_calculator(&mut calculator, score)?;

        let new_pp = calculator.calculate().pp();
        let new_pp = if new_pp.is_finite() { new_pp } else { 0.0 };

        sqlx::query("UPDATE scores SET pp = ? WHERE id = ?")
            .bind(new_pp)
            .bind(score.id)
            .execute(&self.ctx.database)
            .await?;

        debug!(
            "Score {} updated: {:.3}pp -> {:.3}pp",
            score.id, score.pp, new_pp
        );
        Ok(())
    }

    async fn update_user_stats(&self, user_id: i32, mode: i32) -> Result<()> {
        let best_scores: Vec<score::BestScore> = sqlx::query_as(
            "SELECT s.pp, s.acc FROM scores s 
             INNER JOIN maps m ON s.map_md5 = m.md5 
             WHERE s.userid = ? AND s.mode = ? AND s.status = 2 AND m.status IN (2, 3) 
             ORDER BY s.pp DESC",
        )
        .bind(user_id)
        .bind(mode as u8)
        .fetch_all(&self.ctx.database)
        .await?;

        if best_scores.is_empty() {
            return Ok(());
        }

        let (pp, acc) = self.calculate_weighted_stats(&best_scores);

        sqlx::query("UPDATE stats SET pp = ?, acc = ? WHERE id = ? AND mode = ?")
            .bind(pp)
            .bind(acc)
            .bind(user_id)
            .bind(mode as u8)
            .execute(&self.ctx.database)
            .await?;

        self.update_leaderboards(user_id, mode, pp).await?;

        Ok(())
    }

    fn apply_mods_to_calculator(
        &self,
        calculator: &mut refx_pp::Performance,
        score: &score::Score,
    ) -> Result<()> {
        let mode = match score.mode.rem_euclid(4) {
            0 => GameMode::Osu,
            1 => GameMode::Taiko,
            2 => GameMode::Catch,
            3 => GameMode::Mania,
            _ => return Ok(()),
        };
        let mut mods: i32 = score.mods;

        const RELAX: i32 = 1 << 7;
        // SPECIAL CASE: For refx mode, we shouldn't apply relax since
        //               the client has 2 relaxes (relax cheat and relax mod) and the player most likely
        //               cannot adapt to that relax nerf.
        if score.mode == 12 || score.mode == 16 {
            mods &= !RELAX;
        }

        let mods_str = if score.lazer {
            score
                .mods_json
                .as_ref()
                .map(|json| serde_json::to_string(&json.0).unwrap_or_default())
                .unwrap_or_else(|| score.mods.to_string())
        } else {
            mods.to_string()
        };

        // god save me
        match parse_mods(&mods_str, mode) {
            Ok(GameMods::Legacy(mods)) => {
                if score.lazer {
                    *calculator = calculator.clone().mods(mods);
                } else {
                    *calculator = calculator.clone().mods(mods.bits());
                }
            },
            Ok(GameMods::Intermode(mods)) => {
                *calculator = calculator.clone().mods(mods);
            },
            Ok(GameMods::Lazer(mods)) => {
                *calculator = calculator.clone().mods(mods);
            },
            Err(_) => unreachable!(),
        }

        Ok(())
    }

    fn calculate_weighted_stats(&self, scores: &[score::BestScore]) -> (f32, f32) {
        let total_scores = scores.len();

        let weighted_pp: f32 = scores
            .iter()
            .enumerate()
            .map(|(i, score)| score.pp * 0.95_f32.powi(i as i32))
            .sum();

        let weighted_acc: f32 = scores
            .iter()
            .enumerate()
            .map(|(i, score)| score.acc * 0.95_f32.powi(i as i32))
            .sum();

        let bonus_pp = 416.6667 * (1.0 - 0.9994_f32.powi(total_scores as i32));
        let bonus_acc = 100.0 / (20.0 * (1.0 - 0.95_f32.powi(total_scores as i32)));

        let pp = (weighted_pp + bonus_pp).round();
        let acc = (weighted_acc * bonus_acc) / 100.0;

        (pp, acc)
    }

    async fn update_leaderboards(&self, user_id: i32, mode: i32, pp: f32) -> Result<()> {
        let user_info: score::UserInfo =
            sqlx::query_as("SELECT country, priv as privs FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_one(&self.ctx.database)
                .await?;

        if (user_info.privs & UNRESTRICTED) == 0 {
            return Ok(());
        }

        let mut redis = self.ctx.redis_conn.lock().await;
        let pp_int = pp as i32;

        let _: () = redis
            .zadd(
                format!("bancho:leaderboard:{}", mode as u8),
                user_id,
                pp_int,
            )
            .await?;

        let _: () = redis
            .zadd(
                format!("bancho:leaderboard:{}:{}", mode as u8, user_info.country),
                user_id,
                pp_int,
            )
            .await?;

        Ok(())
    }
}
