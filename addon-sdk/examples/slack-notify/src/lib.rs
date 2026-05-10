//! Slack 通知サンプルアドオン
//!
//! 対応 §: ロードマップ §17 §17.3 §17.7 §9.3.1 §31.4
//!
//! 主たる責務: アンドン通知 capability `notify:slack` を取得し、
//! Slack Webhook URL を `getConfig` で受領、`notify(channel=Chat, message)` を発射する。
//!
//! 必要 capability:
//! - `Notify(NotificationChannel::Chat)`
//! - `ConfigRead`
//!
//! Webhook URL は `slack.webhook.url` キーで取得する。

// SDK
use wna_addon_sdk::{AddonContext, AddonError, Host, NotificationChannel};

/// Andon イベント時に Slack 通知を投げる
pub fn on_andon<H: Host>(host: &H, _ctx: &AddonContext, message: &str) -> Result<(), AddonError> {
    // Webhook URL が設定されているか確認する（§17.3 ConfigRead）
    let webhook = host
        .get_config("slack.webhook.url")?
        .ok_or(AddonError::InvalidArgument(
            "slack.webhook.url が未設定です".to_string(),
        ))?;
    // ロギング（既定許可、§17.3）
    host.log("info", &format!("slack-notify: posting to {webhook}"));
    // 通知発射（§17.3 Notify capability）
    host.notify(NotificationChannel::Chat, message)?;
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
    use wna_addon_sdk::TaskInfo;

    // テストホスト
    struct TestHost {
        // 設定値
        config: RefCell<Vec<(String, Option<String>)>>,
        // 通知履歴
        notifications: RefCell<Vec<(NotificationChannel, String)>>,
        // ログ
        logs: RefCell<Vec<(String, String)>>,
    }

    impl Host for TestHost {
        fn get_current_task(&self) -> Result<TaskInfo, AddonError> {
            // ダミー
            Ok(TaskInfo {
                id: "t-1".to_string(),
                state: "Running".to_string(),
            })
        }
        fn append_record(&self, _task_id: &str, _payload: &str) -> Result<(), AddonError> {
            // 未使用
            Ok(())
        }
        fn notify(
            &self,
            channel: NotificationChannel,
            message: &str,
        ) -> Result<(), AddonError> {
            // 履歴記録
            self.notifications
                .borrow_mut()
                .push((channel, message.to_string()));
            Ok(())
        }
        fn log(&self, level: &str, message: &str) {
            // 履歴
            self.logs
                .borrow_mut()
                .push((level.to_string(), message.to_string()));
        }
        fn get_config(&self, key: &str) -> Result<Option<String>, AddonError> {
            // 設定検索
            for (k, v) in self.config.borrow().iter() {
                if k == key {
                    return Ok(v.clone());
                }
            }
            // 未設定
            Ok(None)
        }
        fn now(&self) -> i64 {
            // 固定
            0
        }
    }

    // 設定済み: notify が呼ばれる
    #[test]
    fn posts_when_webhook_configured() {
        // ホスト
        let host = TestHost {
            config: RefCell::new(vec![(
                "slack.webhook.url".to_string(),
                Some("https://hooks.slack.com/services/T0/B0/X".to_string()),
            )]),
            notifications: RefCell::new(Vec::new()),
            logs: RefCell::new(Vec::new()),
        };
        // コンテキスト
        let ctx = AddonContext {
            addon_id: "slack-notify".to_string(),
            locale: "ja".to_string(),
        };
        // 実行
        on_andon(&host, &ctx, "ライン停止: 設備異常").expect("ok");
        // 通知が 1 件発射
        assert_eq!(host.notifications.borrow().len(), 1);
        // チャネルは Chat
        assert_eq!(host.notifications.borrow()[0].0, NotificationChannel::Chat);
        // メッセージが含まれる
        assert!(host.notifications.borrow()[0].1.contains("ライン停止"));
    }

    // 未設定: エラー
    #[test]
    fn errors_when_webhook_missing() {
        // ホスト（config 空）
        let host = TestHost {
            config: RefCell::new(vec![]),
            notifications: RefCell::new(Vec::new()),
            logs: RefCell::new(Vec::new()),
        };
        let ctx = AddonContext {
            addon_id: "slack-notify".to_string(),
            locale: "ja".to_string(),
        };
        // 実行
        let r = on_andon(&host, &ctx, "test");
        // InvalidArgument エラー
        assert!(matches!(r, Err(AddonError::InvalidArgument(_))));
    }
}
