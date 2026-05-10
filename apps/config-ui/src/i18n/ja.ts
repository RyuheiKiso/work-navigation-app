// 対応 §: ロードマップ §11.3 §28 §10.2
// 設定 UI 用 日本語ロケール辞書。

/** 設定 UI 日本語辞書 */
export const ja = {
  // §28 用語との一致
  term: {
    task: '作業',
    process: '工程',
    procedure: '手順',
    flow: 'フロー',
    master: 'マスタ',
    addon: 'アドオン'
  },
  // フローエディタ画面
  flow: {
    title_prefix: 'フロー: ',
    version_label: 'バージョン',
    nodes_label: 'ノード',
    edges_label: '辺',
    nodes_section: 'ノード一覧',
    publish_trial_button: '試行版を発行する',
    aria_publish_trial: '試行版を発行する',
    dirty_indicator: '未保存の変更があります'
  },
  // §10.2.2 14 観点に対応する文言
  setting_ui: {
    autosave_label: '自動保存',
    autosave_just_now: 'たった今保存しました',
    autosave_seconds_ago: '{n} 秒前に保存しました',
    autosave_minutes_ago: '{n} 分前に保存しました',
    autosave_saving: '保存中…',
    autosave_failed: '自動保存に失敗しました',
    autosave_idle: '編集が反映されると自動保存します',
    draft_restored: '前回の編集内容を復元しました',
    discard_draft: '復元を破棄して初期状態に戻す',
    rollback_link: '前のバージョンに戻す',
    save_draft: '下書きに残す',
    impact_count_prefix: '影響件数: '
  },
  // ステップ完了条件
  completion: {
    manual: '人手判定',
    photo: '写真証跡'
  },
  // §10.6 オフライン耐性
  network: {
    online: 'オンライン',
    offline: 'オフライン',
    aria_label: '通信状態'
  }
} as const;
