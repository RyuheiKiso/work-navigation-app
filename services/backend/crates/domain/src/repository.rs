//! Repository trait（依存逆転）
//!
//! 対応 §: ロードマップ §9.1 §9.4 §10.6 §28
//!
//! adapter 層が本 trait を実装し、usecase 層は trait のみに依存する。
//! ドメイン層は永続化の詳細（PostgreSQL／SQLite）に依存しない。

// 値オブジェクトと Aggregate を import
use crate::error::DomainError;
use crate::task::Task;
use crate::value_object::TaskId;

// 非同期 Repository を抽象する型エイリアス
//
// `async_trait` を使わず、ここでは Future 型のジェネリクスをトレイト境界で表現する。
// 実装の詳細は adapter 層に閉じ込める。

/// Task Aggregate を読み書きするリポジトリ
///
/// usecase 層はこの trait のみに依存し、PostgreSQL／SQLite いずれの実装も差替可能とする（§9.1）。
pub trait TaskRepository: Send + Sync {
    /// 取得時に発生しうるエラー型
    ///
    /// adapter 層では sqlx エラー等を `From` で取り込む。
    type Error: std::error::Error + Send + Sync + 'static + From<DomainError>;

    /// Task を ID で取得する
    ///
    /// 戻り値が `None` の場合は未存在。
    fn find_by_id(
        &self,
        id: &TaskId,
    ) -> impl std::future::Future<Output = Result<Option<Task>, Self::Error>> + Send;

    /// Task を保存する（新規作成・更新の両用）
    fn save(
        &self,
        task: &Task,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}
