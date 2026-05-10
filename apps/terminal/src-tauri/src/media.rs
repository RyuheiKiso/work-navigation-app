//! 端末メディア処理
//!
//! 対応 §: ロードマップ §10.4 §10.4.5 §11.4.1 §27 F-008
//!
//! メディアバイト列に対して SHA-256 を計算し、§10.4.5 改ざん検出に使用する。
//! 実カメラ／マイク統合は将来 `tauri-plugin-camera`／`tauri-plugin-mic` を組み込むまで
//! `bytes_from_capture` を擬似的に返すスタブで動作させる。

// SHA-256
use sha2::{Digest, Sha256};
// UTC ISO 8601
use chrono::Utc;

/// メディア取得結果（lib.rs の `CaptureMediaResponse` と一致）
#[derive(Debug, Clone)]
pub struct CaptureResult {
    /// 端末内一意 ID
    pub id: String,
    /// 種別
    pub kind: String,
    /// SHA-256（hex 64 文字）
    pub sha256: String,
    /// ローカルパス
    pub path: String,
    /// バイト数
    pub bytes: u64,
    /// UTC ISO 8601 取得時刻
    pub captured_at: String,
}

/// バイト列から SHA-256 hex 文字列を生成する
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    // ハッシャを構築
    let mut hasher = Sha256::new();
    // バイトを投入
    hasher.update(bytes);
    // ダイジェスト 32 bytes
    let digest = hasher.finalize();
    // hex 文字列に変換
    let mut s = String::with_capacity(64);
    for b in digest.iter() {
        // 各バイトを 2 文字 hex に
        s.push_str(&format!("{b:02x}"));
    }
    // 完成
    s
}

/// メディアキャプチャ（スタブ）
///
/// 実機の `tauri-plugin-camera` 等が整備されるまでは決定的なダミー値で返す。
/// SHA-256 はダミー（種別のバイト列）を実際にハッシュした値で返す。
#[must_use]
pub fn capture_stub(kind: &str) -> CaptureResult {
    // ダミーバイト列（種別＋取得時刻でユニーク化）
    let captured_at = Utc::now().to_rfc3339();
    let payload = format!("{kind}|{captured_at}");
    let bytes = payload.as_bytes();
    // SHA-256 を計算
    let sha = sha256_hex(bytes);
    // 結果
    CaptureResult {
        id: format!("media-{kind}-{}", &sha[..8]),
        kind: kind.to_string(),
        sha256: sha,
        path: format!("/encrypted/store/{kind}.bin"),
        bytes: bytes.len() as u64,
        captured_at,
    }
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;

    // 既知バイト列の SHA-256（NIST テストベクタ）
    #[test]
    fn sha256_empty_string() {
        // 空のハッシュは e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    // "abc" の SHA-256
    #[test]
    fn sha256_abc() {
        // ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    // capture_stub: 64 文字 hex のハッシュを返す
    #[test]
    fn capture_stub_returns_valid_hash() {
        // 任意の種別
        let r = capture_stub("photo");
        // ハッシュは 64 文字
        assert_eq!(r.sha256.len(), 64);
        // 種別が反映
        assert_eq!(r.kind, "photo");
        // ID は media-<kind>-<hash 先頭8> 形式
        assert!(r.id.starts_with("media-photo-"));
    }
}
