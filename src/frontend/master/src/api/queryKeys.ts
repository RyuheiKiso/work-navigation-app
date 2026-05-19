// TanStack Query のキャッシュキー命名規約を集約（散逸防止）
// 階層: [domain, resource, ...filters] のタプル形式で構成する

export const queryKeys = {
  auth: {
    me: () => ['auth', 'me'] as const,
  },
  master: {
    processes: (asOf?: string | null) => ['master', 'processes', { asOf: asOf ?? null }] as const,
    process: (id: string) => ['master', 'process', id] as const,
    operations: (asOf?: string | null) => ['master', 'operations', { asOf: asOf ?? null }] as const,
    products: (asOf?: string | null) => ['master', 'products', { asOf: asOf ?? null }] as const,
    sops: (filters?: Record<string, unknown>) => ['master', 'sops', filters ?? {}] as const,
    sop: (id: string) => ['master', 'sop', id] as const,
    sopVersions: (id: string) => ['master', 'sop', id, 'versions'] as const,
    materials: (asOf?: string | null) => ['master', 'materials', { asOf: asOf ?? null }] as const,
    suppliers: (asOf?: string | null) => ['master', 'suppliers', { asOf: asOf ?? null }] as const,
    samplingPlans: () => ['master', 'sampling-plans'] as const,
    reworkSops: () => ['master', 'rework-sops'] as const,
    reworkSopMappings: () => ['master', 'rework-sop-mappings'] as const,
    reportTemplates: () => ['master', 'report-templates'] as const,
  },
  console: {
    dashboard: (range: { from: string; to: string }) => ['console', 'dashboard', range] as const,
    users: () => ['console', 'users'] as const,
    roles: () => ['console', 'roles'] as const,
    auditLogs: (filters: Record<string, unknown>) => ['console', 'audit-logs', filters] as const,
    backupStatus: () => ['console', 'backup-status'] as const,
    outbox: (filters?: Record<string, unknown>) => ['console', 'outbox', filters ?? {}] as const,
    hashChain: () => ['console', 'hash-chain'] as const,
    concessions: (status?: string) => ['console', 'concessions', { status: status ?? null }] as const,
    iqcDashboard: (range: { from: string; to: string }) => ['console', 'iqc-dashboard', range] as const,
    iqcLots: (filters?: Record<string, unknown>) => ['console', 'iqc-lots', filters ?? {}] as const,
    dispositions: (status?: string) => ['console', 'dispositions', { status: status ?? null }] as const,
    rework: (filters?: Record<string, unknown>) => ['console', 'rework', filters ?? {}] as const,
    reworkTrace: (caseId: string) => ['console', 'rework-trace', caseId] as const,
  },
} as const;
