// 대응 §: 로드맵 §11.3.1（한국어 확장）§28
// 한국어 사전. 「작업」은 일본어와 의미 범위가 가까워 직역으로 통한다（§11.3.1）.

/** 한국어 사전 */
export const ko = {
  term: {
    task: '작업',
    process: '공정',
    procedure: '절차',
    step: '동작',
    completion_criteria: '완료 조건',
    precondition: '시작 조건',
    record: '실적',
    andon: '안돈',
    audit_log: '감사 로그',
    flow: '플로우',
    master: '마스터',
    addon: '애드온'
  },
  action: {
    start: '시작',
    suspend: '중단',
    resume: '재개',
    complete: '완료',
    abort: '취소'
  },
  task: {
    state_label: '상태',
    progress_label: '진척（Lamport）',
    completion_criteria_label: '완료 조건',
    start_button: '시작',
    complete_button: '완료',
    completed_label: '완료됨',
    suspended_label: '중단됨',
    aria_complete: '작업을 완료로 표시'
  },
  state: {
    Idle: '초기',
    Ready: '준비됨',
    Running: '실행 중',
    Suspended: '중단',
    Exception: '예외',
    Completed: '완료',
    Failed: '실패',
    Aborted: '취소'
  },
  error: {
    precondition_not_satisfied: '시작 조건이 충족되지 않았습니다',
    completion_criteria_not_met: '완료 조건이 충족되지 않았습니다',
    invalid_state: '잘못된 상태 전환입니다',
    network_offline: '네트워크 연결이 불가합니다',
    sync_pending: '동기화 보류 중（복구 후 자동 동기화됩니다）'
  },
  completion: {
    manual: '수동 판정',
    photo: '사진 증거'
  }
} as const;
