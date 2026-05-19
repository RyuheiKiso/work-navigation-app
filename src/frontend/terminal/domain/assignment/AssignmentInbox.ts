// 作業指示割当のインボックス。SSE 受信 + Pull 補完 + ACK
import type { JwtService } from '../../auth/JwtService';
import { WorkAssignmentRepository } from '../../db/repositories/WorkAssignmentRepository';
import type { LocalWorkAssignment } from '../../db/entities/LocalWorkAssignment';

export interface AssignmentInboxDeps {
  baseApiUrl: string;
  jwtService: JwtService;
  terminalId: string;
}

export type AssignmentListener = (assignment: LocalWorkAssignment) => void;

export class AssignmentInbox {
  private eventSource: EventSource | null = null;
  private readonly listeners = new Set<AssignmentListener>();
  private readonly repo: WorkAssignmentRepository;
  private pullTimer: ReturnType<typeof setInterval> | null = null;

  constructor(private readonly deps: AssignmentInboxDeps) {
    this.repo = new WorkAssignmentRepository();
  }

  // SSE で push を受信し、定期 Pull で取りこぼしを補完する
  async start(): Promise<void> {
    await this.pullOnce();
    this.startPullTimer();
    this.connectSse();
  }

  stop(): void {
    if (this.eventSource !== null) {
      this.eventSource.close();
      this.eventSource = null;
    }
    if (this.pullTimer !== null) {
      clearInterval(this.pullTimer);
      this.pullTimer = null;
    }
  }

  subscribe(listener: AssignmentListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  async acknowledge(id: string): Promise<void> {
    await this.repo.acknowledge(id);
    try {
      const token = await this.deps.jwtService.getAccessToken();
      await fetch(`${this.deps.baseApiUrl}/work-assignments/${encodeURIComponent(id)}/ack`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}` },
      });
    } catch {
      // ACK の送信失敗はローカル既読のみで完結（次回 Pull で再送される）
    }
  }

  private connectSse(): void {
    if (typeof EventSource === 'undefined') return;
    const url = `${this.deps.baseApiUrl}/work-assignments/stream?terminalId=${encodeURIComponent(this.deps.terminalId)}`;
    this.eventSource = new EventSource(url);
    this.eventSource.onmessage = (event: MessageEvent<string>) => {
      try {
        const payload = JSON.parse(event.data) as LocalWorkAssignment;
        void this.repo.upsert(payload);
        for (const listener of this.listeners) listener(payload);
      } catch {
        // 不正な JSON は無視。pull で正本を取得し直す
      }
    };
    this.eventSource.onerror = () => {
      // SSE が落ちても pullTimer が補完するため再接続は次回 start() で行う
    };
  }

  private startPullTimer(): void {
    this.pullTimer = setInterval(() => {
      void this.pullOnce();
    }, 60 * 1000);
  }

  private async pullOnce(): Promise<void> {
    try {
      const token = await this.deps.jwtService.getAccessToken();
      const res = await fetch(
        `${this.deps.baseApiUrl}/work-assignments?terminalId=${encodeURIComponent(this.deps.terminalId)}&status=pending`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      if (!res.ok) return;
      const data = (await res.json()) as { data: LocalWorkAssignment[] };
      for (const assignment of data.data) {
        await this.repo.upsert(assignment);
        for (const listener of this.listeners) listener(assignment);
      }
    } catch {
      // ネットワークエラーはオフライン継続。次回 pull で再試行する
    }
  }
}
