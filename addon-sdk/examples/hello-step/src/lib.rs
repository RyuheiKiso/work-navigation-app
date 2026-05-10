//! Hello Step サンプルアドオン
//!
//! 対応 §: ロードマップ §17 §17.3 §17.7 §19.4.2
//!
//! 「step 完了時にログを 1 行残す」だけの最小アドオン。
//! 必要 capability は `TaskRead` のみ。

// SDK の trait を import
use wna_addon_sdk::{AddonContext, AddonError, Host};

/// アドオンのエントリポイント
///
/// 実装は WASM ランタイムから「step_completed」イベント時に呼ばれる関数として登録する想定。
/// 本骨格では、ホストが任意のタイミングで呼び出すと仮定する。
pub fn on_step_completed<H: Host>(host: &H, _ctx: &AddonContext) -> Result<(), AddonError> {
    // 現在の作業情報を取得
    let task = host.get_current_task()?;
    // ロギング（既定許可、§17.3）
    host.log("info", &format!("hello, step completed: task={}", task.id));
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
    // 標準 Cell（テスト用ホストの可変状態）
    use std::cell::RefCell;
    // SDK 型
    use wna_addon_sdk::{NotificationChannel, TaskInfo};

    // テスト用 Host 実装
    struct TestHost {
        // 取得すべき作業
        task: TaskInfo,
        // log 呼び出し履歴
        logs: RefCell<Vec<(String, String)>>,
    }

    // Host 実装
    impl Host for TestHost {
        fn get_current_task(&self) -> Result<TaskInfo, AddonError> {
            // 保持している task をクローンして返す
            Ok(self.task.clone())
        }
        fn append_record(&self, _task_id: &str, _payload: &str) -> Result<(), AddonError> {
            // 本テストでは未使用
            Ok(())
        }
        fn notify(&self, _channel: NotificationChannel, _message: &str) -> Result<(), AddonError> {
            // 本テストでは未使用
            Ok(())
        }
        fn log(&self, level: &str, message: &str) {
            // 履歴に push
            self.logs
                .borrow_mut()
                .push((level.to_string(), message.to_string()));
        }
        fn get_config(&self, _key: &str) -> Result<Option<String>, AddonError> {
            // 本テストでは未使用
            Ok(None)
        }
        fn now(&self) -> i64 {
            // 固定値
            0
        }
    }

    // on_step_completed: log が 1 件出ること
    #[test]
    fn logs_step_completion() {
        // テストホスト
        let host = TestHost {
            task: TaskInfo {
                id: "t-1".to_string(),
                state: "Running".to_string(),
            },
            logs: RefCell::new(Vec::new()),
        };
        // コンテキスト
        let ctx = AddonContext {
            addon_id: "hello-step".to_string(),
            locale: "ja".to_string(),
        };
        // 実行
        on_step_completed(&host, &ctx).expect("ok");
        // log が 1 件
        assert_eq!(host.logs.borrow().len(), 1);
        // info レベル
        assert_eq!(host.logs.borrow()[0].0, "info");
        // メッセージに task id を含む
        assert!(host.logs.borrow()[0].1.contains("t-1"));
    }
}
