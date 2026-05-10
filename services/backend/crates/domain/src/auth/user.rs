//! ユーザ識別子と利用者プロフィール
//!
//! 対応 §: ロードマップ §10.5 §10.5.1
//!
//! 認証情報そのものは [`super::credential::Credential`] に分離する（§9.4 関心分離）。

use core::fmt;

use crate::error::DomainError;

/// ユーザ ID 値オブジェクト
///
/// ASCII 文字（英数字・ハイフン・アンダースコア）のみを許容する。
/// 1〜64 文字。空文字／規定外文字は不正。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    /// 文字列から UserId を構築する
    ///
    /// # Errors
    /// 空／長さ超／規定外文字の場合に `DomainError::InvalidIdentifier` を返す。
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let v: String = value.into();
        if v.is_empty() {
            return Err(DomainError::InvalidIdentifier("UserId が空です"));
        }
        if v.len() > 64 {
            return Err(DomainError::InvalidIdentifier("UserId が長すぎます"));
        }
        if !v
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(DomainError::InvalidIdentifier(
                "UserId に許容されない文字が含まれています",
            ));
        }
        Ok(Self(v))
    }

    /// 内部 &str を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 利用者プロフィール
///
/// 認証情報そのものは [`super::credential::Credential`] に分離されており、
/// 本型は表示・権限判定に必要な最小情報のみを保持する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    display_name: String,
    /// 有効化フラグ（§10.5.1 アカウント無効化伝播）
    enabled: bool,
}

impl User {
    /// 新しい User を生成する
    ///
    /// # Errors
    /// 表示名が空または 128 文字超の場合は不正。
    pub fn new(id: UserId, display_name: impl Into<String>) -> Result<Self, DomainError> {
        let dn: String = display_name.into();
        if dn.is_empty() {
            return Err(DomainError::InvalidIdentifier("display_name が空です"));
        }
        if dn.len() > 128 {
            return Err(DomainError::InvalidIdentifier(
                "display_name が長すぎます",
            ));
        }
        Ok(Self {
            id,
            display_name: dn,
            enabled: true,
        })
    }

    /// 利用者 ID を取得する
    #[must_use]
    pub const fn id(&self) -> &UserId {
        &self.id
    }

    /// 表示名を取得する
    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// 有効化されているか
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 無効化する（§10.5.1）
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_id_rejects_empty() {
        let r = UserId::new("");
        assert!(matches!(r, Err(DomainError::InvalidIdentifier(_))));
    }

    #[test]
    fn user_id_rejects_special_chars() {
        let r = UserId::new("user name");
        assert!(matches!(r, Err(DomainError::InvalidIdentifier(_))));
    }

    #[test]
    fn user_id_accepts_valid() {
        let r = UserId::new("operator-1");
        assert!(r.is_ok());
        assert_eq!(r.unwrap().as_str(), "operator-1");
    }

    #[test]
    fn user_id_display_matches_as_str() {
        let id = UserId::new("op-test").expect("valid");
        assert_eq!(format!("{id}"), "op-test");
    }

    #[test]
    fn user_rejects_empty_display_name() {
        let id = UserId::new("u1").expect("valid");
        let r = User::new(id, "");
        assert!(matches!(r, Err(DomainError::InvalidIdentifier(_))));
    }

    #[test]
    fn user_disable_flips_flag() {
        let id = UserId::new("u1").expect("valid");
        let mut u = User::new(id, "オペレータ A").expect("valid");
        assert!(u.is_enabled());
        u.disable();
        assert!(!u.is_enabled());
    }

    #[test]
    fn user_display_name_preserves_value() {
        let id = UserId::new("u1").expect("valid");
        let u = User::new(id, "テストユーザ").expect("valid");
        assert_eq!(u.display_name(), "テストユーザ");
    }

    #[test]
    fn user_display_name_boundary_128_ok_129_reject() {
        let id = UserId::new("u1").expect("valid");
        let ok = "a".repeat(128);
        assert!(User::new(id.clone(), ok).is_ok());
        let ng = "a".repeat(129);
        assert!(User::new(id, ng).is_err());
    }
}
