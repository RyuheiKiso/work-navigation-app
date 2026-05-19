// wnav_common クレート
//
// 共通ライブラリ（MOD-SH-001〜004）。
// バックエンド・フロントエンドを問わず全モジュールから利用される横断的関心事を集約する。
//
// # モジュール構成
// - `locale`: MOD-SH-001 LocaleResolver（i18n バックエンド統合）
// - `id_generator`: MOD-SH-002 IdGenerator（UUID v7 生成）
// - `clock`: MOD-SH-003 ClockService（時刻抽象）
// - `api_client`: MOD-SH-004 ApiClient（親機連携）
// - `error`: RFC 7807 Problem Details 型
//
// # 依存クレート禁止
// 本クレートは他のカスタムクレートに依存しない。
// 完全独立のユーティリティクレートとして設計する。

// unsafe コードを禁止する（src/CLAUDE.md および src/backend/CLAUDE.md の必須要件）
#![forbid(unsafe_code)]
// Clippy の全警告をエラーとして扱う（他クレートとの一貫性）
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// rust-i18n マクロで locales/ ディレクトリを初期化する。
// このマクロはクレートルート（lib.rs）で一度だけ呼び出す必要がある。
// locale.rs の t() 関数はこのマクロが生成した _rust_i18n_t を使用する。
rust_i18n::i18n!("locales");

pub mod api_client;
pub mod clock;
pub mod error;
pub mod id_generator;
pub mod locale;

// 主要な型・関数を再エクスポートして使いやすくする
pub use api_client::{ApiClientError, ParentApiConfig, ParentSystemApiClient};
pub use clock::{Clock, ClockRef, FakeClock, SystemClock, system_clock};
pub use error::ProblemDetails;
pub use id_generator::{id_to_string, new_id, new_idempotency_key, parse_id};
pub use locale::{init_locale, resolve_locale, t};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    #[test]
    fn test_new_id_is_uuid_v7() {
        // new_id() が UUID v7 形式であることを確認する
        let id = new_id();
        assert_eq!(id.get_version_num(), 7);
    }

    #[test]
    fn test_fake_clock_fixed_and_advance() {
        // FakeClock で時刻を固定・進める操作が正しいことを確認する
        let start = Utc.with_ymd_and_hms(2026, 5, 19, 0, 0, 0).unwrap();
        let clock = FakeClock::new(start);

        // 固定時刻が返ることを確認する
        assert_eq!(clock.now(), start);

        // 1 時間進めて確認する
        clock.advance(Duration::hours(1));
        let expected = Utc.with_ymd_and_hms(2026, 5, 19, 1, 0, 0).unwrap();
        assert_eq!(clock.now(), expected);

        // 任意の時刻に変更して確認する
        let new_time = Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59).unwrap();
        clock.set(new_time);
        assert_eq!(clock.now(), new_time);
    }

    #[test]
    fn test_resolve_locale_ja_en_ja_simple() {
        // resolve_locale が正しく ja / en / ja-simple を返すことを確認する
        assert_eq!(resolve_locale(Some("ja")), "ja");
        assert_eq!(resolve_locale(Some("en")), "en");
        assert_eq!(resolve_locale(Some("ja-simple")), "ja-simple");
        // フォールバックを確認する
        assert_eq!(resolve_locale(None), "ja");
        assert_eq!(resolve_locale(Some("zh")), "ja");
    }

    #[test]
    fn test_problem_details_structure() {
        // ProblemDetails が RFC 7807 準拠の構造を持つことを確認する
        let problem = ProblemDetails::new(
            401,
            "https://errors.wnav.example.com/auth/unauthorized",
            "Unauthorized",
            "Authentication required",
        );
        assert_eq!(problem.status, 401);
        assert_eq!(problem.title, "Unauthorized");
        assert!(problem.instance.is_none());

        // JSON シリアライズで "type" フィールドが使われることを確認する
        let json = serde_json::to_string(&problem).expect("シリアライズに失敗した");
        assert!(json.contains(r#""type""#));
    }
}
