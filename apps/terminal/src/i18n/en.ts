// Corresponding sections: roadmap §11.3 §28 (English glossary alignment).
// English locale dictionary. Mirrors the Japanese dictionary structure exactly.

/** English dictionary */
export const en = {
  term: {
    task: 'Task',
    process: 'Process',
    procedure: 'Procedure',
    step: 'Step',
    completion_criteria: 'Completion Criteria',
    precondition: 'Precondition',
    record: 'Record',
    andon: 'Andon',
    audit_log: 'Audit Log',
    flow: 'Flow',
    master: 'Master Data',
    addon: 'Addon'
  },
  action: {
    start: 'Start',
    suspend: 'Suspend',
    resume: 'Resume',
    complete: 'Complete',
    abort: 'Abort'
  },
  task: {
    state_label: 'State',
    progress_label: 'Progress (Lamport)',
    completion_criteria_label: 'Completion Criteria',
    start_button: 'Start',
    complete_button: 'Complete',
    completed_label: 'Completed',
    suspended_label: 'Suspended',
    aria_complete: 'Mark task complete'
  },
  state: {
    Idle: 'Idle',
    Ready: 'Ready',
    Running: 'Running',
    Suspended: 'Suspended',
    Exception: 'Exception',
    Completed: 'Completed',
    Failed: 'Failed',
    Aborted: 'Aborted'
  },
  error: {
    precondition_not_satisfied: 'Precondition not satisfied',
    completion_criteria_not_met: 'Completion criteria not met',
    invalid_state: 'Invalid state transition',
    network_offline: 'Network unreachable',
    sync_pending: 'Sync pending (will auto-sync upon recovery)'
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
    andon_title: 'Trigger andon?',
    andon_description: 'Help will be requested from the lead immediately. This cannot be undone.',
    andon_confirm: 'Trigger',
    cancel: 'Cancel'
  }
} as const;
