// Section feuille de route §: §11.3.1 (extension française) §28.
// Dictionnaire français.

/** Dictionnaire français */
export const fr = {
  term: {
    task: 'Tâche',
    process: 'Processus',
    procedure: 'Procédure',
    step: 'Étape',
    completion_criteria: "Critère d'achèvement",
    precondition: 'Précondition',
    record: 'Enregistrement',
    andon: 'Andon',
    audit_log: "Journal d'audit",
    flow: 'Flux',
    master: 'Données de référence',
    addon: 'Module complémentaire'
  },
  action: {
    start: 'Démarrer',
    suspend: 'Suspendre',
    resume: 'Reprendre',
    complete: 'Terminer',
    abort: 'Annuler'
  },
  task: {
    state_label: 'État',
    progress_label: 'Progression (Lamport)',
    completion_criteria_label: "Critère d'achèvement",
    start_button: 'Démarrer',
    complete_button: 'Terminer',
    completed_label: 'Terminé',
    suspended_label: 'Suspendu',
    aria_complete: 'Marquer la tâche comme terminée'
  },
  state: {
    Idle: 'Inactif',
    Ready: 'Prêt',
    Running: 'En cours',
    Suspended: 'Suspendu',
    Exception: 'Exception',
    Completed: 'Terminé',
    Failed: 'Échoué',
    Aborted: 'Annulé'
  },
  error: {
    precondition_not_satisfied: 'Précondition non remplie',
    completion_criteria_not_met: "Critère d'achèvement non atteint",
    invalid_state: "Transition d'état invalide",
    network_offline: 'Réseau inaccessible',
    sync_pending: 'Synchronisation en attente (reprise automatique au rétablissement)'
  },
  completion: {
    manual: 'Manuel',
    photo: 'Preuve photographique'
  }
} as const;
