import type React from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { AuthGuard } from '@/auth/AuthGuard';
import { RoleGuard } from '@/auth/RoleGuard';
import { MainLayout } from '@/components/MainLayout';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import { LoginPage } from '@/pages/LoginPage';
import { ForbiddenPage } from '@/pages/ForbiddenPage';
import { NotFoundPage } from '@/pages/NotFoundPage';

// マスタメンテ画面
import { ProcessListPage } from '@/pages/master/ProcessListPage';
import { OperationListPage } from '@/pages/master/OperationListPage';
import { ProductListPage } from '@/pages/master/ProductListPage';
import { SopListPage } from '@/pages/master/SopListPage';
import { SopEditPage } from '@/pages/master/SopEditPage';
import { SopImportPage } from '@/pages/master/SopImportPage';
import { SopPreviewPage } from '@/pages/master/SopPreviewPage';
import { ReviewRequestPage } from '@/pages/master/ReviewRequestPage';
import { ApprovalSignPage } from '@/pages/master/ApprovalSignPage';
import { PublishSettingPage } from '@/pages/master/PublishSettingPage';
import { VersionDiffPage } from '@/pages/master/VersionDiffPage';
import { DeprecatePage } from '@/pages/master/DeprecatePage';
import { MaterialMasterPage } from '@/pages/master/MaterialMasterPage';
import { SupplierMasterPage } from '@/pages/master/SupplierMasterPage';
import { SamplingPlanPage } from '@/pages/master/SamplingPlanPage';
import { ReworkSopPage } from '@/pages/master/ReworkSopPage';
import { ReworkSopMappingPage } from '@/pages/master/ReworkSopMappingPage';
import { ReportTemplatePage } from '@/pages/master/ReportTemplatePage';

// 管理コンソール画面
import { OperationDashboardPage } from '@/pages/console/OperationDashboardPage';
import { UserManagementPage } from '@/pages/console/UserManagementPage';
import { RoleSkillPage } from '@/pages/console/RoleSkillPage';
import { AuditLogPage } from '@/pages/console/AuditLogPage';
import { XesExportPage } from '@/pages/console/XesExportPage';
import { BackupStatusPage } from '@/pages/console/BackupStatusPage';
import { OutboxMonitorPage } from '@/pages/console/OutboxMonitorPage';
import { HashChainVerifierPage } from '@/pages/console/HashChainVerifierPage';
import { ReportGeneratorPage } from '@/pages/console/ReportGeneratorPage';
import { ConcessionApprovalPage } from '@/pages/console/ConcessionApprovalPage';
import { IqcDashboardPage } from '@/pages/console/IqcDashboardPage';
import { IqcLotListPage } from '@/pages/console/IqcLotListPage';
import { DispositionApprovalPage } from '@/pages/console/DispositionApprovalPage';
import { ReworkListPage } from '@/pages/console/ReworkListPage';
import { ReworkTraceabilityPage } from '@/pages/console/ReworkTraceabilityPage';

// RBAC: アクセスロールは画面層と API 層の二段で検証する（src/frontend/master/CLAUDE.md §認証・認可）。
export function App(): React.ReactElement {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route path="/403" element={<ForbiddenPage />} />
      <Route
        element={
          <AuthGuard>
            <ErrorBoundary>
              <MainLayout />
            </ErrorBoundary>
          </AuthGuard>
        }
      >
        <Route index element={<Navigate to="/console/dashboard" replace />} />

        {/* SCR-MA-001〜017 */}
        <Route path="/master/processes" element={<RoleGuard roles={['master_admin']}><ProcessListPage /></RoleGuard>} />
        <Route path="/master/operations" element={<RoleGuard roles={['master_admin']}><OperationListPage /></RoleGuard>} />
        <Route path="/master/products" element={<RoleGuard roles={['master_admin']}><ProductListPage /></RoleGuard>} />
        <Route path="/master/sops" element={<RoleGuard roles={['master_admin', 'quality_admin']}><SopListPage /></RoleGuard>} />
        <Route path="/master/sops/new" element={<RoleGuard roles={['master_admin']}><SopEditPage /></RoleGuard>} />
        <Route path="/master/sops/:id/edit" element={<RoleGuard roles={['master_admin']}><SopEditPage /></RoleGuard>} />
        <Route path="/master/sops/import" element={<RoleGuard roles={['master_admin']}><SopImportPage /></RoleGuard>} />
        <Route path="/master/sops/:id/preview" element={<RoleGuard roles={['master_admin', 'quality_admin']}><SopPreviewPage /></RoleGuard>} />
        <Route path="/master/sops/:id/review" element={<RoleGuard roles={['master_admin']}><ReviewRequestPage /></RoleGuard>} />
        <Route path="/master/sops/:id/approve" element={<RoleGuard roles={['quality_admin']}><ApprovalSignPage /></RoleGuard>} />
        <Route path="/master/sops/:id/publish" element={<RoleGuard roles={['quality_admin']}><PublishSettingPage /></RoleGuard>} />
        <Route path="/master/sops/:id/versions" element={<RoleGuard roles={['master_admin', 'quality_admin']}><VersionDiffPage /></RoleGuard>} />
        <Route path="/master/sops/:id/deprecate" element={<RoleGuard roles={['master_admin']}><DeprecatePage /></RoleGuard>} />
        <Route path="/master/materials" element={<RoleGuard roles={['master_admin']}><MaterialMasterPage /></RoleGuard>} />
        <Route path="/master/suppliers" element={<RoleGuard roles={['master_admin']}><SupplierMasterPage /></RoleGuard>} />
        <Route path="/master/sampling-plans" element={<RoleGuard roles={['quality_admin']}><SamplingPlanPage /></RoleGuard>} />
        <Route path="/master/rework-sops" element={<RoleGuard roles={['master_admin']}><ReworkSopPage /></RoleGuard>} />
        <Route path="/master/rework-sop-mappings" element={<RoleGuard roles={['master_admin', 'quality_admin']}><ReworkSopMappingPage /></RoleGuard>} />
        <Route path="/master/report-templates" element={<RoleGuard roles={['system_admin']}><ReportTemplatePage /></RoleGuard>} />

        {/* SCR-MC-001〜015 */}
        <Route path="/console/dashboard" element={<RoleGuard roles={['system_admin', 'executive']}><OperationDashboardPage /></RoleGuard>} />
        <Route path="/console/users" element={<RoleGuard roles={['system_admin']}><UserManagementPage /></RoleGuard>} />
        <Route path="/console/roles" element={<RoleGuard roles={['system_admin']}><RoleSkillPage /></RoleGuard>} />
        <Route path="/console/audit-logs" element={<RoleGuard roles={['quality_admin', 'system_admin']}><AuditLogPage /></RoleGuard>} />
        <Route path="/console/xes-export" element={<RoleGuard roles={['quality_admin', 'system_admin']}><XesExportPage /></RoleGuard>} />
        <Route path="/console/backup" element={<RoleGuard roles={['system_admin']}><BackupStatusPage /></RoleGuard>} />
        <Route path="/console/outbox" element={<RoleGuard roles={['system_admin']}><OutboxMonitorPage /></RoleGuard>} />
        <Route path="/console/hash-chain" element={<RoleGuard roles={['quality_admin', 'system_admin']}><HashChainVerifierPage /></RoleGuard>} />
        <Route path="/console/reports" element={<RoleGuard roles={['quality_admin', 'system_admin']}><ReportGeneratorPage /></RoleGuard>} />
        <Route path="/console/concession-approvals" element={<RoleGuard roles={['quality_admin']}><ConcessionApprovalPage /></RoleGuard>} />
        <Route path="/console/iqc-dashboard" element={<RoleGuard roles={['quality_admin', 'executive']}><IqcDashboardPage /></RoleGuard>} />
        <Route path="/console/iqc-lots" element={<RoleGuard roles={['quality_admin', 'system_admin']}><IqcLotListPage /></RoleGuard>} />
        <Route path="/console/dispositions" element={<RoleGuard roles={['quality_admin', 'supervisor']}><DispositionApprovalPage /></RoleGuard>} />
        <Route path="/console/rework-list" element={<RoleGuard roles={['quality_admin', 'supervisor']}><ReworkListPage /></RoleGuard>} />
        <Route path="/console/rework-traceability" element={<RoleGuard roles={['quality_admin', 'system_admin']}><ReworkTraceabilityPage /></RoleGuard>} />

        <Route path="*" element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
}
