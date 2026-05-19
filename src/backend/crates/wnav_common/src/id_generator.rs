// MOD-SH-002: IdGenerator（UUID v7 生成）
// UUID v7 はタイムスタンプ（ミリ秒精度）を先頭に埋め込むため、
// ORDER BY id の性能が UUID v4 より高く、タイムスタンプでの自然ソートが可能。
// すべての主キー生成はこのモジュール経由で行い、DB の DEFAULT 句による UUID 生成は使用しない。

use uuid::Uuid;

/// UUID v7 を生成する。
///
/// # UUID v7 の特性
/// - タイムスタンプ（ミリ秒精度）を先頭 48 ビットに埋め込む
/// - ORDER BY id で時系列ソートが可能（UUID v4 より高性能）
/// - すべての主キー生成はこの関数経由で行う
/// - DB の DEFAULT 句による UUID 生成は使用しない（バージョン統一のため）
#[must_use]
pub fn new_id() -> Uuid {
    // タイムスタンプ埋め込みの UUID v7 を生成する
    Uuid::now_v7()
}

/// Idempotency Key を UUID v7 で生成する。
///
/// event_id と同一の UUID v7 を使用することで、
/// WorkEvent.event_id と outbox_events.idempotency_key が一致する設計を実現する。
#[must_use]
pub fn new_idempotency_key() -> Uuid {
    // Idempotency Key も UUID v7 で生成する（タイムスタンプ埋め込み）
    Uuid::now_v7()
}

/// UUID を文字列に変換する。
///
/// API レスポンスや JSON シリアライズ時に使用する。
/// フォーマット: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx（ハイフン区切り小文字）
#[must_use]
pub fn id_to_string(id: &Uuid) -> String {
    id.to_string()
}

/// 文字列から UUID をパースする。
///
/// API 入力値の検証に使用する。
/// パース可能なフォーマット: ハイフンあり・なし・ブレース付きなど UUID の標準フォーマット全般。
///
/// # エラー
/// 有効な UUID 文字列でない場合は `uuid::Error` を返す。
pub fn parse_id(s: &str) -> Result<Uuid, uuid::Error> {
    Uuid::parse_str(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_id_is_uuid_v7() {
        // new_id() が UUID v7 形式であることを確認する
        let id = new_id();
        // UUID v7 のバージョンビットを確認する（バージョンフィールドは 7）
        assert_eq!(id.get_version_num(), 7);
    }

    #[test]
    fn test_new_id_is_monotonic() {
        // UUID v7 はタイムスタンプ埋め込みのため時系列順に生成されることを確認する
        let id1 = new_id();
        let id2 = new_id();
        // UUID v7 はタイムスタンプベースのため、後に生成した UUID が大きいか等しいことを確認する
        assert!(id2 >= id1);
    }

    #[test]
    fn test_new_idempotency_key_is_uuid_v7() {
        // new_idempotency_key() が UUID v7 形式であることを確認する
        let key = new_idempotency_key();
        assert_eq!(key.get_version_num(), 7);
    }

    #[test]
    fn test_id_to_string_format() {
        // id_to_string が正しい UUID 文字列フォーマットを返すことを確認する
        let id = Uuid::parse_str("11111111-2222-7333-8444-555555555555")
            .expect("テスト用 UUID のパースに失敗した");
        let s = id_to_string(&id);
        // ハイフン区切りの小文字 UUID 文字列（36 文字）であることを確認する
        assert_eq!(s.len(), 36);
        assert!(s.contains('-'));
        assert_eq!(s, "11111111-2222-7333-8444-555555555555");
    }

    #[test]
    fn test_parse_id_valid() {
        // 有効な UUID 文字列のパースが成功することを確認する
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_id(valid_uuid);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_id_invalid() {
        // 無効な文字列のパースがエラーになることを確認する
        let invalid = "not-a-uuid";
        let result = parse_id(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_id_roundtrip() {
        // new_id → to_string → parse_id の往復変換が正しいことを確認する
        let original = new_id();
        let s = id_to_string(&original);
        let parsed = parse_id(&s).expect("UUID 文字列のパースに失敗した");
        assert_eq!(original, parsed);
    }
}
