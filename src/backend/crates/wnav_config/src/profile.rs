// 環境プロファイルを型として表現し、不正な文字列を早期に拒絶する

use crate::error::ConfigError;
use std::fmt;

/// デプロイ環境を表す列挙型
/// 環境ごとに異なる設定ファイル（config.{profile}.yml）をマージする
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Profile {
    // ローカル開発環境（開発者 PC 上で動かす）
    Local,
    // 開発検証環境（チーム共有の開発サーバー）
    Dev,
    // ステージング環境（本番同等の検証環境）
    Staging,
    // 本番環境（最高の厳格さで設定を適用する）
    Prod,
}

impl Profile {
    /// WNAV_PROFILE 環境変数からプロファイルを読み込む
    /// 未設定の場合は fail-fast のため ProfileNotSet エラーを返す
    pub fn from_env() -> Result<Self, ConfigError> {
        let value = std::env::var("WNAV_PROFILE").map_err(|_| ConfigError::ProfileNotSet)?;
        match value.to_lowercase().as_str() {
            "local" => Ok(Profile::Local),
            "dev" => Ok(Profile::Dev),
            "staging" => Ok(Profile::Staging),
            "prod" => Ok(Profile::Prod),
            // 不明なプロファイル名は ProfileNotSet として扱いエラーメッセージで候補を示す
            _ => Err(ConfigError::ProfileNotSet),
        }
    }
}

// YAML ファイル名生成（config.{profile}.yml）に使用するため Display を実装する
impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Profile::Local => write!(f, "local"),
            Profile::Dev => write!(f, "dev"),
            Profile::Staging => write!(f, "staging"),
            Profile::Prod => write!(f, "prod"),
        }
    }
}
