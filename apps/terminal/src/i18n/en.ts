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
    sync_pending: 'Sync pending (will auto-sync upon recovery)',
    api: {
      network: 'Cannot reach the server. Check your signal or wired network.',
      timeout: 'No response. Wait a moment and try again.',
      auth: 'Your session has expired. Please log in again.',
      forbidden: 'You do not have permission. Please contact your lead.',
      not_found: 'The target was not found. Refresh the view.',
      conflict: 'Another device updated this first. Refresh and try again.',
      rate_limited: 'Operations are throttled. Please wait briefly and retry.',
      server: 'A server error occurred. Please contact your lead.',
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
    loading_tasks: 'Loading tasks…',
    loading_steps: 'Loading steps…',
    no_tasks_title: 'No tasks for today',
    no_tasks_description: 'Tasks will appear here once an administrator assigns them.',
    no_steps_title: 'This task has no steps'
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
