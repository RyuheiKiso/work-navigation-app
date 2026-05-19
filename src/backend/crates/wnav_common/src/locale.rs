// MOD-SH-001: LocaleResolver（i18n バックエンド統合）
// rust-i18n クレートで ja / en / ja-simple の 3 言語を提供する。
// Accept-Language ヘッダからロケールを解決して全エラーメッセージを多言語化する。
//
// 注意: rust_i18n::i18n! マクロは lib.rs のクレートルートで呼び出す必要がある。
// このモジュールでは rust_i18n::t! マクロを直接使用する。

/// バックエンドの翻訳設定を初期化する。
///
/// main.rs の起動処理の最初に呼び出す。
///
/// # 翻訳ファイルの配置
/// ```text
/// locales/
///   ja.toml        — 日本語（必須、100% 翻訳率）
///   en.toml        — 英語（必須、100% 翻訳率）
///   ja-simple.toml — やさしい日本語（JLPT N4 相当、FR-UI-002）
/// ```
pub fn init_locale() {
    // rust-i18n は i18n! マクロ呼び出し時に初期化されるため、
    // 明示的な初期化処理は不要だが、呼び出し元への明示的なインターフェースとして提供する
    rust_i18n::set_locale("ja");
}

/// リクエストの Accept-Language ヘッダからロケールを解決する。
///
/// # 対応言語
/// - `ja`: 日本語（デフォルトフォールバック）
/// - `en`: 英語
/// - `ja-simple`: やさしい日本語（JLPT N4 相当）
///
/// # フォールバック
/// 対応言語にマッチしない場合は "ja" を返す。
pub fn resolve_locale(accept_language: Option<&str>) -> &'static str {
    match accept_language {
        // ja-simple は en より前に判定する（より具体的なマッチを優先する）
        Some(lang) if lang.contains("ja-simple") => "ja-simple",
        // 英語の Accept-Language ヘッダを検出する
        Some(lang) if lang.contains("en") => "en",
        // 日本語明示指定（明示しなくても ja にフォールバックする）
        Some(lang) if lang.contains("ja") => "ja",
        // 対応外の言語またはヘッダなしは ja にフォールバックする
        _ => "ja",
    }
}

/// 翻訳キーからメッセージを取得する。
///
/// # キー形式
/// `{feature}.{component}.{key}`
///
/// # 例
/// ```rust,ignore
/// let msg = t("error.biz.lock_step_violation", "ja");
/// assert_eq!(msg, "前の手順が完了していません。手順を順番に実施してください");
/// ```
///
/// # 引数
/// - `key`: 翻訳キー（ドット区切り）
/// - `locale`: ロケール文字列（"ja" / "en" / "ja-simple"）
pub fn t(key: &str, locale: &str) -> String {
    // rust_i18n マクロを使って翻訳を取得する
    rust_i18n::t!(key, locale = locale).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_locale_ja() {
        // 日本語のロケール解決が正しいことを確認する
        assert_eq!(resolve_locale(Some("ja")), "ja");
        assert_eq!(resolve_locale(Some("ja-JP")), "ja");
    }

    #[test]
    fn test_resolve_locale_en() {
        // 英語のロケール解決が正しいことを確認する
        assert_eq!(resolve_locale(Some("en")), "en");
        assert_eq!(resolve_locale(Some("en-US")), "en");
        assert_eq!(resolve_locale(Some("en-GB,en;q=0.9")), "en");
    }

    #[test]
    fn test_resolve_locale_ja_simple() {
        // やさしい日本語のロケール解決が正しいことを確認する
        assert_eq!(resolve_locale(Some("ja-simple")), "ja-simple");
    }

    #[test]
    fn test_resolve_locale_fallback_to_ja() {
        // 未知のロケールは ja にフォールバックすることを確認する
        assert_eq!(resolve_locale(None), "ja");
        assert_eq!(resolve_locale(Some("zh-CN")), "ja");
        assert_eq!(resolve_locale(Some("ko")), "ja");
        assert_eq!(resolve_locale(Some("")), "ja");
    }

    #[test]
    fn test_t_ja() {
        // 日本語翻訳キーが正しいメッセージを返すことを確認する
        let msg = t("error.auth.unauthorized", "ja");
        assert_eq!(msg, "認証が必要です");
    }

    #[test]
    fn test_t_en() {
        // 英語翻訳キーが正しいメッセージを返すことを確認する
        let msg = t("error.auth.unauthorized", "en");
        assert_eq!(msg, "Authentication required");
    }

    #[test]
    fn test_t_biz_errors_ja() {
        // ビジネスロジックエラーの日本語翻訳が正しいことを確認する
        assert_eq!(
            t("error.biz.lock_step_violation", "ja"),
            "前の手順が完了していません。手順を順番に実施してください"
        );
        assert_eq!(
            t("error.biz.evidence_required", "ja"),
            "この手順には証拠記録が必要です"
        );
        assert_eq!(
            t("error.biz.sign_required", "ja"),
            "この手順には電子サインが必要です"
        );
        assert_eq!(
            t("error.biz.case_locked", "ja"),
            "この作業セッションは別の端末で作業中のため操作できません"
        );
        assert_eq!(
            t("error.biz.step_not_found", "ja"),
            "指定された手順が見つかりません"
        );
        assert_eq!(
            t("error.biz.sop_not_published", "ja"),
            "この標準作業手順書はまだ公開されていません"
        );
    }
}
