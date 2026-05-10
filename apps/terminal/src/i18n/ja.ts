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
    aria_complete: '作業を完了する',
    today_list_title: '当日のタスク',
    select_task_prompt: 'タスクを開始してください',
    start_button_short: '開始',
    suspend_button: '中断',
    resume_button: '再開',
    andon_button: 'アンドン',
    all_steps_done: '全ステップ完了',
    step_map_title: 'ステップマップ',
    storage_title: 'ストレージ',
    voice_section_title: '音声コマンド',
    voice_input_placeholder: '開始 / 完了 / 中断',
    voice_recognize_button: '認識',
    voice_unrecognized: '音声コマンドが認識できませんでした',
    no_andon: '異常なし',
    andon_severity_prefix: 'アンドン Lv.',
    andon_default_message: '部材切れ — 班長に応援要請しました',
    suspend_default_message: '一時中断中',
    next_action_prefix: '次の動作',
    overrun_label: '超過',
    standard_time_prefix: '標準時間',
    progress_aria_label: '作業全体の進捗',
    peek_next_title: '次の動作',
    peek_next_end: '次の動作はありません'
  },
  // メディアスロット文言
  media: {
    image: '写真',
    video: '動画',
    diagram: '図面'
  },
  // ショートカット
  shortcut: {
    complete: 'ステップを完了',
    suspend_resume: '中断 / 再開を切替',
    andon: 'アンドンを発火',
    toggle_task_drawer: 'タスク一覧を開閉',
    toggle_step_map: 'ステップマップを開閉',
    cycle_theme: 'テーマを循環 (standard → outdoor → dark → auto)',
    show_help: 'ショートカット一覧を表示',
    close_dialogs: 'ダイアログを閉じる',
    help_title: 'キーボードショートカット',
    close_label: '閉じる'
  },
  // ヘッダ・シェル共通
  shell: {
    logout: 'ログアウト',
    state_prefix: '状態',
    task_id_prefix: 'タスク',
    task_id_unselected: '未選択',
    toggle_task_drawer: 'タスク一覧の表示切替',
    toggle_step_map_drawer: 'ステップマップの表示切替',
    theme_label: '表示モード',
    shortcuts_label: 'ショートカット一覧'
  },
  // ログイン画面
  login: {
    title: 'ログイン',
    subtitle: 'work-navigation-app 端末',
    backend_url_label: 'バックエンド URL',
    user_id_label: 'ユーザ ID',
    password_label: 'パスワード',
    submit: 'ログイン',
    submit_busy: '処理中…',
    demo_banner: 'デモモード — 本番環境では利用しないでください',
    demo_users: 'デモユーザ: alice / bob / charlie（パスワード: hello-world）'
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
    sync_pending: '同期保留中（復旧後に自動同期されます）',
    // §10.5 / §10.6 API 失敗の分類別ユーザー文言
    api: {
      network: 'サーバに接続できません。電波・LAN を確認してください',
      timeout: '応答がありません。少し待ってから再操作してください',
      auth: 'ログインが切れました。再度ログインしてください',
      forbidden: '権限がありません。班長に相談してください',
      not_found: '対象が見つかりません。表示を更新してください',
      conflict: '他の端末が先に更新しました。最新化してから再操作してください',
      rate_limited: '操作が混み合っています。少し時間を空けてから再試行してください',
      server: 'サーバ側で問題が発生しました。班長へ連絡してください',
      unknown: '不明なエラーが発生しました'
    },
    boundary_title: '画面に異常が発生しました',
    boundary_description: '安全のため処理を中断しました。再読込してください。',
    reload: '再読込',
    retry: '再試行',
    dismiss: '閉じる',
    show_detail: '詳細',
    hide_detail: '詳細を隠す'
  },
  // 共通の状態 UI 文言
  state_label: {
    loading_tasks: 'タスクを読込中…',
    loading_steps: 'ステップを読込中…',
    no_tasks_title: '今日のタスクはありません',
    no_tasks_description: '管理者が割り当てを行うと表示されます',
    no_steps_title: 'このタスクには手順がありません'
  },
  // 完了条件種別
  completion: {
    manual: '人手判定',
    photo: '写真証跡'
  },
  // §10.6 オフライン耐性: ネットワーク状態の現場可視化
  network: {
    online: 'オンライン',
    offline: 'オフライン',
    aria_label: '通信状態'
  },
  // §9.2.2 誤操作予防の確認文言
  confirm: {
    andon_title: 'アンドンを発火しますか？',
    andon_description: '班長へ即時に応援要請が送信されます。発火後は取り消しできません。',
    andon_confirm: '発火する',
    cancel: '取消'
  }
} as const;
