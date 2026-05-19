// OutboxWorker。created_at 昇順で 1 件ずつ送信、二重起動禁止、指数バックオフ
import { OutboxRepository } from '../../db/repositories/OutboxRepository';
import { WorkEventRepository } from '../../db/repositories/WorkEventRepository';
import type { LocalOutboxEvent } from '../../db/entities/LocalOutboxEvent';
import type { JwtService } from '../../auth/JwtService';

const MAX_BACKOFF_MS = 5 * 60 * 1000;
const POLL_INTERVAL_MS = 5000;
const MAX_RETRIES = 10;

export interface OutboxWorkerDeps {
  baseApiUrl: string;
  jwtService: JwtService;
}

export class OutboxWorker {
  private isRunning = false;
  private readonly outboxRepo: OutboxRepository;
  private readonly workEventRepo: WorkEventRepository;

  constructor(private readonly deps: OutboxWorkerDeps) {
    this.outboxRepo = new OutboxRepository();
    this.workEventRepo = new WorkEventRepository();
  }

  // シングルトン起動。Outbox の順序保証のため二重起動を禁止する
  async start(): Promise<void> {
    if (this.isRunning) return;
    this.isRunning = true;
    await this.processLoop();
  }

  stop(): void {
    this.isRunning = false;
  }

  isActive(): boolean {
    return this.isRunning;
  }

  private async processLoop(): Promise<void> {
    while (this.isRunning) {
      const pending = await this.outboxRepo.findOldestPending();
      if (pending === null) {
        await this.sleep(POLL_INTERVAL_MS);
        continue;
      }

      const success = await this.sendWithRetry(pending);
      if (success) {
        // work_events は永続保持。outbox レコードのみ削除する（Append-only 原則）
        await this.outboxRepo.delete(pending.id);
        await this.workEventRepo.markSynced(pending.eventId);
      }
    }
  }

  private async sendWithRetry(event: LocalOutboxEvent): Promise<boolean> {
    let delay = 1000;
    for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
      try {
        const token = await this.deps.jwtService.getAccessToken();
        const res = await fetch(`${this.deps.baseApiUrl}/work-events`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Idempotency-Key': event.idempotencyKey,
            Authorization: `Bearer ${token}`,
          },
          body: event.payload,
        });
        // 409 は冪等性キャッシュヒット。成功と同じ扱いで進める
        if (res.ok || res.status === 409) return true;
        if (res.status >= 400 && res.status < 500 && res.status !== 408 && res.status !== 429) {
          // 4xx の業務エラーはリトライ不可。記録した上で諦める
          await this.outboxRepo.markRetry(event.id, attempt + 1, this.nextRetryIso(MAX_BACKOFF_MS), `HTTP ${res.status}`);
          return false;
        }
        throw new Error(`HTTP ${res.status}`);
      } catch (err) {
        const message = err instanceof Error ? err.message : 'unknown';
        if (attempt === MAX_RETRIES) {
          await this.outboxRepo.markRetry(event.id, attempt + 1, this.nextRetryIso(MAX_BACKOFF_MS), message);
          return false;
        }
        await this.sleep(Math.min(delay, MAX_BACKOFF_MS));
        delay *= 2;
      }
    }
    return false;
  }

  private nextRetryIso(deltaMs: number): string {
    return new Date(Date.now() + deltaMs).toISOString();
  }

  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
