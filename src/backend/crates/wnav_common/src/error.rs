// RFC 7807 Problem Details 型定義モジュール
// クライアント向けエラーレスポンスを RFC 7807 準拠の形式で表現する。
// スタックトレース・内部エラー詳細はクライアントに返さない（ログにのみ記録する）。

/// RFC 7807 Problem Details 準拠のエラーレスポンス型。
///
/// # 参照
/// - RFC 7807: Problem Details for HTTP APIs
/// - `docs/05_詳細設計/02_バックエンド詳細設計/09_共通ライブラリ詳細設計.md` §4
///
/// # エラーボディ例
/// ```json
/// {
///   "type": "https://errors.example.com/insufficient-permission",
///   "title": "Insufficient Permission",
///   "status": 403,
///   "detail": "RBAC role 'operator' cannot access audit trail.",
///   "instance": "/api/v1/audit/events/12345"
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ProblemDetails {
    /// エラー種別を示す URI（RFC 7807 の "type" フィールド）
    /// 例: "https://errors.wnav.example.com/auth/unauthorized"
    #[serde(rename = "type")]
    pub problem_type: String,

    /// エラーの短いタイトル（RFC 7807 の "title" フィールド）
    /// HTTP ステータスコードの標準フレーズと一致させることが多い
    pub title: String,

    /// HTTP ステータスコード（RFC 7807 の "status" フィールド）
    pub status: u16,

    /// エラーの詳細説明（RFC 7807 の "detail" フィールド）
    /// クライアントが理解できる説明文を記述する（内部エラーは含めない）
    pub detail: String,

    /// このエラーが発生したリクエストの URI（RFC 7807 の "instance" フィールド）
    /// 省略可能。指定する場合はリクエストパスを使用する
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl ProblemDetails {
    /// RFC 7807 Problem Details インスタンスを生成する。
    ///
    /// # 引数
    /// - `status`: HTTP ステータスコード（例: 401, 403, 404, 422, 500）
    /// - `problem_type`: エラー種別 URI（例: "about:blank" または "https://errors.wnav.example.com/..."）
    /// - `title`: エラーの短いタイトル（例: "Unauthorized", "Forbidden"）
    /// - `detail`: エラーの詳細説明（クライアント向け、内部情報を含めない）
    pub fn new(status: u16, problem_type: &str, title: &str, detail: &str) -> Self {
        Self {
            problem_type: problem_type.to_string(),
            title: title.to_string(),
            status,
            detail: detail.to_string(),
            instance: None,
        }
    }

    /// instance フィールドを設定したビルダーメソッド。
    pub fn with_instance(mut self, instance: &str) -> Self {
        self.instance = Some(instance.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_details_new() {
        // ProblemDetails の基本的な生成が正しいことを確認する
        let problem = ProblemDetails::new(
            401,
            "https://errors.wnav.example.com/auth/unauthorized",
            "Unauthorized",
            "Authentication required",
        );

        assert_eq!(problem.status, 401);
        assert_eq!(problem.title, "Unauthorized");
        assert_eq!(problem.detail, "Authentication required");
        assert!(problem.instance.is_none());
    }

    #[test]
    fn test_problem_details_with_instance() {
        // instance フィールドの設定が正しいことを確認する
        let problem = ProblemDetails::new(
            404,
            "https://errors.wnav.example.com/step/not-found",
            "Not Found",
            "The specified step was not found",
        )
        .with_instance("/api/v1/steps/nonexistent-id");

        assert_eq!(
            problem.instance,
            Some("/api/v1/steps/nonexistent-id".to_string())
        );
    }

    #[test]
    fn test_problem_details_serialization() {
        // RFC 7807 準拠の JSON シリアライズが正しいことを確認する
        let problem = ProblemDetails::new(
            403,
            "about:blank",
            "Forbidden",
            "Insufficient role",
        );

        let json = serde_json::to_string(&problem).expect("シリアライズに失敗した");

        // "type" フィールドとして出力されることを確認する
        assert!(json.contains(r#""type""#));
        assert!(json.contains("about:blank"));
        assert!(json.contains("\"status\":403"));

        // instance が None の場合は出力されないことを確認する
        assert!(!json.contains("\"instance\""));
    }

    #[test]
    fn test_problem_details_instance_in_json() {
        // instance がある場合は JSON に含まれることを確認する
        let problem = ProblemDetails::new(500, "about:blank", "Internal Server Error", "An unexpected error occurred")
            .with_instance("/api/v1/work-events");

        let json = serde_json::to_string(&problem).expect("シリアライズに失敗した");
        assert!(json.contains("\"instance\""));
        assert!(json.contains("/api/v1/work-events"));
    }
}
