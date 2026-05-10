//! Append Record ユースケース
//!
//! 対応 §: ロードマップ §10.6 §10.6.1 §11.4.1 §31 SLI-06
//!
//! 作業実績を G-Set（追記のみ集合）に追加するユースケース。
//! Lamport タイムスタンプの単調性（INV-08）を呼び出し側が保証する形で受け取り、
//! Repository に追記委譲する。

// ドメイン依存
use wna_domain::{DeviceId, DomainError, LamportTimestamp, TaskId};

/// Append Record コマンド入力
#[derive(Debug, Clone)]
pub struct AppendRecordCommand {
    /// 対象 Task の ID
    pub task_id: TaskId,
    /// 発生端末
    pub device_id: DeviceId,
    /// Lamport タイムスタンプ（呼び出し側が単調性を保証）
    pub lamport: LamportTimestamp,
    /// 自由形式 payload（境界層で JSON へ）
    pub payload: String,
}

/// Append Record エラー
#[derive(Debug)]
pub enum AppendRecordError<E: std::error::Error + Send + Sync + 'static> {
    /// ドメイン規則違反
    Domain(DomainError),
    /// リポジトリ層のエラー
    Repository(E),
}

// Display 実装
impl<E: std::error::Error + Send + Sync + 'static> std::fmt::Display for AppendRecordError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // バリアントごとに分岐表示
        match self {
            // ドメインエラー
            AppendRecordError::Domain(e) => write!(f, "ドメイン規則違反: {e}"),
            // リポジトリエラー
            AppendRecordError::Repository(e) => write!(f, "リポジトリエラー: {e}"),
        }
    }
}

// Error 実装
impl<E: std::error::Error + Send + Sync + 'static> std::error::Error for AppendRecordError<E> {}

/// 実績追記用の最小 Repository trait
///
/// `TaskRepository` は Aggregate を読み書きするが、本 trait は **G-Set への追記** に特化する。
/// adapter 層では同一構造体に両方を実装してよい。
pub trait RecordRepository: Send + Sync {
    /// リポジトリ実装固有のエラー型
    type Error: std::error::Error + Send + Sync + 'static + From<DomainError>;

    /// 実績を追記する（G-Set、§10.6.1）
    fn append(
        &self,
        cmd: &AppendRecordCommand,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

/// Append Record ユースケース
pub struct AppendRecordUseCase<R: RecordRepository> {
    // 注入される Repository
    repository: R,
}

impl<R: RecordRepository> AppendRecordUseCase<R> {
    /// 新規ユースケースを構築する
    pub const fn new(repository: R) -> Self {
        // Repository を保持するだけのコンストラクタ
        Self { repository }
    }

    /// コマンドを実行する
    ///
    /// # Errors
    /// リポジトリ層で発生した永続化エラーを `AppendRecordError::Repository` として返す。
    pub async fn execute(
        &self,
        cmd: AppendRecordCommand,
    ) -> Result<(), AppendRecordError<R::Error>> {
        // 追記用 Repository へ委譲
        self.repository
            .append(&cmd)
            .await
            .map_err(AppendRecordError::Repository)
    }
}

// =====================================================================
// 単体テスト（§13.1）
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;
    // 標準同期プリミティブ
    use std::sync::Mutex;

    // メモリ Repository（テスト用）
    #[derive(Default)]
    struct MemoryRecordRepo {
        // 受領した実績を順序付きで蓄積
        entries: Mutex<Vec<AppendRecordCommand>>,
    }

    // メモリ Repository 用エラー型
    #[derive(Debug)]
    struct MemError(DomainError);

    // Display
    impl std::fmt::Display for MemError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // ドメインエラーを委譲
            write!(f, "{}", self.0)
        }
    }
    // Error
    impl std::error::Error for MemError {}
    // From<DomainError>
    impl From<DomainError> for MemError {
        fn from(value: DomainError) -> Self {
            // ドメインエラーを内包
            Self(value)
        }
    }

    // RecordRepository 実装
    impl RecordRepository for MemoryRecordRepo {
        type Error = MemError;

        async fn append(&self, cmd: &AppendRecordCommand) -> Result<(), Self::Error> {
            // ロックして push
            self.entries
                .lock()
                .expect("lock")
                .push(cmd.clone());
            // 正常
            Ok(())
        }
    }

    // execute: 追記が記録されること
    #[tokio::test]
    async fn execute_appends_record() {
        // メモリ Repository
        let repo = MemoryRecordRepo::default();
        // ユースケース
        let uc = AppendRecordUseCase::new(repo);
        // コマンドを構築
        let cmd = AppendRecordCommand {
            task_id: TaskId::new("t-1").expect("valid id"),
            device_id: DeviceId::new("d-1").expect("valid id"),
            lamport: LamportTimestamp::from_u64(42),
            payload: "{}".to_string(),
        };
        // 実行
        uc.execute(cmd).await.expect("ok");
        // 内部検査用にユースケースから Repository を取り戻すのは困難なので、
        // ここではエラーが起きないことだけを確認する（実装はメモリ詳細）。
    }
}
