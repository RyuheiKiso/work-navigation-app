// Đáp ứng §: lộ trình §11.3.1 (mở rộng tiếng Việt) §28.
// Từ điển tiếng Việt. ASEAN 製造業集積（日系企業の進出先）。

/** Từ điển tiếng Việt */
export const vi = {
  term: {
    // Công việc は仕事寄り、Thao tác は動作寄り（§3.1.5.2）。本辞書では Công việc を作業（Task）とする
    task: 'Công việc',
    process: 'Quy trình',
    procedure: 'Thủ tục',
    step: 'Thao tác',
    completion_criteria: 'Điều kiện hoàn thành',
    precondition: 'Điều kiện bắt đầu',
    record: 'Bản ghi',
    andon: 'Andon',
    audit_log: 'Nhật ký kiểm toán',
    flow: 'Luồng',
    master: 'Dữ liệu chính',
    addon: 'Tiện ích bổ sung'
  },
  action: {
    start: 'Bắt đầu',
    suspend: 'Tạm dừng',
    resume: 'Tiếp tục',
    complete: 'Hoàn thành',
    abort: 'Hủy'
  },
  task: {
    state_label: 'Trạng thái',
    progress_label: 'Tiến độ (Lamport)',
    completion_criteria_label: 'Điều kiện hoàn thành',
    start_button: 'Bắt đầu',
    complete_button: 'Hoàn thành',
    completed_label: 'Đã hoàn thành',
    suspended_label: 'Đã tạm dừng',
    aria_complete: 'Đánh dấu công việc đã hoàn thành'
  },
  state: {
    Idle: 'Nghỉ',
    Ready: 'Sẵn sàng',
    Running: 'Đang chạy',
    Suspended: 'Tạm dừng',
    Exception: 'Ngoại lệ',
    Completed: 'Hoàn thành',
    Failed: 'Thất bại',
    Aborted: 'Đã hủy'
  },
  error: {
    precondition_not_satisfied: 'Điều kiện bắt đầu chưa được đáp ứng',
    completion_criteria_not_met: 'Điều kiện hoàn thành chưa được đáp ứng',
    invalid_state: 'Chuyển trạng thái không hợp lệ',
    network_offline: 'Không thể truy cập mạng',
    sync_pending: 'Đang chờ đồng bộ (sẽ tự động đồng bộ sau khi khôi phục)'
  },
  completion: {
    manual: 'Xác nhận thủ công',
    photo: 'Bằng chứng ảnh'
  }
} as const;
