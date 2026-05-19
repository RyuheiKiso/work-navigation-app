// trybuild を使ってコンパイルエラーが期待通りに発生することを検証する
// TerminalApiConfig.database.write へのアクセスがコンパイルエラーになることを保証する

#[test]
fn test_terminal_api_config_has_no_write_field() {
    // trybuild で compile_fail ディレクトリのファイルがコンパイルに失敗することを確認する
    // この保証は DB ロール分離の型レベルでの強制を証明する
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/terminal_no_write.rs");
}
