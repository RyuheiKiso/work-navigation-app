import { authHandlers } from './auth';
import { workExecutionHandlers } from './work-executions';
import { evidenceHandlers } from './evidences';
import { masterHandlers } from './master';
import { sopWorkflowHandlers } from './sop-workflow';
import { userHandlers } from './users';
import { alertHandlers } from './alerts';
import { workAssignmentHandlers } from './work-assignments';
import { iqcHandlers } from './iqc';
import { reworkHandlers } from './reworks';
import { auditHandlers } from './audit';

export const handlers = [
  ...authHandlers,
  ...workExecutionHandlers,
  ...evidenceHandlers,
  ...masterHandlers,
  ...sopWorkflowHandlers,
  ...userHandlers,
  ...alertHandlers,
  ...workAssignmentHandlers,
  ...iqcHandlers,
  ...reworkHandlers,
  ...auditHandlers,
];
