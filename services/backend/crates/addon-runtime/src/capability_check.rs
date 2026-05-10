//! capability の付与判定（§17.4 既定 deny）
//!
//! 対応 §: ロードマップ §17.4 §27 F-004

// SDK
use wna_addon_sdk::Capability;
// thiserror
use thiserror::Error;

/// capability 違反
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CapabilityViolation {
    /// 必要 capability が宣言されていない
    #[error("必要 capability が未宣言: {0:?}")]
    Missing(Capability),
    /// glob／ワイルドカード越境を試みた
    #[error("glob／ワイルドカード越境は禁止されています: {0}")]
    WildcardForbidden(String),
}

/// 必要 capability の集合がアドオン宣言に含まれているかを検査する
///
/// `granted` がアドオンマニフェスト由来、`required` が今呼び出した API の必要 capability。
/// `granted` に **完全一致** で含まれない `required` 要素があれば違反。
///
/// 文字列マッチではなく値オブジェクトの等価比較を行うため、
/// `NetOutbound("a.example.com")` と `NetOutbound("*.example.com")` は別物として扱われる（§17.4 glob 不許可）。
pub fn check_required(
    granted: &[Capability],
    required: &[Capability],
) -> Result<(), CapabilityViolation> {
    // ワイルドカード越境チェック（granted 側に "*" を含むホスト／名前空間があれば拒否）
    for cap in granted {
        match cap {
            Capability::NetOutbound(host) if host.contains('*') => {
                return Err(CapabilityViolation::WildcardForbidden(host.clone()));
            }
            Capability::Storage(ns) if ns.contains('*') => {
                return Err(CapabilityViolation::WildcardForbidden(ns.clone()));
            }
            Capability::UiExtend(slot) if slot.contains('*') => {
                return Err(CapabilityViolation::WildcardForbidden(slot.clone()));
            }
            _ => {}
        }
    }
    // 各 required を granted で完全一致検索
    for r in required {
        if !granted.contains(r) {
            return Err(CapabilityViolation::Missing(r.clone()));
        }
    }
    // 全要素が granted に含まれていれば OK
    Ok(())
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;
    // SDK
    use wna_addon_sdk::NotificationChannel;

    // 必要 capability が含まれていれば OK
    #[test]
    fn pass_when_required_subset() {
        let granted = vec![
            Capability::TaskRead,
            Capability::TaskWrite,
            Capability::Notify(NotificationChannel::Andon),
        ];
        let required = vec![Capability::TaskRead];
        // 違反なし
        assert!(check_required(&granted, &required).is_ok());
    }

    // 不足があれば Missing
    #[test]
    fn fail_when_required_missing() {
        let granted = vec![Capability::TaskRead];
        let required = vec![Capability::TaskWrite];
        let r = check_required(&granted, &required);
        assert!(matches!(r, Err(CapabilityViolation::Missing(_))));
    }

    // ワイルドカード混入は WildcardForbidden
    #[test]
    fn fail_on_wildcard_host() {
        let granted = vec![Capability::NetOutbound("*.example.com".to_string())];
        let required = vec![Capability::NetOutbound("a.example.com".to_string())];
        let r = check_required(&granted, &required);
        assert!(matches!(r, Err(CapabilityViolation::WildcardForbidden(_))));
    }
}
