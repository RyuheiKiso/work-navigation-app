// סעיף מפת הדרכים §: §11.3.1 (הרחבה לעברית, RTL) §28.
// מילון עברי. RTL レイアウト検証対象（§11.3.1）。

/** מילון עברי */
export const he = {
  term: {
    task: 'משימה',
    process: 'תהליך',
    procedure: 'הליך',
    step: 'צעד',
    completion_criteria: 'קריטריון השלמה',
    precondition: 'תנאי מקדים',
    record: 'רשומה',
    andon: 'אנדון',
    audit_log: 'יומן ביקורת',
    flow: 'זרימה',
    master: 'נתוני אב',
    addon: 'תוסף'
  },
  action: {
    start: 'התחל',
    suspend: 'השעה',
    resume: 'המשך',
    complete: 'השלם',
    abort: 'בטל'
  },
  task: {
    state_label: 'מצב',
    progress_label: 'התקדמות (Lamport)',
    completion_criteria_label: 'קריטריון השלמה',
    start_button: 'התחל',
    complete_button: 'השלם',
    completed_label: 'הושלם',
    suspended_label: 'מושהה',
    aria_complete: 'סמן משימה כמושלמת'
  },
  state: {
    Idle: 'במנוחה',
    Ready: 'מוכן',
    Running: 'פועל',
    Suspended: 'מושהה',
    Exception: 'חריג',
    Completed: 'הושלם',
    Failed: 'נכשל',
    Aborted: 'בוטל'
  },
  error: {
    precondition_not_satisfied: 'התנאי המקדים לא התקיים',
    completion_criteria_not_met: 'קריטריון ההשלמה לא התקיים',
    invalid_state: 'מעבר מצב לא חוקי',
    network_offline: 'הרשת אינה זמינה',
    sync_pending: 'הסנכרון ממתין (יחודש אוטומטית בעת השחזור)'
  },
  completion: {
    manual: 'ידני',
    photo: 'הוכחת תמונה'
  }
} as const;
