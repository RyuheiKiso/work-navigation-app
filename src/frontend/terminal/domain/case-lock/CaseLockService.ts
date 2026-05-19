// case_locks サービス。60 秒 heartbeat、5 分 EXPIRED 監視、ERR-BIZ-026 409 ハンドリング
import { CaseLockRepository } from '../../db/repositories/CaseLockRepository';
import type { JwtService } from '../../auth/JwtService';

const HEARTBEAT_INTERVAL_MS = 60 * 1000;
const EMERGENCY_THRESHOLD_MS = 5 * 60 * 1000;

export type LockOutcome = 'acquired' | 'conflict' | 'network_error';

export interface CaseLockServiceDeps {
  baseApiUrl: string;
  jwtService: JwtService;
}

export class CaseLockService {
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  private currentCaseId: string | null = null;
  private readonly repo: CaseLockRepository;

  constructor(private readonly deps: CaseLockServiceDeps) {
    this.repo = new CaseLockRepository();
  }

  // 占有獲得は POST /work-executions または POST /resume 経由。409 ERR-BIZ-026 で中断する
  async acquire(params: {
    caseId: string;
    terminalId: string;
    userId: string;
    resume: boolean;
  }): Promise<LockOutcome> {
    try {
      const url = params.resume
        ? `${this.deps.baseApiUrl}/work-executions/${encodeURIComponent(params.caseId)}/resume`
        : `${this.deps.baseApiUrl}/work-executions`;
      const token = await this.deps.jwtService.getAccessToken();
      const res = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({ caseId: params.caseId, terminalId: params.terminalId, userId: params.userId }),
      });
      if (res.status === 409) return 'conflict';
      if (!res.ok) return 'network_error';

      const now = new Date().toISOString();
      await this.repo.upsert({
        caseId: params.caseId,
        terminalId: params.terminalId,
        userId: params.userId,
        acquiredAt: now,
        heartbeatAt: now,
        lockStatus: 'ACTIVE',
      });
      this.startHeartbeat(params.caseId);
      return 'acquired';
    } catch {
      return 'network_error';
    }
  }

  // 60 秒間隔で heartbeat を送信。失敗時はローカル時刻のみ更新し再接続時に同期する
  startHeartbeat(caseId: string): void {
    this.stopHeartbeat();
    this.currentCaseId = caseId;
    this.heartbeatTimer = setInterval(() => {
      void this.heartbeat(caseId);
    }, HEARTBEAT_INTERVAL_MS);
  }

  stopHeartbeat(): void {
    if (this.heartbeatTimer !== null) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
    this.currentCaseId = null;
  }

  private async heartbeat(caseId: string): Promise<void> {
    const now = new Date().toISOString();
    await this.repo.heartbeat(caseId, now);
    try {
      const token = await this.deps.jwtService.getAccessToken();
      await fetch(`${this.deps.baseApiUrl}/work-executions/${encodeURIComponent(caseId)}/heartbeat`, {
        method: 'PUT',
        headers: { Authorization: `Bearer ${token}` },
      });
    } catch {
      // 切断中は heartbeat 送信失敗を許容（ローカル時刻は更新済み）
    }
  }

  // 5 分以上経過した heartbeat は EXPIRED とみなす
  async checkExpired(caseId: string): Promise<boolean> {
    const lock = await this.repo.find(caseId);
    if (lock === null) return true;
    const last = Date.parse(lock.heartbeatAt);
    if (Number.isNaN(last)) return true;
    return Date.now() - last > EMERGENCY_THRESHOLD_MS;
  }

  async release(caseId: string): Promise<void> {
    await this.repo.release(caseId);
    if (this.currentCaseId === caseId) this.stopHeartbeat();
  }

  getActiveCaseId(): string | null {
    return this.currentCaseId;
  }
}
