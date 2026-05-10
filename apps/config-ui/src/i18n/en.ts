// Corresponding sections: roadmap §11.3 §28 §10.2.
// English locale dictionary for the config UI.

/** Config UI English dictionary */
export const en = {
  term: {
    task: 'Task',
    process: 'Process',
    procedure: 'Procedure',
    flow: 'Flow',
    master: 'Master Data',
    addon: 'Addon'
  },
  flow: {
    title_prefix: 'Flow: ',
    version_label: 'Version',
    nodes_label: 'Nodes',
    edges_label: 'Edges',
    nodes_section: 'Node list',
    publish_trial_button: 'Publish trial',
    aria_publish_trial: 'Publish trial version',
    dirty_indicator: 'Unsaved changes'
  },
  setting_ui: {
    autosave_label: 'Autosave',
    autosave_just_now: 'Saved just now',
    autosave_seconds_ago: 'Saved {n} seconds ago',
    autosave_minutes_ago: 'Saved {n} minutes ago',
    autosave_saving: 'Saving…',
    autosave_failed: 'Autosave failed',
    autosave_idle: 'Autosaves on edit',
    draft_restored: 'Restored previous edits',
    discard_draft: 'Discard restored draft',
    rollback_link: 'Revert to previous version',
    save_draft: 'Save as draft',
    impact_count_prefix: 'Impact: '
  },
  completion: {
    manual: 'Manual',
    photo: 'Photo evidence'
  },
  network: {
    online: 'Online',
    offline: 'Offline',
    aria_label: 'Network status'
  },
  confirm: {
    delete_title: 'Delete record',
    delete_description_prefix: 'Code ',
    delete_description_suffix: ' will be deleted. This cannot be undone.',
    delete_confirm: 'Delete',
    cancel: 'Cancel'
  }
} as const;
