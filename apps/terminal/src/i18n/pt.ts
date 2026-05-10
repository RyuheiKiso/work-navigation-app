// Seção do roadmap §: §11.3.1 (expansão para português) §28.
// Dicionário em português (PT-BR/PT-PT comum).

/** Dicionário em português */
export const pt = {
  term: {
    task: 'Tarefa',
    process: 'Processo',
    procedure: 'Procedimento',
    step: 'Passo',
    completion_criteria: 'Critério de conclusão',
    precondition: 'Pré-condição',
    record: 'Registro',
    andon: 'Andon',
    audit_log: 'Log de auditoria',
    flow: 'Fluxo',
    master: 'Dados mestre',
    addon: 'Complemento'
  },
  action: {
    start: 'Iniciar',
    suspend: 'Suspender',
    resume: 'Retomar',
    complete: 'Concluir',
    abort: 'Cancelar'
  },
  task: {
    state_label: 'Estado',
    progress_label: 'Progresso (Lamport)',
    completion_criteria_label: 'Critério de conclusão',
    start_button: 'Iniciar',
    complete_button: 'Concluir',
    completed_label: 'Concluído',
    suspended_label: 'Suspenso',
    aria_complete: 'Marcar tarefa como concluída'
  },
  state: {
    Idle: 'Ocioso',
    Ready: 'Pronto',
    Running: 'Em execução',
    Suspended: 'Suspenso',
    Exception: 'Exceção',
    Completed: 'Concluído',
    Failed: 'Falhou',
    Aborted: 'Cancelado'
  },
  error: {
    precondition_not_satisfied: 'Pré-condição não atendida',
    completion_criteria_not_met: 'Critério de conclusão não atendido',
    invalid_state: 'Transição de estado inválida',
    network_offline: 'Rede inacessível',
    sync_pending: 'Sincronização pendente (será retomada automaticamente)'
  },
  completion: {
    manual: 'Manual',
    photo: 'Evidência fotográfica'
  }
} as const;
