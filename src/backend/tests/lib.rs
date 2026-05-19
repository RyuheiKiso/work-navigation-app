// wnav_tests クレート
//
// 作業ナビゲーションシステムバックエンドの統合テスト・ALCOA+検証テスト・セキュリティテスト。
// testcontainers-rs で実際の PostgreSQL コンテナを使用する（Docker 必須）。
//
// # テストレベル
// - `integration/`: 統合テスト（TST-intg-001〜014）
// - `alcoa/`: ALCOA+ 検証テスト（TST-alcoa-001〜010）
// - `security/`: セキュリティテスト（TST-sec-001〜011）
//
// # 実行方法
// ```bash
// # Docker 要求テスト（#[ignore = "requires Docker"] 付き）も含めて実行する
// cargo test -p wnav_tests -- --include-ignored
//
// # Docker なし環境でのユニットテストのみ（#[ignore] を除外）
// cargo test -p wnav_tests
// ```

// unsafe コードを禁止する
#![forbid(unsafe_code)]
