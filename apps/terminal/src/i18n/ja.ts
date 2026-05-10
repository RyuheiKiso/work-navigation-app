// 対応 §: ロードマップ §11.3 §28（用語集と完全一致）
// 日本語ロケール辞書。主要 UI 文言と §28 用語の表記を定義する。

/** 日本語辞書 */
export const ja = {
  // §28 用語集と一致する基本語彙
  term: {
    task: '作業',
    process: '工程',
    procedure: '手順',
    step: '動作',
    completion_criteria: '完了条件',
    precondition: '開始条件',
    record: '実績',
    andon: 'アンドン',
    audit_log: '監査ログ',
    flow: 'フロー',
    master: 'マスタ',
    addon: 'アドオン'
  },
  // 共通アクション
  action: {
    start: '開始',
    suspend: '中断',
    resume: '再開',
    complete: '完了',
    abort: '取消'
  },
  // 作業ナビ画面
  task: {
    state_label: '状態',
    progress_label: '進捗（Lamport）',
    completion_criteria_label: '完了条件',
    start_button: '開始する',
    complete_button: '完了する',
    completed_label: '完了済み',
    suspended_label: '中断中',
    aria_complete: '作業を完了する'
  },
  // 状態名（HSM 状態と一致）
  state: {
    Idle: '初期',
    Ready: '開始可能',
    Running: '実行中',
    Suspended: '中断',
    Exception: '例外',
    Completed: '完了',
    Failed: '失敗',
    Aborted: '取消'
  },
  // §20.1 エラー表現（人を責めない）
  error: {
    precondition_not_satisfied: '開始条件が満たされていません',
    completion_criteria_not_met: '完了条件が満たされていません',
    invalid_state: '不正な状態遷移です',
    network_offline: 'ネットワークに接続できません',
    sync_pending: '同期保留中（復旧後に自動同期されます）'
  },
  // 完了条件種別
  completion: {
    manual: '人手判定',
    photo: '写真証跡'
  }
} as const;
