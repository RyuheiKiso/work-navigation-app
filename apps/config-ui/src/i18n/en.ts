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
    autosave_indicator: 'Autosaving every 10 seconds',
    rollback_link: 'Revert to previous version',
    save_draft: 'Save as draft',
    impact_count_prefix: 'Impact: '
  },
  completion: {
    manual: 'Manual',
    photo: 'Photo evidence'
  }
} as const;
