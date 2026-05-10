//! Start Task ユースケース
//!
//! 対応 §: ロードマップ §10.1 §3.1.1 §10.6.1
//!
//! 既存の Task を Repository から取得し、`start()` を呼び出して状態遷移させる。
//! 副作用は `TaskRepository` trait に委譲する。

// ドメインから必要な型を import
use wna_domain::{DomainError, Task, TaskId, TaskRepository};

/// Start Task コマンド入力
///
/// プレゼンテーション層が DTO から組み立てる値。
#[derive(Debug, Clone)]
pub struct StartTaskCommand {
    /// 開始対象の TaskId
    pub task_id: TaskId,
}

/// Start Task ユースケースのエラー型
#[derive(Debug)]
pub enum StartTaskError<E: std::error::Error + Send + Sync + 'static> {
    /// 対象 Task が存在しない
    NotFound,
    /// ドメイン規則違反
    Domain(DomainError),
    /// リポジトリ層のエラー
    Repository(E),
}

// Display 実装（境界層でログ出力する用途）
impl<E: std::error::Error + Send + Sync + 'static> std::fmt::Display for StartTaskError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // バリアントごとに人間可読メッセージを返す
        match self {
            // 未存在
            StartTaskError::NotFound => write!(f, "対象の作業が存在しません"),
            // ドメインエラーは Display を委譲
            StartTaskError::Domain(e) => write!(f, "ドメイン規則違反: {e}"),
            // リポジトリエラーは Display を委譲
            StartTaskError::Repository(e) => write!(f, "リポジトリエラー: {e}"),
        }
    }
}

// Error 実装
impl<E: std::error::Error + Send + Sync + 'static> std::error::Error for StartTaskError<E> {}

/// Start Task ユースケース実装
///
/// 型パラメータ `R` は `TaskRepository` 実装。
pub struct StartTaskUseCase<R: TaskRepository> {
    // 注入される Repository（DI コンテナで構築）
    repository: R,
}

impl<R: TaskRepository> StartTaskUseCase<R> {
    /// 新規ユースケースを構築する（DI 用）
    pub const fn new(repository: R) -> Self {
        // Repository を保持するだけの単純なコンストラクタ
        Self { repository }
    }

    /// コマンドを実行する
    ///
    /// # Errors
    /// 対象未存在／ドメイン規則違反／リポジトリ層失敗のいずれか。
    pub async fn execute(&self, cmd: StartTaskCommand) -> Result<Task, StartTaskError<R::Error>> {
        // 対象 Task を取得する
        let mut task = self
            .repository
            .find_by_id(&cmd.task_id)
            .await
            .map_err(StartTaskError::Repository)?
            .ok_or(StartTaskError::NotFound)?;

        // 開始する（前提条件を満たしていなければエラー）
        task.start().map_err(StartTaskError::Domain)?;

        // 永続化する
        self.repository
            .save(&task)
            .await
            .map_err(StartTaskError::Repository)?;

        // 更新後の Task を返す（プレゼンテーション層へ）
        Ok(task)
    }
}

// =====================================================================
// 単体テスト（§13.1 単体）
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象とドメイン依存
    use super::*;
    use wna_domain::{CompletionCriteria, DeviceId, Evidence};

    // 単純なメモリ Repository（テスト用）
    #[derive(Default)]
    struct MemoryRepo {
        // 1 件だけ持つ最小ストア
        slot: tokio::sync::Mutex<Option<Task>>,
    }

    // メモリ Repository 用エラー型
    #[derive(Debug)]
    struct MemError(DomainError);

    // Display
    impl std::fmt::Display for MemError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // ドメインエラーを委譲表示
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

    // TaskRepository 実装
    impl TaskRepository for MemoryRepo {
        type Error = MemError;

        async fn find_by_id(&self, _id: &TaskId) -> Result<Option<Task>, Self::Error> {
            // ロックを取得してクローンを返す
            Ok(self.slot.lock().await.clone())
        }

        async fn save(&self, task: &Task) -> Result<(), Self::Error> {
            // ロックを取得して上書き保存
            *self.slot.lock().await = Some(task.clone());
            // 正常
            Ok(())
        }
    }

    // 補助: 前提条件を充足した Task を作る
    fn make_ready_task() -> Task {
        // ID を生成
        let id = TaskId::new("u-task-1").expect("valid id");
        // Device を生成
        let dev = DeviceId::new("u-dev").expect("valid id");
        // 完了条件 Manual で構築
        let mut t = Task::create(id, CompletionCriteria::Manual, dev);
        // 前提条件を満たして Ready に上げる
        t.mark_precondition_satisfied().expect("ok");
        // 完成
        t
    }

    // execute: 対象未存在の場合は NotFound
    #[tokio::test]
    async fn execute_returns_not_found_when_missing() {
        // 空のリポジトリ
        let repo = MemoryRepo::default();
        // ユースケース
        let uc = StartTaskUseCase::new(repo);
        // 任意の TaskId を入れて実行
        let cmd = StartTaskCommand {
            task_id: TaskId::new("missing").expect("valid id"),
        };
        // エラー判定
        let r = uc.execute(cmd).await;
        // NotFound を期待
        assert!(matches!(r, Err(StartTaskError::NotFound)));
    }

    // execute: Ready 状態から呼ぶと Running になる
    #[tokio::test]
    async fn execute_transitions_ready_to_running() {
        // テスト用 Task をセットアップ
        let repo = MemoryRepo::default();
        let task = make_ready_task();
        repo.save(&task).await.expect("save");
        // ユースケース
        let uc = StartTaskUseCase::new(repo);
        // 実行
        let cmd = StartTaskCommand { task_id: task.id().clone() };
        let updated = uc.execute(cmd).await.expect("ok");
        // Running に遷移していること
        assert_eq!(updated.state(), wna_domain::TaskState::Running);
    }

    // 完了条件未充足での Evidence は Domain エラーになる（参考: 直接 complete のパス）
    #[test]
    fn evidence_default_does_not_satisfy_manual() {
        // Manual 完了条件
        let cri = CompletionCriteria::Manual;
        // 空証跡では満たされない
        assert!(!cri.is_met(&Evidence::default()));
    }
}
