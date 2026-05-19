// BAT-001: ハッシュチェーン定期検証（Hash Chain Verifier）
//
// 実行スケジュール: 設定の `hash_chain_verify.cron` から読む（週次 月曜 03:00）。
// 全 case_id のハッシュチェーンを検証し、破断を検知した場合は tracing::error! でアラートを出す。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use chrono::{Datelike, Utc, Weekday};
use sqlx::PgPool;
use std::time::Duration;

/// BAT-001 を常駐 tokio task として実行する。
pub async fn run(write_pool: PgPool, read_pool: PgPool, cron_expr: String) {
    tracing::info!(
        bat_id = "BAT-001",
        cron = %cron_expr,
        "ハッシュチェーン検証バッチを起動しました",
    );

    loop {
        // cron 式を解析して次回実行までの待機時間を計算する
        let sleep_duration = next_sleep_duration(&cron_expr);
        tracing::info!(
            bat_id = "BAT-001",
            sleep_secs = sleep_duration.as_secs(),
            "次回ハッシュチェーン検証まで待機します",
        );
        tokio::time::sleep(sleep_duration).await;

        tracing::info!(bat_id = "BAT-001", "ハッシュチェーン全量検証を開始します");

        run_verification(&read_pool).await;

        let _ = sqlx::query(
            r#"
            INSERT INTO batch_execution_logs
                (id, bat_id, status, executed_at)
            VALUES (gen_random_uuid(), 'BAT-001', 'completed', NOW())
            "#,
        )
        .execute(&write_pool)
        .await;
    }
}

/// cron 式を解析して次回実行時刻までの Duration を計算する。
///
/// 対応パターン（5 フィールド: min hour dom month dow）:
/// - "0 3 * * 1" → 毎週月曜 03:00 UTC
/// - その他は警告ログを出して 7 日待機をフォールバックとする。
fn next_sleep_duration(cron_expr: &str) -> Duration {
    // 週次パターン "0 3 * * 1" を解析する
    let parsed = parse_weekly_cron(cron_expr);
    match parsed {
        Some((minute, hour, weekday)) => {
            duration_until_next_weekday_time(weekday, hour, minute)
        }
        None => {
            tracing::warn!(
                bat_id = "BAT-001",
                cron = %cron_expr,
                "cron 式を解析できませんでした。7 日待機をフォールバックとして使用します",
            );
            Duration::from_secs(7 * 24 * 3600)
        }
    }
}

/// cron 式（5 フィールド形式）から週次スケジュール（minute, hour, weekday）を解析する。
///
/// フォーマット: "MIN HOUR DOM MONTH DOW" where DOM="*" and MONTH="*"
/// DOW: 0=日 1=月 2=火 3=水 4=木 5=金 6=土
/// 解析できない場合は None を返す。
fn parse_weekly_cron(expr: &str) -> Option<(u32, u32, Weekday)> {
    let fields: Vec<&str> = expr.split_whitespace().collect();
    if fields.len() != 5 {
        return None;
    }

    // DOM と MONTH は "*" であることを確認する
    if fields[2] != "*" || fields[3] != "*" {
        return None;
    }

    let minute: u32 = fields[0].parse().ok()?;
    let hour: u32 = fields[1].parse().ok()?;
    let dow: u32 = fields[4].parse().ok()?;

    if minute >= 60 || hour >= 24 || dow > 6 {
        return None;
    }

    // chrono::Weekday に変換する（0=日 1=月 ... 6=土）
    let weekday = match dow {
        0 => Weekday::Sun,
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        _ => return None,
    };

    Some((minute, hour, weekday))
}

/// 次回の指定曜日・時刻（UTC）までの Duration を計算する。
fn duration_until_next_weekday_time(weekday: Weekday, hour: u32, minute: u32) -> Duration {
    let now = Utc::now();

    // 現在の曜日番号（0=Mon, 6=Sun）
    let now_weekday_num = now.weekday().num_days_from_monday();
    // 目標曜日番号（0=Mon, 6=Sun）
    let target_weekday_num = weekday.num_days_from_monday();

    // 目標曜日まで何日後か計算する
    let days_diff = if target_weekday_num > now_weekday_num {
        target_weekday_num - now_weekday_num
    } else if target_weekday_num < now_weekday_num {
        7 - (now_weekday_num - target_weekday_num)
    } else {
        // 同じ曜日: 現在時刻が目標時刻を過ぎていれば 7 日後、そうでなければ今日
        0
    };

    // 目標時刻（UTC）を構築する
    let target_naive = now
        .date_naive()
        .and_hms_opt(hour, minute, 0)
        .expect("and_hms_opt: 有効な時刻")
        + chrono::Duration::days(i64::from(days_diff));
    let target_datetime =
        chrono::DateTime::<Utc>::from_naive_utc_and_offset(target_naive, Utc);

    // 目標時刻が現在以前であれば（同曜日で時刻が過ぎている場合）7 日後に設定する
    let target_datetime = if target_datetime <= now {
        target_datetime + chrono::Duration::weeks(1)
    } else {
        target_datetime
    };

    let diff = target_datetime - now;
    // chrono::Duration を std::time::Duration に変換する（負の場合はフォールバック）
    diff.to_std().unwrap_or(Duration::from_secs(7 * 24 * 3600))
}

/// ハッシュチェーン検証の実処理
async fn run_verification(read_pool: &PgPool) {
    use chrono::Utc;
    use sqlx::Row as _;
    use wnav_hash_chain::{ChainBlock, verify_chain};

    let case_ids: Vec<uuid::Uuid> = match sqlx::query_scalar(
        r#"SELECT DISTINCT case_id FROM work_event_blocks ORDER BY case_id"#,
    )
    .fetch_all(read_pool)
    .await
    {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!(bat_id = "BAT-001", error = %e, "case_id 一覧の取得に失敗しました");
            return;
        }
    };

    let mut verified_count: i64 = 0;
    let mut broken_count: i64 = 0;

    for case_id in &case_ids {
        let blocks = match sqlx::query(
            r#"
            SELECT id, case_id, sequence_number, prev_block_hash, content_hash, block_hash, created_at
            FROM work_event_blocks
            WHERE case_id = $1
            ORDER BY sequence_number ASC
            "#,
        )
        .bind(case_id)
        .fetch_all(read_pool)
        .await
        {
            Ok(b) => b,
            Err(e) => {
                tracing::error!(bat_id = "BAT-001", case_id = %case_id, error = %e, "ブロック取得に失敗しました");
                continue;
            }
        };

        let chain_blocks: Vec<ChainBlock> = blocks
            .iter()
            .map(|b| {
                let prev: Vec<u8> = b.get("prev_block_hash");
                let content: Vec<u8> = b.get("content_hash");
                let block: Vec<u8> = b.get("block_hash");
                let created_at: chrono::DateTime<Utc> = b.get("created_at");
                ChainBlock {
                    id: b.get("id"),
                    case_id: b.get("case_id"),
                    sequence_number: b.get("sequence_number"),
                    prev_block_hash: prev.try_into().unwrap_or([0u8; 32]),
                    content_hash: content.try_into().unwrap_or([0u8; 32]),
                    block_hash: block.try_into().unwrap_or([0u8; 32]),
                    created_at,
                }
            })
            .collect();

        verified_count += chain_blocks.len() as i64;

        if let Err(e) = verify_chain(&chain_blocks) {
            broken_count += 1;
            tracing::error!(
                bat_id = "BAT-001",
                case_id = %case_id,
                error = %e,
                "ハッシュチェーン破断を検知しました！改ざんの可能性があります",
            );
        }
    }

    tracing::info!(
        bat_id = "BAT-001",
        verified_count = verified_count,
        broken_count = broken_count,
        "ハッシュチェーン全量検証が完了しました",
    );

    if broken_count > 0 {
        tracing::error!(
            bat_id = "BAT-001",
            broken_count = broken_count,
            "ハッシュチェーン破断が検知されました。即時調査が必要です",
        );
    }
}
