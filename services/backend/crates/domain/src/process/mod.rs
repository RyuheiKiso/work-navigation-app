//! プロセス（製造工程）ドメイン
//!
//! 対応 §: ロードマップ §3.4 §10.2.1 §10.2.2
//!
//! [`Step`] を頂点、[`ProcessEdge`] を辺とする有向非巡回グラフ。
//! HSM の各 state への対応はプレゼンテーション層で行う。
//! 不変条件は [`Process::create`] が型レベルで強制する（"parse, don't validate"）。

use core::fmt;

use crate::error::DomainError;
use crate::value_object::CompletionCriteria;

mod graph;

#[cfg(test)]
mod tests;

/// プロセス（工程）の識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessId(String);

impl ProcessId {
    /// 文字列から構築する
    ///
    /// # Errors
    /// 空または 256 文字超は不正。
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let v: String = value.into();
        if v.is_empty() {
            return Err(DomainError::InvalidIdentifier("ProcessId が空です"));
        }
        if v.len() > 256 {
            return Err(DomainError::InvalidIdentifier("ProcessId が長すぎます"));
        }
        Ok(Self(v))
    }

    /// 内部 &str を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// ステップ（工程内の単位作業）の識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StepId(String);

impl StepId {
    /// 文字列から構築する
    ///
    /// # Errors
    /// 空または 256 文字超は不正。
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let v: String = value.into();
        if v.is_empty() {
            return Err(DomainError::InvalidIdentifier("StepId が空です"));
        }
        if v.len() > 256 {
            return Err(DomainError::InvalidIdentifier("StepId が長すぎます"));
        }
        Ok(Self(v))
    }

    /// 内部 &str を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// ステップ（工程内の単位作業）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    id: StepId,
    label: String,
    /// 標準時間（秒）。0 は未定義扱い。
    standard_seconds: u32,
    completion: CompletionCriteria,
}

impl Step {
    /// ステップを構築する
    ///
    /// # Errors
    /// ラベルが空または 256 文字超は不正。
    pub fn new(
        id: StepId,
        label: impl Into<String>,
        standard_seconds: u32,
        completion: CompletionCriteria,
    ) -> Result<Self, DomainError> {
        let label: String = label.into();
        if label.is_empty() {
            return Err(DomainError::InvalidIdentifier("Step.label が空です"));
        }
        if label.len() > 256 {
            return Err(DomainError::InvalidIdentifier("Step.label が長すぎます"));
        }
        Ok(Self { id, label, standard_seconds, completion })
    }

    /// ステップ ID を取得する
    #[must_use]
    pub const fn id(&self) -> &StepId {
        &self.id
    }

    /// ラベルを取得する
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// 標準時間（秒）を取得する
    #[must_use]
    pub const fn standard_seconds(&self) -> u32 {
        self.standard_seconds
    }

    /// 完了条件を取得する
    #[must_use]
    pub const fn completion(&self) -> &CompletionCriteria {
        &self.completion
    }
}

/// ステップ間の遷移辺
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessEdge {
    from: StepId,
    to: StepId,
    /// 遷移条件式（任意）。`None` なら無条件遷移。
    condition: Option<String>,
}

impl ProcessEdge {
    /// 辺を構築する
    #[must_use]
    pub const fn new(from: StepId, to: StepId, condition: Option<String>) -> Self {
        Self { from, to, condition }
    }

    /// from 端点を取得する
    #[must_use]
    pub const fn from(&self) -> &StepId {
        &self.from
    }

    /// to 端点を取得する
    #[must_use]
    pub const fn to(&self) -> &StepId {
        &self.to
    }

    /// 遷移条件を取得する
    #[must_use]
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }
}

/// プロセス（工程）Aggregate
///
/// 不変条件 (parse, don't validate):
/// - Step ID は一意
/// - 全ての辺の端点は実在する Step を指す
/// - 巡回しない（DAG）
/// - 入次数 0 の Step（start）が 1 つ以上存在する
/// - 出次数 0 の Step（end）が 1 つ以上存在する
/// - 全ての Step は start から到達可能
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    id: ProcessId,
    name: String,
    steps: Vec<Step>,
    edges: Vec<ProcessEdge>,
    version: u32,
}

impl Process {
    /// プロセスを構築する
    ///
    /// 不変条件を全て検証し、違反があれば [`ProcessError`] を返す。
    ///
    /// # Errors
    /// 名称が空、または構造的に妥当でないグラフを与えた場合。
    pub fn create(
        id: ProcessId,
        name: impl Into<String>,
        steps: Vec<Step>,
        edges: Vec<ProcessEdge>,
        version: u32,
    ) -> Result<Self, ProcessError> {
        let name: String = name.into();
        if name.is_empty() {
            return Err(ProcessError::EmptyName);
        }
        graph::validate(&steps, &edges)?;
        Ok(Self { id, name, steps, edges, version })
    }

    /// プロセス ID を取得する
    #[must_use]
    pub const fn id(&self) -> &ProcessId {
        &self.id
    }

    /// 名称を取得する
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// バージョンを取得する
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// ステップ一覧を取得する
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// 辺一覧を取得する
    #[must_use]
    pub fn edges(&self) -> &[ProcessEdge] {
        &self.edges
    }

    /// start ステップ（入次数 0）一覧を返す
    #[must_use]
    pub fn start_steps(&self) -> Vec<&StepId> {
        graph::find_starts(&self.steps, &self.edges)
    }

    /// end ステップ（出次数 0）一覧を返す
    #[must_use]
    pub fn end_steps(&self) -> Vec<&StepId> {
        graph::find_ends(&self.steps, &self.edges)
    }
}

/// プロセス特有のエラー
///
/// 注: DAG 構造を保証した時点で「start から到達不能な Step」は数学的に発生しない
/// （任意の頂点から逆向き辺を辿ると DAG 上では入次数 0 の頂点に必ず到達するため）。
/// よって UnreachableStep バリアントは存在しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessError {
    /// 名称が空
    EmptyName,
    /// Step ID が重複
    DuplicateStepId(String),
    /// 辺の端点が実在しない Step を指す
    EdgeReferencesUnknownStep(String),
    /// 巡回が存在する（DAG 違反）
    Cyclic,
    /// 入次数 0 の Step が無い
    NoStartStep,
    /// 出次数 0 の Step が無い
    NoEndStep,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::EmptyName => write!(f, "プロセス名称が空です"),
            ProcessError::DuplicateStepId(id) => write!(f, "Step ID が重複しています: {id}"),
            ProcessError::EdgeReferencesUnknownStep(id) => {
                write!(f, "辺が未定義の Step を参照しています: {id}")
            }
            ProcessError::Cyclic => write!(f, "プロセスに巡回があります（DAG 違反）"),
            ProcessError::NoStartStep => write!(f, "入次数 0 の Step が存在しません"),
            ProcessError::NoEndStep => write!(f, "出次数 0 の Step が存在しません"),
        }
    }
}

impl std::error::Error for ProcessError {}

