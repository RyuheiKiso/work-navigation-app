// Secret 型の Debug 出力に平文パスワードが含まれないことを検証する
// 開発者がデバッグ時にうっかりシークレットをログに出力する事故を防ぐための型保証テスト

use wnav_config::redact::Secret;

#[test]
fn test_secret_debug_does_not_contain_plain_text() {
    // Secret の Debug 出力が平文値を含まないことを確認する
    // serde の Deserialize を通して生成するのが本番に近い使い方

    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Wrapper {
        secret: Secret,
    }

    // serde_json で平文値から Secret を生成する
    let json = r#"{"secret": "super_secret_password_12345"}"#;
    let wrapper: Wrapper = serde_json::from_str(json).expect("Wrapper のデシリアライズに失敗");

    // Debug 出力を取得する
    let debug_output = format!("{:?}", wrapper.secret);

    // 平文が Debug に漏れていないことを確認する
    assert!(
        !debug_output.contains("super_secret_password_12345"),
        "Debug 出力に平文パスワードが含まれている: {debug_output}"
    );

    // マスキング文字列が含まれていることを確認する
    assert!(
        debug_output.contains("***REDACTED***"),
        "Debug 出力にマスキング文字列が含まれていない: {debug_output}"
    );

    // バイト数が含まれていることを確認する（設定値の存在を確認しやすくする）
    assert!(
        debug_output.contains("bytes"),
        "Debug 出力にバイト数が含まれていない: {debug_output}"
    );
}

#[test]
fn test_secret_expose_returns_plain_text() {
    // expose() メソッドが平文値を正しく返すことを確認する
    // expose() は明示的に選択した場合のみ平文にアクセスできる設計を保証する

    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Wrapper {
        secret: Secret,
    }

    let json = r#"{"secret": "exposed_value_xyz"}"#;
    let wrapper: Wrapper = serde_json::from_str(json).expect("Wrapper のデシリアライズに失敗");

    // expose() が正しい平文値を返すことを確認する
    assert_eq!(
        wrapper.secret.expose(),
        "exposed_value_xyz",
        "expose() が期待値を返さなかった"
    );
}

#[test]
fn test_secret_debug_shows_byte_count() {
    // Debug 出力がバイト数を正確に反映することを確認する
    // 設定値の有無と大まかなサイズを確認できるようにするための保証

    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Wrapper {
        secret: Secret,
    }

    // ASCII 文字列 8 バイトの Secret を作成する
    let json = r#"{"secret": "12345678"}"#;
    let wrapper: Wrapper = serde_json::from_str(json).expect("Wrapper のデシリアライズに失敗");

    let debug_output = format!("{:?}", wrapper.secret);

    // "8 bytes" が含まれることを確認する
    assert!(
        debug_output.contains("8 bytes"),
        "Debug 出力に正確なバイト数が含まれていない: {debug_output}"
    );
}

#[test]
fn test_secret_clone_does_not_expose_in_debug() {
    // Clone した Secret も Debug 出力でマスキングされることを確認する
    // Clone 後に平文が漏れる実装バグを防ぐ

    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Wrapper {
        secret: Secret,
    }

    let json = r#"{"secret": "clone_test_password"}"#;
    let wrapper: Wrapper = serde_json::from_str(json).expect("Wrapper のデシリアライズに失敗");

    let cloned = wrapper.secret.clone();
    let debug_output = format!("{cloned:?}");

    assert!(
        !debug_output.contains("clone_test_password"),
        "Clone した Secret の Debug に平文が含まれている: {debug_output}"
    );
}

#[test]
fn test_secret_empty_string_is_masked() {
    // 空文字列の Secret も正しくマスキングされることを確認する
    // 空の設定値が誤って出力されないことを保証する

    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Wrapper {
        secret: Secret,
    }

    let json = r#"{"secret": ""}"#;
    let wrapper: Wrapper = serde_json::from_str(json).expect("Wrapper のデシリアライズに失敗");

    let debug_output = format!("{:?}", wrapper.secret);

    // 空文字列でも ***REDACTED*** と 0 bytes が表示されることを確認する
    assert!(
        debug_output.contains("***REDACTED***"),
        "空文字列 Secret の Debug にマスキング文字列が含まれていない: {debug_output}"
    );
    assert!(
        debug_output.contains("0 bytes"),
        "空文字列 Secret の Debug に '0 bytes' が含まれていない: {debug_output}"
    );
}
