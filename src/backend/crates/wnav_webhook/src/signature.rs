// HMAC-SHA256 署名生成・検証モジュール（MOD-BE-007 §2 / FNC-BE-013）
// 配信側: sign_payload() でペイロードに署名し X-WNav-Signature ヘッダを生成する
// 受信側: verify_signature() で定数時間比較（timing attack 防止）を行う

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::error::WebhookError;

/// ペイロードを HMAC-SHA256 で署名して "sha256={hex}" 形式の文字列を返す（配信側）。
///
/// # 引数
/// - `payload`: 署名対象のバイト列（JSON ボディ等）
/// - `secret`: HMAC 秘密鍵文字列（hex デコード前の生文字列、または UTF-8 バイト列として使用）
///
/// # 戻り値
/// `"sha256={hex_digest}"` 形式の文字列（X-WNav-Signature ヘッダ値）
///
/// # ヘッダ形式
/// ```text
/// X-WNav-Signature: sha256=abc123def456...
/// ```
pub fn sign_payload(payload: &[u8], secret: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;

    // HMAC-SHA256 インスタンスを生成して署名を計算する
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);
    let result = mac.finalize();

    // "sha256={hex}" 形式で返す
    format!("sha256={}", hex::encode(result.into_bytes()))
}

/// 受信ペイロードの HMAC-SHA256 署名を検証する（受信側）。
///
/// タイミング攻撃防止のため `subtle::ConstantTimeEq` による定数時間比較を使用する。
/// `==` 演算子は使用しない。
///
/// # 引数
/// - `payload`: 受信したリクエストボディのバイト列
/// - `secret`: HMAC 秘密鍵文字列（配信側と同一）
/// - `header_value`: `X-WNav-Signature` ヘッダの値（"sha256={hex}" 形式）
///
/// # エラー
/// - `WebhookError::InvalidSignature`: 署名が一致しない場合
pub fn verify_signature(
    payload: &[u8],
    secret: &str,
    header_value: &str,
) -> Result<(), WebhookError> {
    // expected_hex を "sha256=" プレフィックスを除いて取得する
    let expected_hex = header_value
        .strip_prefix("sha256=")
        .unwrap_or(header_value);

    // 受信ペイロードで署名を計算する
    let computed = sign_payload(payload, secret);
    // "sha256=" プレフィックスを除いた hex 部分を取得する
    let computed_hex = computed
        .strip_prefix("sha256=")
        .unwrap_or(&computed);

    let computed_bytes = computed_hex.as_bytes();
    let expected_bytes = expected_hex.as_bytes();

    // 長さが異なる場合は即座に拒否する（ただし定数時間性を保つため比較も行う）
    if computed_bytes.len() != expected_bytes.len() {
        // 定数時間比較でダミー操作を行ってからエラーを返す
        let _ = computed_bytes.ct_eq(b"dummy");
        return Err(WebhookError::InvalidSignature);
    }

    // 定数時間比較（タイミング攻撃防止）で一致を確認する
    if computed_bytes.ct_eq(expected_bytes).into() {
        Ok(())
    } else {
        Err(WebhookError::InvalidSignature)
    }
}

/// タイムスタンプ検証（リプレイ攻撃防止）。
///
/// `X-WNav-Timestamp` ヘッダのエポック秒が現在時刻との差分が `tolerance_secs` 以内か確認する。
/// tolerance_secs のデフォルトは 300 秒（5 分）推奨。
///
/// # 引数
/// - `timestamp_str`: エポック秒の文字列
/// - `tolerance_secs`: 許容する時刻差分（秒）
///
/// # エラー
/// - `WebhookError::RequestTimeout`: タイムスタンプが許容範囲外の場合
pub fn verify_timestamp(
    timestamp_str: &str,
    tolerance_secs: i64,
) -> Result<(), WebhookError> {
    // タイムスタンプを Unix 秒としてパースする
    let ts: i64 = timestamp_str
        .parse()
        .map_err(|_| WebhookError::RequestTimeout)?;

    let now = chrono::Utc::now().timestamp();
    let diff = (now - ts).abs();

    // 時刻差分が許容範囲を超える場合はリプレイ攻撃とみなして拒否する
    if diff > tolerance_secs {
        Err(WebhookError::RequestTimeout)
    } else {
        Ok(())
    }
}
