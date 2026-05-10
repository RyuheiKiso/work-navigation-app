// 对应 §: 路线图 §11.3.1（中国语简体扩张）§28
// 简体中文 词典。`作业` 在中文中存在「学校宿题」歧义，UI 中 § 11.3.1 表载明的
// 「操作」「工序」を併用して誤読を避ける。

/** 简体中文词典 */
export const zh = {
  term: {
    // 「作业」と「操作」の併用で誤読回避（§11.3.1）
    task: '作业（操作）',
    process: '工序',
    procedure: '步骤',
    step: '动作',
    completion_criteria: '完成条件',
    precondition: '开始条件',
    record: '实绩',
    andon: '安灯',
    audit_log: '审计日志',
    flow: '流程',
    master: '主数据',
    addon: '插件'
  },
  action: {
    start: '开始',
    suspend: '中断',
    resume: '继续',
    complete: '完成',
    abort: '取消'
  },
  task: {
    state_label: '状态',
    progress_label: '进度（Lamport）',
    completion_criteria_label: '完成条件',
    start_button: '开始',
    complete_button: '完成',
    completed_label: '已完成',
    suspended_label: '已中断',
    aria_complete: '标记作业完成'
  },
  state: {
    Idle: '初始',
    Ready: '就绪',
    Running: '运行中',
    Suspended: '中断',
    Exception: '异常',
    Completed: '完成',
    Failed: '失败',
    Aborted: '取消'
  },
  error: {
    precondition_not_satisfied: '未满足开始条件',
    completion_criteria_not_met: '未满足完成条件',
    invalid_state: '无效的状态转换',
    network_offline: '网络不可达',
    sync_pending: '同步待处理（恢复后将自动同步）'
  },
  completion: {
    manual: '人工判定',
    photo: '照片证据'
  }
} as const;
