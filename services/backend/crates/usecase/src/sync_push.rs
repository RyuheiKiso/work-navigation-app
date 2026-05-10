//! 端末側 sync push ループ
//!
//! 対応 §: ロードマップ §10.6 §10.6.1 §10.6.2 §27 F-002 §29 R-016
//!
//! TLA+ 仕様の `EnqueueSend → Transmit → Receive` 経路を usecase 層で表現する。
//! 端末側 G-Set バッファ → 送信キュー → ネットワーク送信 → サーバ受領 を担う。

// ドメイン
use wna_domain::{SyncEvent};

// =====================================================================
// trait: 端末バッファ／送信キュー／ネットワーク
// =====================================================================

/// 端末側で発生したイベントを保持するバッファ（G-Set 同等）
pub trait TerminalEventBuffer: Send + Sync {
    /// 実装固有エラー
    type Error: std::error::Error + Send + Sync + 'static;

    /// バッファ先頭の未送信イベントを取得する
    fn peek(
        &self,
    ) -> impl std::future::Future<Output = Result<Option<SyncEvent>, Self::Error>> + Send;

    /// 先頭イベントを削除する（送信完了確認後に呼ぶ）
    fn dequeue(
        &self,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

/// イベントをサーバへ送信する境界（HTTPS REST 等）
pub trait SyncTransport: Send + Sync {
    /// 実装固有エラー
    type Error: std::error::Error + Send + Sync + 'static;

    /// 単一イベントを送信する
    ///
    /// 失敗時は呼び出し側がリトライする。
    fn send(
        &self,
        event: &SyncEvent,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

// =====================================================================
// SyncPushUseCase
// =====================================================================

/// 端末側 push ループのエラー
#[derive(Debug)]
pub enum SyncPushError<B, T>
where
    B: std::error::Error + Send + Sync + 'static,
    T: std::error::Error + Send + Sync + 'static,
{
    /// バッファエラー
    Buffer(B),
    /// 送信エラー（リトライ対象）
    Transport(T),
}

// Display
impl<B, T> std::fmt::Display for SyncPushError<B, T>
where
    B: std::error::Error + Send + Sync + 'static,
    T: std::error::Error + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 分岐
        match self {
            SyncPushError::Buffer(e) => write!(f, "バッファ: {e}"),
            SyncPushError::Transport(e) => write!(f, "送信: {e}"),
        }
    }
}

// Error
impl<B, T> std::error::Error for SyncPushError<B, T>
where
    B: std::error::Error + Send + Sync + 'static,
    T: std::error::Error + Send + Sync + 'static,
{
}

/// SyncPushUseCase
///
/// バッファ→送信→確認→削除 を 1 ステップ進める。連続呼び出しでループとして機能する。
pub struct SyncPushUseCase<B: TerminalEventBuffer, T: SyncTransport> {
    /// バッファ
    buffer: B,
    /// 送信境界
    transport: T,
}

impl<B: TerminalEventBuffer, T: SyncTransport> SyncPushUseCase<B, T> {
    /// 新規構築
    pub const fn new(buffer: B, transport: T) -> Self {
        // フィールドを保持
        Self { buffer, transport }
    }

    /// 1 ステップ進める
    ///
    /// 戻り値の `bool` は「実際にイベントを送信したか」（バッファ空なら false）。
    /// `Inv_NoEventLoss`（INV-01）を満たすため、送信成功確認 **後にのみ** dequeue する。
    pub async fn step(&self) -> Result<bool, SyncPushError<B::Error, T::Error>> {
        // バッファ先頭を peek
        let Some(ev) = self
            .buffer
            .peek()
            .await
            .map_err(SyncPushError::Buffer)?
        else {
            // 空 → 何もしない
            return Ok(false);
        };
        // 送信を試みる（失敗時は dequeue せずに戻す＝再送可能）
        self.transport
            .send(&ev)
            .await
            .map_err(SyncPushError::Transport)?;
        // 送信成功時のみ dequeue（INV-01）
        self.buffer
            .dequeue()
            .await
            .map_err(SyncPushError::Buffer)?;
        // 1 件処理した
        Ok(true)
    }

    /// バッファが空になるまで連続実行する
    ///
    /// 失敗時は途中で停止し、エラーを返す（呼び出し側がリトライ）。
    pub async fn drain(&self) -> Result<usize, SyncPushError<B::Error, T::Error>> {
        // 処理件数
        let mut n = 0;
        // バッファ空まで連続
        while self.step().await? {
            // カウント
            n += 1;
            // 安全弁: 1 ステップで最大 10000 件まで（無限ループ防止）
            if n >= 10000 {
                break;
            }
        }
        // 処理件数を返す
        Ok(n)
    }
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;
    use std::sync::Mutex;
    use wna_domain::{DeviceId, LamportTimestamp, TaskId};

    // メモリバッファ
    #[derive(Default)]
    struct MemBuf {
        // FIFO
        q: Mutex<Vec<SyncEvent>>,
    }

    #[derive(Debug)]
    struct E;
    impl std::fmt::Display for E {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // 表示
            write!(f, "mem error")
        }
    }
    impl std::error::Error for E {}

    impl MemBuf {
        fn push(&self, ev: SyncEvent) {
            // 末尾に追加
            self.q.lock().expect("lock").push(ev);
        }
    }

    impl TerminalEventBuffer for MemBuf {
        type Error = E;
        async fn peek(&self) -> Result<Option<SyncEvent>, Self::Error> {
            // 先頭を返す
            Ok(self.q.lock().expect("lock").first().cloned())
        }
        async fn dequeue(&self) -> Result<(), Self::Error> {
            // 先頭を削除
            let mut q = self.q.lock().expect("lock");
            if !q.is_empty() {
                q.remove(0);
            }
            Ok(())
        }
    }

    // 常に成功する送信
    struct OkTransport {
        // 受領履歴
        log: Mutex<Vec<SyncEvent>>,
    }

    impl SyncTransport for OkTransport {
        type Error = E;
        async fn send(&self, ev: &SyncEvent) -> Result<(), Self::Error> {
            // 履歴
            self.log.lock().expect("lock").push(ev.clone());
            Ok(())
        }
    }

    // 失敗する送信
    struct FailTransport;
    impl SyncTransport for FailTransport {
        type Error = E;
        async fn send(&self, _ev: &SyncEvent) -> Result<(), Self::Error> {
            // 失敗
            Err(E)
        }
    }

    // 補助: テスト用イベント
    fn ev(seed: u64) -> SyncEvent {
        // 値オブジェクト
        let dev = DeviceId::new(format!("d-{seed}")).expect("valid");
        let ts = LamportTimestamp::from_u64(seed);
        let task = TaskId::new(format!("t-{seed}")).expect("valid");
        // 構築
        SyncEvent::record(dev, ts, task, "{}")
    }

    // step: 空バッファでは false
    #[tokio::test]
    async fn step_returns_false_on_empty() {
        let buf = MemBuf::default();
        let tx = OkTransport {
            log: Mutex::new(Vec::new()),
        };
        let uc = SyncPushUseCase::new(buf, tx);
        assert!(!uc.step().await.expect("ok"));
    }

    // step: 成功時は dequeue され、送信履歴に 1 件残る
    #[tokio::test]
    async fn step_dequeues_on_success() {
        let buf = MemBuf::default();
        buf.push(ev(1));
        let tx = OkTransport {
            log: Mutex::new(Vec::new()),
        };
        let uc = SyncPushUseCase::new(buf, tx);
        // 1 ステップ
        assert!(uc.step().await.expect("ok"));
        // 送信履歴に 1 件
        assert_eq!(uc.transport.log.lock().expect("lock").len(), 1);
        // バッファは空
        assert!(uc.buffer.q.lock().expect("lock").is_empty());
    }

    // step: 失敗時は dequeue されない（INV-01 NoEventLoss）
    #[tokio::test]
    async fn step_keeps_event_on_failure() {
        let buf = MemBuf::default();
        buf.push(ev(1));
        let tx = FailTransport;
        let uc = SyncPushUseCase::new(buf, tx);
        // 失敗
        assert!(uc.step().await.is_err());
        // バッファに残っている
        assert_eq!(uc.buffer.q.lock().expect("lock").len(), 1);
    }

    // drain: 複数件すべて送信される
    #[tokio::test]
    async fn drain_processes_all() {
        let buf = MemBuf::default();
        for i in 1..=5 {
            buf.push(ev(i));
        }
        let tx = OkTransport {
            log: Mutex::new(Vec::new()),
        };
        let uc = SyncPushUseCase::new(buf, tx);
        let n = uc.drain().await.expect("ok");
        assert_eq!(n, 5);
        assert_eq!(uc.transport.log.lock().expect("lock").len(), 5);
    }
}
