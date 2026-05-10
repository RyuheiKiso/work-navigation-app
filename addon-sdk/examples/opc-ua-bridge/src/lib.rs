//! OPC UA Bridge サンプルアドオン
//!
//! 対応 §: ロードマップ §17 §17.7 §10.3.1 §10.3.4 §27 F-005
//!
//! 主たる責務: OPC UA タグ値を取得し、`appendRecord` で実績として連結する。
//!
//! 必要 capability:
//! - `TaskRead`
//! - `TaskWrite`
//! - `NetOutbound("opc-ua-server.local")` 等のホスト指定
//! - `ConfigRead`
//!
//! 本骨格では実 OPC UA 通信は行わず、設定された擬似タグ値を `appendRecord` 経由で連結する流れのみを示す。
//! 実通信は将来 `opcua` crate を本 crate に組み込み、adapter 層に押し込む。

// SDK
use wna_addon_sdk::{AddonContext, AddonError, Host};

/// OPC UA タグ収集結果（簡略表現）
#[derive(Debug, Clone)]
pub struct TagSample {
    /// タグ ID
    pub tag_id: String,
    /// 値（文字列で受領、JSON へ載せる）
    pub value: String,
}

/// OPC UA タグから収集したサンプル列を、現在の作業実績として追記する
pub fn append_tag_samples<H: Host>(
    host: &H,
    _ctx: &AddonContext,
    samples: &[TagSample],
) -> Result<(), AddonError> {
    // 現在作業を取得（capability: TaskRead）
    let task = host.get_current_task()?;
    // 端末・サーバ間の連携可否ログ（既定許可）
    host.log("info", &format!(
        "opc-ua-bridge: appending {} samples to task={}",
        samples.len(),
        task.id
    ));
    // 各サンプルを実績として書き込む（capability: TaskWrite）
    for s in samples {
        // payload を JSON 風文字列で組み立てる
        let payload = format!(
            r#"{{"source":"opc-ua","tag":"{}","value":"{}"}}"#,
            s.tag_id, s.value
        );
        // 追記
        host.append_record(&task.id, &payload)?;
    }
    // 正常終了
    Ok(())
}

// =====================================================================
// 単体テスト（§13.1）
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;
    // 標準
    use std::cell::RefCell;
    // SDK
    use wna_addon_sdk::{NotificationChannel, TaskInfo};

    // テストホスト
    struct TestHost {
        // append_record の受領履歴
        records: RefCell<Vec<(String, String)>>,
        // ログ
        logs: RefCell<Vec<String>>,
    }

    impl Host for TestHost {
        fn get_current_task(&self) -> Result<TaskInfo, AddonError> {
            // ダミー
            Ok(TaskInfo {
                id: "task-bridge-1".to_string(),
                state: "Running".to_string(),
            })
        }
        fn append_record(&self, task_id: &str, payload: &str) -> Result<(), AddonError> {
            // 履歴
            self.records
                .borrow_mut()
                .push((task_id.to_string(), payload.to_string()));
            Ok(())
        }
        fn notify(
            &self,
            _channel: NotificationChannel,
            _message: &str,
        ) -> Result<(), AddonError> {
            // 未使用
            Ok(())
        }
        fn log(&self, _level: &str, message: &str) {
            // ログ記録
            self.logs.borrow_mut().push(message.to_string());
        }
        fn get_config(&self, _key: &str) -> Result<Option<String>, AddonError> {
            // 未使用
            Ok(None)
        }
        fn now(&self) -> i64 {
            // 固定
            0
        }
    }

    // 複数サンプルを 1 件ずつ append_record する
    #[test]
    fn appends_each_sample() {
        // ホスト
        let host = TestHost {
            records: RefCell::new(Vec::new()),
            logs: RefCell::new(Vec::new()),
        };
        // コンテキスト
        let ctx = AddonContext {
            addon_id: "opc-ua-bridge".to_string(),
            locale: "ja".to_string(),
        };
        // 2 サンプル
        let samples = vec![
            TagSample {
                tag_id: "ns=2;s=Boiler/Temperature".to_string(),
                value: "85.4".to_string(),
            },
            TagSample {
                tag_id: "ns=2;s=Boiler/Pressure".to_string(),
                value: "1.21".to_string(),
            },
        ];
        // 実行
        append_tag_samples(&host, &ctx, &samples).expect("ok");
        // 2 件追記
        assert_eq!(host.records.borrow().len(), 2);
        // 各 payload に source=opc-ua が含まれる
        for (_, payload) in host.records.borrow().iter() {
            assert!(payload.contains("opc-ua"));
        }
    }

    // サンプル 0 件でも正常終了（実績は追記されない）
    #[test]
    fn no_samples_results_in_no_records() {
        // ホスト
        let host = TestHost {
            records: RefCell::new(Vec::new()),
            logs: RefCell::new(Vec::new()),
        };
        let ctx = AddonContext {
            addon_id: "opc-ua-bridge".to_string(),
            locale: "ja".to_string(),
        };
        // 空サンプル
        append_tag_samples(&host, &ctx, &[]).expect("ok");
        // 0 件
        assert_eq!(host.records.borrow().len(), 0);
        // ログは 1 件出ている
        assert_eq!(host.logs.borrow().len(), 1);
    }
}
