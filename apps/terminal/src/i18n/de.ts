// Roadmap section: §11.3.1 (German expansion) §28.
// Deutsche Sprachdatei. "Arbeitsschritt" entspricht Step, "Vorgang" entspricht Procedure
// (siehe §3.1.5.2 multilingual comparison table).

/** Deutsches Wörterbuch */
export const de = {
  term: {
    task: 'Aufgabe',
    process: 'Prozess',
    procedure: 'Vorgang',
    step: 'Arbeitsschritt',
    completion_criteria: 'Abschlusskriterium',
    precondition: 'Vorbedingung',
    record: 'Aufzeichnung',
    andon: 'Andon',
    audit_log: 'Audit-Protokoll',
    flow: 'Flow',
    master: 'Stammdaten',
    addon: 'Add-on'
  },
  action: {
    start: 'Starten',
    suspend: 'Unterbrechen',
    resume: 'Fortsetzen',
    complete: 'Abschließen',
    abort: 'Abbrechen'
  },
  task: {
    state_label: 'Zustand',
    progress_label: 'Fortschritt (Lamport)',
    completion_criteria_label: 'Abschlusskriterium',
    start_button: 'Starten',
    complete_button: 'Abschließen',
    completed_label: 'Abgeschlossen',
    suspended_label: 'Unterbrochen',
    aria_complete: 'Aufgabe als abgeschlossen markieren'
  },
  state: {
    Idle: 'Leerlauf',
    Ready: 'Bereit',
    Running: 'Läuft',
    Suspended: 'Unterbrochen',
    Exception: 'Ausnahme',
    Completed: 'Abgeschlossen',
    Failed: 'Fehlgeschlagen',
    Aborted: 'Abgebrochen'
  },
  error: {
    precondition_not_satisfied: 'Vorbedingung nicht erfüllt',
    completion_criteria_not_met: 'Abschlusskriterium nicht erfüllt',
    invalid_state: 'Ungültiger Zustandsübergang',
    network_offline: 'Netzwerk nicht erreichbar',
    sync_pending: 'Synchronisierung ausstehend (wird nach Wiederherstellung automatisch ausgeführt)'
  },
  completion: {
    manual: 'Manuell',
    photo: 'Foto-Beweis'
  }
} as const;
