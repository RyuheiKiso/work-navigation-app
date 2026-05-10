//! 対応 §: ロードマップ §7.1 §10.1 §10.6 §11.4.2 §27 F-008
//!
//! Tauri command の最小実装。React 側の `TauriTaskRepository` から呼ばれる
//! `get_task` / `save_task` を提供する。
//!
//! SQLite／SQLCipher 統合は `secure_storage` モジュールを参照する（feature = "sqlcipher" で有効化）。

// シリアライズ
use serde::{Deserialize, Serialize};
// Tauri コマンドマクロ
use tauri::generate_handler;

// 暗号化ストレージ抽象（§11.4.2 ADR-0004）
pub mod secure_storage;
// メディア処理（SHA-256 計算、§10.4 §10.4.5）
pub mod media;

/// 端末側で扱う Task DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    /// Task ID
    pub id: String,
    /// 状態ラベル
    pub state: String,
    /// 主体端末 ID
    pub device_id: String,
    /// Lamport タイムスタンプ
    pub lamport: u64,
}

/// `get_task` コマンド: ID で Task を取得する
#[tauri::command]
fn get_task(id: String) -> Option<TaskDto> {
    // スタブ: 既知 ID のみサンプルを返す
    if id == "sample-task" {
        // サンプル Task DTO を返す
        Some(TaskDto {
            id,
            state: "Idle".to_string(),
            device_id: "sample-device".to_string(),
            lamport: 0,
        })
    } else {
        // 未存在
        None
    }
}

/// `save_task` コマンド: Task を保存する
#[tauri::command]
fn save_task(_id: String, _state: String, _device_id: String, _lamport: u64) -> Result<(), String> {
    // スタブ: 受領のみ。永続化は §10.6 実装で SQLCipher と接続する。
    Ok(())
}

/// メディアキャプチャ DTO（§10.4 §10.4.5）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMediaResponse {
    /// 端末内一意 ID
    pub id: String,
    /// 種別（photo／burst／video／timelapse／audio／qr／ocr）
    pub kind: String,
    /// SHA-256 ハッシュ（hex 64 文字）
    pub sha256: String,
    /// ローカルファイルパス（端末暗号化領域）
    pub path: String,
    /// バイト数
    pub bytes: u64,
    /// 取得時刻（UTC ISO 8601、§20.2）
    pub captured_at: String,
}

/// `capture_media` コマンド: メディアを取得する
#[tauri::command]
fn capture_media(
    kind: String,
    _max_duration_seconds: Option<u64>,
    _recognition_timeout_ms: Option<u64>,
) -> Result<CaptureMediaResponse, String> {
    // メディアモジュールに委譲（SHA-256 計算は媒体バイト列に対して行う）
    let r = media::capture_stub(&kind);
    // DTO に射影
    Ok(CaptureMediaResponse {
        id: r.id,
        kind: r.kind,
        sha256: r.sha256,
        path: r.path,
        bytes: r.bytes,
        captured_at: r.captured_at,
    })
}

/// アプリケーションのエントリポイント
///
/// `main.rs` から呼ばれ、Tauri ランタイムを起動する。
pub fn run() {
    // tauri::Builder を構築
    tauri::Builder::default()
        // コマンドハンドラを登録
        .invoke_handler(generate_handler![get_task, save_task, capture_media])
        // 既定設定で起動
        .run(tauri::generate_context!())
        .expect("tauri アプリの起動に失敗しました");
}
