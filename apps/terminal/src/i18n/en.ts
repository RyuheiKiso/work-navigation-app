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
    aria_complete: 'Mark task complete',
    today_list_title: 'Today’s tasks',
    select_task_prompt: 'Please start a task',
    start_button_short: 'Start',
    suspend_button: 'Suspend',
    resume_button: 'Resume',
    andon_button: 'Andon',
    all_steps_done: 'All steps complete',
    step_map_title: 'Step map',
    storage_title: 'Storage',
    voice_section_title: 'Voice command',
    voice_input_placeholder: 'start / complete / suspend',
    voice_recognize_button: 'Recognize',
    voice_unrecognized: 'Voice command not recognized',
    no_andon: 'No issues',
    andon_severity_prefix: 'Andon Lv.',
    andon_default_message: 'Out of parts — help requested from lead',
    suspend_default_message: 'Suspended',
    next_action_prefix: 'Next action',
    overrun_label: 'Overrun',
    standard_time_prefix: 'Std. time',
    progress_aria_label: 'Overall task progress',
    peek_next_title: 'Next step',
    peek_next_end: 'No upcoming step'
  },
  media: {
    image: 'Photo',
    video: 'Video',
    diagram: 'Diagram'
  },
  shortcut: {
    complete: 'Complete current step',
    suspend_resume: 'Suspend / Resume',
    andon: 'Trigger andon',
    toggle_task_drawer: 'Toggle task list',
    toggle_step_map: 'Toggle step map',
    cycle_theme: 'Cycle theme (standard → outdoor → dark → auto)',
    show_help: 'Show shortcut help',
    close_dialogs: 'Close dialogs',
    help_title: 'Keyboard shortcuts',
    close_label: 'Close'
  },
  shell: {
    logout: 'Log out',
    state_prefix: 'State',
    task_id_prefix: 'Task',
    task_id_unselected: 'unselected',
    toggle_task_drawer: 'Toggle task list',
    toggle_step_map_drawer: 'Toggle step map',
    theme_label: 'Theme',
    shortcuts_label: 'Shortcuts'
  },
  login: {
    title: 'Log in',
    subtitle: 'work-navigation-app terminal',
    backend_url_label: 'Backend URL',
    user_id_label: 'User ID',
    password_label: 'Password',
    submit: 'Log in',
    submit_busy: 'Working…',
    demo_banner: 'Demo mode — do not use in production',
    demo_users: 'Demo: alice / bob / charlie (password: hello-world)'
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
