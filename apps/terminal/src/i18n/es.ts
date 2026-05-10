// Sección hoja de ruta: §11.3.1 (expansión al español) §28.
// Diccionario en español. Cobertura para América Latina y España.

/** Diccionario en español */
export const es = {
  term: {
    task: 'Tarea',
    process: 'Proceso',
    procedure: 'Procedimiento',
    step: 'Paso',
    completion_criteria: 'Criterio de finalización',
    precondition: 'Condición previa',
    record: 'Registro',
    andon: 'Andon',
    audit_log: 'Registro de auditoría',
    flow: 'Flujo',
    master: 'Datos maestros',
    addon: 'Complemento'
  },
  action: {
    start: 'Iniciar',
    suspend: 'Suspender',
    resume: 'Reanudar',
    complete: 'Completar',
    abort: 'Cancelar'
  },
  task: {
    state_label: 'Estado',
    progress_label: 'Progreso (Lamport)',
    completion_criteria_label: 'Criterio de finalización',
    start_button: 'Iniciar',
    complete_button: 'Completar',
    completed_label: 'Completado',
    suspended_label: 'Suspendido',
    aria_complete: 'Marcar tarea como completada'
  },
  state: {
    Idle: 'Inactivo',
    Ready: 'Listo',
    Running: 'En ejecución',
    Suspended: 'Suspendido',
    Exception: 'Excepción',
    Completed: 'Completado',
    Failed: 'Fallido',
    Aborted: 'Cancelado'
  },
  error: {
    precondition_not_satisfied: 'No se cumple la condición previa',
    completion_criteria_not_met: 'No se cumple el criterio de finalización',
    invalid_state: 'Transición de estado no válida',
    network_offline: 'Red inalcanzable',
    sync_pending: 'Sincronización pendiente (se reanudará automáticamente al recuperarse)'
  },
  completion: {
    manual: 'Manual',
    photo: 'Evidencia fotográfica'
  }
} as const;
