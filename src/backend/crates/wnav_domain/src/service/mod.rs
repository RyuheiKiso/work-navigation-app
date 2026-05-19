// ドメインサービスモジュール
// wnav_domain のドメインサービス・アプリケーションサービス Trait を集約する。

pub mod hash_chain_service;
pub mod iqc_decision_service;
pub mod json_logic_evaluator;
pub mod master_version_service;
pub mod step_engine_service;
pub mod work_execution_service;
pub mod work_execution_service_impl;

// 主要な型を再エクスポートして使いやすくする
pub use hash_chain_service::{ChainVerifyResult, HashChainService};
pub use iqc_decision_service::IqcDecisionService;
pub use json_logic_evaluator::{JsonLogicError, JsonLogicEvaluator};
pub use master_version_service::{DryRunResult, MasterVersionService};
pub use step_engine_service::StepEngineService;
pub use work_execution_service::{
    CompleteStepCmd, CompleteWorkCmd, ResumeCmd, StartWorkCmd, Suspension, SuspendCmd,
    WorkExecutionService,
};
pub use work_execution_service_impl::WorkExecutionServiceImpl;
