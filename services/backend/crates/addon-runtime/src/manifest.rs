//! アドオンマニフェスト（capability 宣言）
//!
//! 対応 §: ロードマップ §17.4 §17.6
//!
//! `manifest.toml` を SDK の `Capability` 列に変換する。

// シリアライズ
use serde::{Deserialize, Serialize};
// SDK
use wna_addon_sdk::Capability;
// thiserror
use thiserror::Error;

/// アドオンマニフェストの生表現
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonManifest {
    /// アドオン ID
    pub id: String,
    /// 表示名
    pub name: String,
    /// バージョン（SemVer）
    pub version: String,
    /// 必要 capability の文字列列
    pub capabilities: Vec<String>,
}

/// マニフェストパースエラー
#[derive(Debug, Error)]
pub enum ManifestError {
    /// capability 文字列が認識できない
    #[error("capability の解釈に失敗: {0}")]
    InvalidCapability(String),
}

impl AddonManifest {
    /// capability 文字列を SDK 列に変換する
    ///
    /// 形式は §17.3 表に従う:
    /// - `task.read`／`task.write`
    /// - `media.read`／`media.write`
    /// - `net.outbound:<host>`
    /// - `storage:<namespace>`
    /// - `notify:<channel>`（andon／email／chat）
    /// - `ui.extend:<slot>`
    /// - `config.read`
    /// - `crypto.sign`
    pub fn parse_capabilities(&self) -> Result<Vec<Capability>, ManifestError> {
        // 走査結果
        let mut out = Vec::with_capacity(self.capabilities.len());
        // 各文字列を解釈
        for cap_str in &self.capabilities {
            // ":" の手前と後ろで分解
            let (head, tail) = match cap_str.split_once(':') {
                Some((h, t)) => (h, Some(t)),
                None => (cap_str.as_str(), None),
            };
            // 既知パターンを判定
            let cap = match (head, tail) {
                ("task.read", None) => Capability::TaskRead,
                ("task.write", None) => Capability::TaskWrite,
                ("media.read", None) => Capability::MediaRead,
                ("media.write", None) => Capability::MediaWrite,
                ("config.read", None) => Capability::ConfigRead,
                ("crypto.sign", None) => Capability::CryptoSign,
                ("net.outbound", Some(host)) => Capability::NetOutbound(host.to_string()),
                ("storage", Some(ns)) => Capability::Storage(ns.to_string()),
                ("ui.extend", Some(slot)) => Capability::UiExtend(slot.to_string()),
                ("notify", Some(channel)) => match channel {
                    "andon" => Capability::Notify(wna_addon_sdk::NotificationChannel::Andon),
                    "email" => Capability::Notify(wna_addon_sdk::NotificationChannel::Email),
                    "chat" => Capability::Notify(wna_addon_sdk::NotificationChannel::Chat),
                    _ => {
                        return Err(ManifestError::InvalidCapability(cap_str.clone()));
                    }
                },
                _ => return Err(ManifestError::InvalidCapability(cap_str.clone())),
            };
            // 結果に追加
            out.push(cap);
        }
        Ok(out)
    }
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;

    // 既知 capability を全網羅
    #[test]
    fn parses_all_known_capabilities() {
        // すべての分類を含むマニフェスト
        let m = AddonManifest {
            id: "test".into(),
            name: "Test".into(),
            version: "0.1.0".into(),
            capabilities: vec![
                "task.read".into(),
                "task.write".into(),
                "media.read".into(),
                "media.write".into(),
                "config.read".into(),
                "crypto.sign".into(),
                "net.outbound:opc-ua-server.local".into(),
                "storage:my-ns".into(),
                "ui.extend:flow-detail-panel".into(),
                "notify:andon".into(),
                "notify:email".into(),
                "notify:chat".into(),
            ],
        };
        // 解釈
        let caps = m.parse_capabilities().expect("ok");
        // 12 件
        assert_eq!(caps.len(), 12);
    }

    // 未知 capability はエラー
    #[test]
    fn rejects_unknown_capability() {
        // 未知のもの
        let m = AddonManifest {
            id: "test".into(),
            name: "Test".into(),
            version: "0.1.0".into(),
            capabilities: vec!["unknown.thing".into()],
        };
        let r = m.parse_capabilities();
        // エラー
        assert!(matches!(r, Err(ManifestError::InvalidCapability(_))));
    }
}
