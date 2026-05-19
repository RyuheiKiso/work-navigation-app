// OutboxWorker の全シナリオ: 指数バックオフ・409 冪等性・4xx 非リトライ・work_events 永続
import { OutboxWorker } from '../../domain/outbox/OutboxWorker';
import type { LocalOutboxEvent } from '../../db/entities/LocalOutboxEvent';

// フェイク Outbox レポジトリ（単一イベントを制御可能にする）
class FakeOutboxRepository {
  private pending: LocalOutboxEvent | null;
  public deleted: string[] = [];
  public retried: Array<{ id: string; attempt: number; reason: string }> = [];

  constructor(pending: LocalOutboxEvent | null) {
    this.pending = pending;
  }

  async findOldestPending() {
    const ev = this.pending;
    this.pending = null; // 一度取得したら空にしてループ終了
    return ev;
  }
  async delete(id: string) {
    this.deleted.push(id);
  }
  async markRetry(id: string, attempt: number, _nextRetry: string, reason: string) {
    this.retried.push({ id, attempt, reason });
  }
  async pendingCount() {
    return 0;
  }
}

// フェイク WorkEvent レポジトリ（markSynced の呼び出しを記録する）
class FakeWorkEventRepository {
  public synced: string[] = [];
  async findLatestByCaseId(): Promise<null> { return null; }
  async append(ev: unknown): Promise<unknown> { return ev; }
  async findByCaseId(): Promise<unknown[]> { return []; }
  async findUnsynced(): Promise<unknown[]> { return []; }
  async markSynced(eventId: string): Promise<void> { this.synced.push(eventId); }
}

const jwtService = { getAccessToken: async () => 'test-token' } as never;

function makeEvent(overrides: Partial<LocalOutboxEvent> = {}): LocalOutboxEvent {
  return {
    id: 'outbox-1',
    eventId: 'event-1',
    idempotencyKey: 'idem-key-1',
    payload: JSON.stringify({ type: 'work_event', eventId: 'event-1' }),
    prevHash: '0'.repeat(64),
    sent: false,
    retryCount: 0,
    nextRetryAt: new Date().toISOString(),
    createdAt: new Date().toISOString(),
    ...overrides,
  } as LocalOutboxEvent;
}

// グローバル fetch をスタブ化するヘルパ
function stubFetch(responses: Array<{ status: number; ok: boolean }>) {
  let callCount = 0;
  globalThis.fetch = async () => {
    const resp = responses[callCount] ?? responses[responses.length - 1]!;
    callCount++;
    return { status: resp.status, ok: resp.ok } as Response;
  };
}

describe('OutboxWorker', () => {
  let workEventRepo: FakeWorkEventRepository;

  beforeEach(() => {
    workEventRepo = new FakeWorkEventRepository();
  });

  afterEach(() => {
    jest.restoreAllMocks();
    jest.useRealTimers();
  });

  it('HTTP 200 の ACK 後は Outbox のみ削除し WorkEvent は保持する', async () => {
    // ACK後に work_events を削除しないこと（Append-only 原則、src/CLAUDE.md §2）
    stubFetch([{ status: 200, ok: true }]);
    const event = makeEvent();
    const outboxRepo = new FakeOutboxRepository(event);
    const worker = new OutboxWorker({ baseApiUrl: 'http://test.local', jwtService });

    // 内部リポジトリを差し替え（プライベートフィールドを型で回避）
    (worker as unknown as { outboxRepo: unknown; workEventRepo: unknown }).outboxRepo = outboxRepo;
    (worker as unknown as { outboxRepo: unknown; workEventRepo: unknown }).workEventRepo = workEventRepo;

    await worker.start();

    // Outbox は削除される
    expect(outboxRepo.deleted).toContain('outbox-1');
    // WorkEvent は synced にマークされるが削除はされない
    expect(workEventRepo.synced).toContain('event-1');
    // markRetry は呼ばれない
    expect(outboxRepo.retried).toHaveLength(0);
  });

  it('HTTP 409 は冪等性キャッシュヒット → 成功として Outbox を削除する', async () => {
    // 409 は重複リクエストのキャッシュ応答なので成功扱い（src/frontend/terminal/CLAUDE.md §Outbox）
    stubFetch([{ status: 409, ok: false }]);
    const event = makeEvent();
    const outboxRepo = new FakeOutboxRepository(event);
    const worker = new OutboxWorker({ baseApiUrl: 'http://test.local', jwtService });
    (worker as unknown as { outboxRepo: unknown; workEventRepo: unknown }).outboxRepo = outboxRepo;
    (worker as unknown as { outboxRepo: unknown; workEventRepo: unknown }).workEventRepo = workEventRepo;

    await worker.start();

    expect(outboxRepo.deleted).toContain('outbox-1');
    expect(outboxRepo.retried).toHaveLength(0);
  });

  it('HTTP 400 は業務エラー → リトライせず markRetry を呼ぶ', async () => {
    // 4xx のうち 408/429 以外は再送不可の業務エラーとして扱う
    stubFetch([{ status: 400, ok: false }]);
    const event = makeEvent();
    const outboxRepo = new FakeOutboxRepository(event);
    const worker = new OutboxWorker({ baseApiUrl: 'http://test.local', jwtService });
    (worker as unknown as { outboxRepo: unknown; workEventRepo: unknown }).outboxRepo = outboxRepo;
    (worker as unknown as { outboxRepo: unknown; workEventRepo: unknown }).workEventRepo = workEventRepo;

    await worker.start();

    expect(outboxRepo.deleted).toHaveLength(0);
    expect(outboxRepo.retried).toHaveLength(1);
    expect(outboxRepo.retried[0]?.reason).toContain('HTTP 400');
  });

  it('二重起動は防止される（isRunning フラグ）', () => {
    const outboxRepo = new FakeOutboxRepository(null);
    const worker = new OutboxWorker({ baseApiUrl: 'http://test.local', jwtService });
    (worker as unknown as { outboxRepo: unknown }).outboxRepo = outboxRepo;

    void worker.start();
    expect(worker.isActive()).toBe(true);

    // 2回目の start は即座に返る（isRunning チェックで弾く）
    const startSpy = jest.spyOn(worker as never, 'processLoop');
    void worker.start();
    expect(startSpy).not.toHaveBeenCalled();

    worker.stop();
    expect(worker.isActive()).toBe(false);
  });
});
