// สอดคล้องกับ §: โรดแมป §11.3.1 (การขยายภาษาไทย) §28.
// พจนานุกรมภาษาไทย。ASEAN 製造業集積。

/** พจนานุกรมภาษาไทย */
export const th = {
  term: {
    task: 'งาน',
    process: 'กระบวนการ',
    procedure: 'ขั้นตอน',
    step: 'การกระทำ',
    completion_criteria: 'เกณฑ์การเสร็จสมบูรณ์',
    precondition: 'เงื่อนไขเริ่มต้น',
    record: 'บันทึก',
    andon: 'แอนดอน',
    audit_log: 'บันทึกการตรวจสอบ',
    flow: 'โฟลว์',
    master: 'ข้อมูลหลัก',
    addon: 'ส่วนเสริม'
  },
  action: {
    start: 'เริ่ม',
    suspend: 'หยุดชั่วคราว',
    resume: 'ดำเนินการต่อ',
    complete: 'เสร็จสมบูรณ์',
    abort: 'ยกเลิก'
  },
  task: {
    state_label: 'สถานะ',
    progress_label: 'ความคืบหน้า (Lamport)',
    completion_criteria_label: 'เกณฑ์การเสร็จสมบูรณ์',
    start_button: 'เริ่ม',
    complete_button: 'เสร็จสมบูรณ์',
    completed_label: 'เสร็จสมบูรณ์แล้ว',
    suspended_label: 'หยุดชั่วคราว',
    aria_complete: 'ทำเครื่องหมายงานเสร็จสมบูรณ์'
  },
  state: {
    Idle: 'ว่าง',
    Ready: 'พร้อม',
    Running: 'กำลังทำงาน',
    Suspended: 'หยุดชั่วคราว',
    Exception: 'ข้อยกเว้น',
    Completed: 'เสร็จสมบูรณ์',
    Failed: 'ล้มเหลว',
    Aborted: 'ยกเลิกแล้ว'
  },
  error: {
    precondition_not_satisfied: 'เงื่อนไขเริ่มต้นยังไม่ตรงตามข้อกำหนด',
    completion_criteria_not_met: 'เกณฑ์การเสร็จสมบูรณ์ยังไม่ตรงตามข้อกำหนด',
    invalid_state: 'การเปลี่ยนสถานะไม่ถูกต้อง',
    network_offline: 'ไม่สามารถเข้าถึงเครือข่ายได้',
    sync_pending: 'รอการซิงค์ (จะซิงค์อัตโนมัติเมื่อกู้คืน)'
  },
  completion: {
    manual: 'ตัดสินด้วยตนเอง',
    photo: 'หลักฐานภาพถ่าย'
  }
} as const;
