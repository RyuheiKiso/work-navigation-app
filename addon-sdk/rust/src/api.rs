//! addon SDK API surface v1
//!
//! 対応 §: ロードマップ §17.3 §17.4 §17.5

// シリアライズ
use serde::{Deserialize, Serialize};

/// アドオン実行コンテキスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonContext {
    /// アドオン ID（マニフェスト）
    pub addon_id: String,
    /// ロケール（§11.3 動作中ロケール）
    pub locale: String,
}

/// capability（§17.4 最小権限）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    /// 作業情報読取
    TaskRead,
    /// 作業実績書込
    TaskWrite,
    /// メディア書込
    MediaWrite,
    /// メディア読取
    MediaRead,
    /// HTTP アウトバウンド（特定ホストのみ）
    NetOutbound(String),
    /// ストレージ KV（名前空間）
    Storage(String),
    /// 通知（チャネル）
    Notify(NotificationChannel),
    /// UI 拡張（スロット）
    UiExtend(String),
    /// 設定読取
    ConfigRead,
    /// 暗号署名
    CryptoSign,
}

/// 通知チャネル
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// アンドン（§9.3.1）
    Andon,
    /// メール
    Email,
    /// Slack 等の外部チャット
    Chat,
}

/// 現在の作業情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// 作業 ID
    pub id: String,
    /// 状態ラベル
    pub state: String,
}

/// アドオン API 呼び出しのエラー
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddonError {
    /// 必要な capability が宣言されていない（§17.4 既定 deny）
    CapabilityMissing(String),
    /// ホスト側の一時障害
    Transient(String),
    /// 引数不正
    InvalidArgument(String),
}

/// ホスト API
///
/// アドオンランタイム（Wasmtime, §17.5）が本 trait を実装し、アドオンに注入する。
/// アドオン作者は本 trait のみに依存し、ランタイムの詳細から独立する。
pub trait Host {
    /// 現在の作業情報を取得（capability: `TaskRead`）
    fn get_current_task(&self) -> Result<TaskInfo, AddonError>;

    /// 作業実績を追記する（capability: `TaskWrite`）
    fn append_record(&self, task_id: &str, payload: &str) -> Result<(), AddonError>;

    /// 通知を送る（capability: `Notify(channel)`）
    fn notify(&self, channel: NotificationChannel, message: &str) -> Result<(), AddonError>;

    /// ロギング（既定許可、§17.3）
    fn log(&self, level: &str, message: &str);

    /// 公開設定の読取（capability: `ConfigRead`）
    fn get_config(&self, key: &str) -> Result<Option<String>, AddonError>;

    /// 時刻取得（既定許可、サーバ時刻同期済み）
    fn now(&self) -> i64;
}
