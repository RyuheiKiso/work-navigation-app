//! デモシード投入サブコマンド
//!
//! 対応 §: ロードマップ §14.2 §10.5.1 §10.2.1
//!
//! preset (`minimal` / `showcase`) に応じて adapter の upsert API を呼び出す。
//! SQL は本ファイルでは一切書かない（書込みはすべて adapter 層に閉じる）。
//! upsert ベースのため再実行は冪等で、`make demo` 連打でも DB が壊れない。

use anyhow::{Context, Result};
use clap::ValueEnum;
use sqlx::PgPool;
use tracing::info;

use wna_adapter::{
    Argon2idHasher, PostgresCredentialRepository, PostgresMasterRepository, PostgresRepository,
    TaskStepRow,
};
use wna_domain::{
    CompletionCriteria, DeviceId, LamportTimestamp, PasswordHasher, Task, TaskId, TaskRepository,
    TaskState,
};

use super::seed_data::{
    DemoStep, DemoTask, EQUIPMENTS, FLOWS, PARTS, PRODUCTS, SEED_PASSWORD, STEPS, TASKS, USERS,
};

/// シードプリセット
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SeedPreset {
    /// ユーザ 3 名のみ（開発時の手元動作確認）
    Minimal,
    /// マスタ＋フロー＋進行中タスクまで含むデモ向けセット
    Showcase,
}

/// シードを実行する
///
/// # Errors
/// パスワードハッシュ生成失敗、DB 書き込み失敗、ドメイン値構築失敗等で `Err` を返す。
pub async fn run(preset: SeedPreset, pool: &PgPool) -> Result<()> {
    info!(preset = ?preset, "デモシードを開始します");

    let cred_repo = PostgresCredentialRepository::new(pool.clone());
    let master_repo = PostgresMasterRepository::new(pool.clone());
    let task_repo = PostgresRepository::new(pool.clone());
    let hasher = Argon2idHasher::new();

    seed_users(&cred_repo, &hasher).await?;

    if preset == SeedPreset::Showcase {
        seed_master(&master_repo).await?;
        seed_flows(&master_repo).await?;
        seed_tasks(&task_repo, &master_repo).await?;
    }

    info!(preset = ?preset, "デモシードが完了しました");
    Ok(())
}

/// ユーザを upsert する
async fn seed_users(repo: &PostgresCredentialRepository, hasher: &Argon2idHasher) -> Result<()> {
    for user in USERS {
        // パスワードは固定平文を Argon2id でハッシュ化（毎回ソルト変動するため値が同じでも upsert 可）
        let phc = hasher
            .hash(SEED_PASSWORD)
            .map_err(|e| anyhow::anyhow!("パスワードハッシュに失敗: {e}"))?;
        repo.upsert_credential(user.user_id, user.display_name, phc.as_str(), true)
            .await
            .with_context(|| format!("credentials upsert: {}", user.user_id))?;
    }
    info!(count = USERS.len(), "credentials を投入しました");
    Ok(())
}

/// 製品／設備／部材の各マスタを upsert する
async fn seed_master(repo: &PostgresMasterRepository) -> Result<()> {
    for p in PRODUCTS {
        repo.upsert_product(p.code, p.name, p.extra)
            .await
            .with_context(|| format!("products upsert: {}", p.code))?;
    }
    for e in EQUIPMENTS {
        repo.upsert_equipment(e.code, e.name, e.extra)
            .await
            .with_context(|| format!("equipments upsert: {}", e.code))?;
    }
    for p in PARTS {
        repo.upsert_part(p.code, p.name, p.extra)
            .await
            .with_context(|| format!("parts upsert: {}", p.code))?;
    }
    info!(
        products = PRODUCTS.len(),
        equipments = EQUIPMENTS.len(),
        parts = PARTS.len(),
        "マスタを投入しました"
    );
    Ok(())
}

/// フローを upsert する
async fn seed_flows(repo: &PostgresMasterRepository) -> Result<()> {
    for f in FLOWS {
        repo.upsert_flow(f.id, f.version, f.name, f.industry, f.status, f.body_json)
            .await
            .with_context(|| format!("flows upsert: {}", f.id))?;
    }
    info!(count = FLOWS.len(), "flows を投入しました");
    Ok(())
}

/// タスク本体（Aggregate）と表示メタ・ステップを投入する
async fn seed_tasks(
    task_repo: &PostgresRepository,
    master_repo: &PostgresMasterRepository,
) -> Result<()> {
    for t in TASKS {
        let task = build_task_aggregate(t)?;
        // 1) Aggregate ルートを永続化（state / device_id / lamport / completion_criteria）
        task_repo
            .save(&task)
            .await
            .with_context(|| format!("tasks save: {}", t.id))?;
        // 2) 表示メタ（title / flow_id / responsible_user）を反映
        master_repo
            .update_task_meta(t.id, Some(t.title), Some(t.flow_id), Some(t.responsible_user))
            .await
            .with_context(|| format!("tasks meta: {}", t.id))?;
        // 3) 現在ステップ（None で OK）
        master_repo
            .update_current_step(t.id, t.current_step_id)
            .await
            .with_context(|| format!("tasks current_step: {}", t.id))?;
        // 4) ステップを upsert（done フラグは現在ステップ手前まで true）
        for step in STEPS {
            let row = build_step_row(t, step);
            master_repo
                .upsert_step(t.id, &row)
                .await
                .with_context(|| format!("task_steps upsert: {}/{}", t.id, step.id))?;
        }
    }
    info!(tasks = TASKS.len(), steps_per_task = STEPS.len(), "tasks/task_steps を投入しました");
    Ok(())
}

/// `DemoTask` から `Task` Aggregate を rehydrate する
fn build_task_aggregate(t: &DemoTask) -> Result<Task> {
    let task_id = TaskId::new(t.id.to_string()).context("TaskId")?;
    let device_id = DeviceId::new(t.device_id.to_string()).context("DeviceId")?;
    let lamport = LamportTimestamp::from_u64(t.lamport);
    let cri = parse_criteria(t.completion_criteria)?;
    let state = TaskState::from_label(t.state_label)
        .ok_or_else(|| anyhow::anyhow!("未知の TaskState ラベル: {}", t.state_label))?;
    // Idle 以外は前提条件充足済みとみなす（postgres_repository の復元規則と整合）
    let precondition_satisfied = !matches!(state, TaskState::Idle);
    Ok(Task::rehydrate(task_id, state, cri, device_id, lamport, precondition_satisfied))
}

/// 文字列タグから `CompletionCriteria` を構築する
fn parse_criteria(tag: &str) -> Result<CompletionCriteria> {
    match tag {
        "manual" => Ok(CompletionCriteria::Manual),
        "photo" => Ok(CompletionCriteria::Photo),
        other => Err(anyhow::anyhow!("未知の completion_criteria: {other}")),
    }
}

/// タスクの現在ステップ位置から、各ステップの done フラグを決定する
fn build_step_row(t: &DemoTask, step: &DemoStep) -> TaskStepRow {
    // 「現在ステップより前 = done」とする見栄え用ロジック。
    // current_step_id 未設定（None）の場合は全ステップ未完。
    // Completed タスクは全ステップ done として扱う。
    let done = if t.state_label == "Completed" {
        true
    } else {
        match (t.current_step_id, current_step_sequence(t)) {
            (Some(_), Some(cur)) => step.sequence < cur,
            _ => false,
        }
    };
    TaskStepRow {
        id: step.id.to_string(),
        sequence: step.sequence,
        label: step.label.to_string(),
        completion_criteria: step.completion_criteria.to_string(),
        standard_time_seconds: step.standard_time_seconds,
        done,
    }
}

/// 現在ステップ ID から sequence を逆引きする
fn current_step_sequence(t: &DemoTask) -> Option<i32> {
    let cur_id = t.current_step_id?;
    STEPS.iter().find(|s| s.id == cur_id).map(|s| s.sequence)
}
