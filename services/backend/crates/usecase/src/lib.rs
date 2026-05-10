//! work-navigation-app ユースケース層
//!
//! 対応 §: ロードマップ §9.1 §10.1 §10.6.1
//!
//! ドメイン層に対する操作を **アプリケーションサービス** として表現する。
//! 副作用は trait 経由で adapter 層に委譲する（依存逆転）。

// 子モジュール宣言（責務単位で分割し 1 ファイル ≤ 500 行を維持）
// 作業を開始するユースケース（§10.1）
pub mod start_task;
// 作業実績を追記するユースケース（§10.6.1 G-Set）
pub mod append_record;
// ログイン（§10.5）
pub mod login;
// 順序情報受領（§10.3.2 §10.3.1 Idempotency-Key）
pub mod receive_order;
// 端末側 sync push ループ（§10.6 §27 F-002）
pub mod sync_push;

// 上位から import しやすいよう代表型を再エクスポートする
pub use append_record::{
    AppendRecordCommand, AppendRecordError, AppendRecordUseCase, RecordRepository,
};
pub use start_task::{StartTaskCommand, StartTaskError, StartTaskUseCase};
pub use login::{
    CredentialRepository, LoginCommand, LoginError, LoginUseCase, SessionFactory,
};
pub use receive_order::{
    OrderRepository, ReceiveOrderCommand, ReceiveOrderError, ReceiveOrderUseCase,
};
pub use sync_push::{
    SyncPushError, SyncPushUseCase, SyncTransport, TerminalEventBuffer,
};
