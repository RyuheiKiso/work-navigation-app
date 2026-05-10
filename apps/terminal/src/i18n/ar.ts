// قسم خارطة الطريق §: §11.3.1 (التوسع للغة العربية، RTL مسار) §28.
// قاموس اللغة العربية. RTL レイアウト時の検証対象（§11.3.1）。

/** قاموس عربي */
export const ar = {
  term: {
    task: 'مهمة',
    process: 'عملية',
    procedure: 'إجراء',
    step: 'خطوة',
    completion_criteria: 'معيار الإكمال',
    precondition: 'شرط مسبق',
    record: 'سجل',
    andon: 'أندون',
    audit_log: 'سجل التدقيق',
    flow: 'تدفق',
    master: 'البيانات الرئيسية',
    addon: 'إضافة'
  },
  action: {
    start: 'بدء',
    suspend: 'تعليق',
    resume: 'استئناف',
    complete: 'إكمال',
    abort: 'إلغاء'
  },
  task: {
    state_label: 'الحالة',
    progress_label: 'التقدم (Lamport)',
    completion_criteria_label: 'معيار الإكمال',
    start_button: 'بدء',
    complete_button: 'إكمال',
    completed_label: 'مكتمل',
    suspended_label: 'معلق',
    aria_complete: 'وضع علامة على المهمة كمكتملة'
  },
  state: {
    Idle: 'خامل',
    Ready: 'جاهز',
    Running: 'قيد التشغيل',
    Suspended: 'معلق',
    Exception: 'استثناء',
    Completed: 'مكتمل',
    Failed: 'فشل',
    Aborted: 'ملغى'
  },
  error: {
    precondition_not_satisfied: 'لم يتم استيفاء الشرط المسبق',
    completion_criteria_not_met: 'لم يتم استيفاء معيار الإكمال',
    invalid_state: 'انتقال حالة غير صالح',
    network_offline: 'الشبكة غير متاحة',
    sync_pending: 'المزامنة معلقة (ستستأنف تلقائياً عند الاستعادة)'
  },
  completion: {
    manual: 'يدوي',
    photo: 'دليل بالصورة'
  }
} as const;
