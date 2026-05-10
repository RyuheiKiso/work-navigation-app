// Sesuai §: peta jalan §11.3.1 (perluasan bahasa Indonesia) §28.
// Kamus bahasa Indonesia. ASEAN 製造業集積。

/** Kamus bahasa Indonesia */
export const id = {
  term: {
    task: 'Tugas',
    process: 'Proses',
    procedure: 'Prosedur',
    step: 'Langkah',
    completion_criteria: 'Kriteria penyelesaian',
    precondition: 'Prasyarat',
    record: 'Catatan',
    andon: 'Andon',
    audit_log: 'Log audit',
    flow: 'Alur',
    master: 'Data master',
    addon: 'Pengaya'
  },
  action: {
    start: 'Mulai',
    suspend: 'Tunda',
    resume: 'Lanjutkan',
    complete: 'Selesai',
    abort: 'Batalkan'
  },
  task: {
    state_label: 'Status',
    progress_label: 'Kemajuan (Lamport)',
    completion_criteria_label: 'Kriteria penyelesaian',
    start_button: 'Mulai',
    complete_button: 'Selesai',
    completed_label: 'Selesai',
    suspended_label: 'Tertunda',
    aria_complete: 'Tandai tugas selesai'
  },
  state: {
    Idle: 'Diam',
    Ready: 'Siap',
    Running: 'Berjalan',
    Suspended: 'Tertunda',
    Exception: 'Pengecualian',
    Completed: 'Selesai',
    Failed: 'Gagal',
    Aborted: 'Dibatalkan'
  },
  error: {
    precondition_not_satisfied: 'Prasyarat belum terpenuhi',
    completion_criteria_not_met: 'Kriteria penyelesaian belum terpenuhi',
    invalid_state: 'Transisi status tidak valid',
    network_offline: 'Jaringan tidak dapat dijangkau',
    sync_pending: 'Sinkronisasi tertunda (akan otomatis saat pulih)'
  },
  completion: {
    manual: 'Manual',
    photo: 'Bukti foto'
  }
} as const;
