//! 端末ローカル暗号化ストレージ
//!
//! 対応 §: ロードマップ §11.4.1 §11.4.2 ADR-0004 §27 F-008 §29 R-008 R-015
//!
//! SQLCipher（AES-256）で SQLite を暗号化し、鍵は OS Keystore（Android）／DPAPI（Windows）に
//! 保護保存する想定。本ファイルは **抽象 trait + プラットフォーム別実装の入口** を提供する。
//! 実装の本体は `feature = "sqlcipher"` 有効時のみコンパイルされる。

// thiserror で境界エラー派生
use thiserror::Error;

/// セキュアストレージのエラー
#[derive(Debug, Error)]
pub enum SecureStorageError {
    /// 鍵保護層（OS Keystore／DPAPI）でのエラー
    #[error("鍵保護: {0}")]
    KeyProtection(String),
    /// データベースアクセスエラー
    #[error("データベース: {0}")]
    Database(String),
    /// 鍵の長さ・形式が不正
    #[error("不正な鍵形式")]
    InvalidKey,
}

/// セキュアストレージ trait
///
/// `open` で SQLCipher 暗号化 SQLite を開き、`close` で安全に閉じる。
/// 鍵は trait 実装側で OS の鍵保護機構から取得する。
pub trait SecureStorage: Send + Sync {
    /// 暗号化ストアを開く
    fn open(&self, db_path: &str) -> Result<(), SecureStorageError>;
    /// ストアを閉じる
    fn close(&self) -> Result<(), SecureStorageError>;
    /// マイグレーションを実行する
    fn run_migrations(&self) -> Result<(), SecureStorageError>;
}

// =====================================================================
// SQLCipher 実装（feature = sqlcipher 有効時のみ）
// =====================================================================
#[cfg(feature = "sqlcipher")]
pub mod sqlcipher_backend {
    use super::{SecureStorage, SecureStorageError};
    use rusqlite::Connection;
    use std::sync::Mutex;

    /// SQLCipher 実装
    pub struct SqlCipherStorage {
        // 鍵（実装は OS Keystore／DPAPI から取得した派生鍵を保持）
        key: Vec<u8>,
        // DB 接続
        conn: Mutex<Option<Connection>>,
    }

    impl SqlCipherStorage {
        /// 鍵バイト列から構築する
        pub const fn new(key: Vec<u8>) -> Self {
            // 鍵を保持
            Self {
                key,
                conn: Mutex::new(None),
            }
        }
    }

    impl SecureStorage for SqlCipherStorage {
        fn open(&self, db_path: &str) -> Result<(), SecureStorageError> {
            // 接続を開く
            let conn = Connection::open(db_path)
                .map_err(|e| SecureStorageError::Database(e.to_string()))?;
            // PRAGMA key で SQLCipher を有効化（鍵は hex 文字列）
            let key_hex: String = self
                .key
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            conn.execute_batch(&format!("PRAGMA key = \"x'{key_hex}'\";"))
                .map_err(|e| SecureStorageError::Database(e.to_string()))?;
            // 接続を保持
            *self
                .conn
                .lock()
                .map_err(|_| SecureStorageError::Database("lock 失敗".to_string()))? =
                Some(conn);
            Ok(())
        }

        fn close(&self) -> Result<(), SecureStorageError> {
            // 接続を破棄
            *self
                .conn
                .lock()
                .map_err(|_| SecureStorageError::Database("lock 失敗".to_string()))? = None;
            Ok(())
        }

        fn run_migrations(&self) -> Result<(), SecureStorageError> {
            // 端末側 SQLite の最小スキーマを作成
            let guard = self
                .conn
                .lock()
                .map_err(|_| SecureStorageError::Database("lock 失敗".to_string()))?;
            let conn = guard
                .as_ref()
                .ok_or_else(|| SecureStorageError::Database("未接続".to_string()))?;
            // 端末側の作業実績一時保存テーブル（同期前のキャッシュ）
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS terminal_record_buffer ( \
                   id INTEGER PRIMARY KEY AUTOINCREMENT, \
                   task_id TEXT NOT NULL, \
                   device_id TEXT NOT NULL, \
                   lamport INTEGER NOT NULL, \
                   payload TEXT NOT NULL, \
                   created_at TEXT NOT NULL DEFAULT (datetime('now')) \
                 );",
            )
            .map_err(|e| SecureStorageError::Database(e.to_string()))?;
            Ok(())
        }
    }
}

// =====================================================================
// OS Keystore（Android）／DPAPI（Windows）抽象
// =====================================================================

/// OS Keystore 抽象 trait
pub trait KeyProtection: Send + Sync {
    /// 鍵を保護保存する
    fn protect(&self, alias: &str, key: &[u8]) -> Result<(), SecureStorageError>;
    /// 鍵を取り出す
    fn unprotect(&self, alias: &str) -> Result<Vec<u8>, SecureStorageError>;
}

/// 開発用のメモリ実装（実機では OS 固有実装に差し替え）
///
/// **注意**: 本実装は本番では使用しない（鍵がプロセスメモリのみに存在）。
/// 実機では Android Keystore（`tauri-plugin-keystore` 等）／DPAPI（`windows-rs`）を組み込む。
pub struct InMemoryKeyProtection {
    // alias → key
    map: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

impl Default for InMemoryKeyProtection {
    fn default() -> Self {
        // 空マップ
        Self {
            map: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl KeyProtection for InMemoryKeyProtection {
    fn protect(&self, alias: &str, key: &[u8]) -> Result<(), SecureStorageError> {
        // 鍵を保存
        self.map
            .lock()
            .map_err(|_| SecureStorageError::KeyProtection("lock 失敗".to_string()))?
            .insert(alias.to_string(), key.to_vec());
        Ok(())
    }

    fn unprotect(&self, alias: &str) -> Result<Vec<u8>, SecureStorageError> {
        // 取り出し
        let map = self
            .map
            .lock()
            .map_err(|_| SecureStorageError::KeyProtection("lock 失敗".to_string()))?;
        map.get(alias)
            .cloned()
            .ok_or_else(|| SecureStorageError::KeyProtection("alias 未登録".to_string()))
    }
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;

    // メモリ KeyProtection の保存・取り出し
    #[test]
    fn in_memory_protect_unprotect_round_trips() {
        // 実装
        let kp = InMemoryKeyProtection::default();
        // 保存
        let key = vec![0x01u8, 0x02, 0x03, 0x04];
        kp.protect("test-alias", &key).expect("ok");
        // 取り出し
        let got = kp.unprotect("test-alias").expect("ok");
        // 一致
        assert_eq!(got, key);
    }

    // 未登録 alias は KeyProtection エラー
    #[test]
    fn in_memory_unprotect_missing_alias_errors() {
        // 実装
        let kp = InMemoryKeyProtection::default();
        // 取り出し
        let r = kp.unprotect("missing");
        // エラー
        assert!(matches!(r, Err(SecureStorageError::KeyProtection(_))));
    }
}
