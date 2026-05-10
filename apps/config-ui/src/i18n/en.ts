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
  },
  error: {
    api: {
      network: 'Cannot reach the server. Check your network.',
      timeout: 'No response. Wait a moment and try again.',
      auth: 'Your session has expired. Please log in again.',
      forbidden: 'You do not have permission. Please contact an admin.',
      not_found: 'The target was not found. Refresh the list.',
      conflict: 'Another user updated this first. Refresh and try again.',
      rate_limited: 'Operations are throttled. Please wait briefly and retry.',
      server: 'A server error occurred. Please contact an admin.',
      unknown: 'An unknown error occurred.'
    },
    boundary_title: 'A display error occurred',
    boundary_description: 'The process was halted as a precaution. Please reload.',
    reload: 'Reload',
    retry: 'Retry',
    dismiss: 'Dismiss',
    show_detail: 'Details',
    hide_detail: 'Hide details'
  },
  state_label: {
    loading_master: 'Loading master data…',
    loading_audit: 'Loading audit log…',
    loading_dashboard: 'Loading dashboard…',
    no_master_title: 'No records yet',
    no_master_description: 'Create one with the form above.',
    no_audit_title: 'No matching audit entries',
    no_dashboard_title: 'No tasks in progress'
  }
} as const;
