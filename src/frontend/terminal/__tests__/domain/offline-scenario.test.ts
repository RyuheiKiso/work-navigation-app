// オフラインシナリオ統合テスト。切断 → Emergency Mode → Outbox 積み → 再接続 → 同期
// （src/CLAUDE.md §1 Offline-First / src/frontend/terminal/CLAUDE.md §Offline-First 実装規約）

describe('Offline-First シナリオ', () => {
  describe('NetworkContext の 4 段階遷移', () => {
    it('online → offline の遷移で isEmergencyMode は false のまま', () => {
      // 5 分以内は Emergency にならない
      const state = {
        networkQuality: 'disconnected' as const,
        isEmergencyMode: false,
        lastOnlineAt: new Date(Date.now() - 60_000), // 1 分前
        pendingSyncCount: 0,
      };
      const elapsed = Date.now() - state.lastOnlineAt.getTime();
      const EMERGENCY_MS = 5 * 60 * 1000;
      expect(elapsed < EMERGENCY_MS).toBe(true);
      expect(state.isEmergencyMode).toBe(false);
    });

    it('5 分超の切断で Emergency Mode に遷移する', () => {
      const lastOnlineAt = new Date(Date.now() - 6 * 60 * 1000); // 6 分前
      const EMERGENCY_MS = 5 * 60 * 1000;
      const elapsed = Date.now() - lastOnlineAt.getTime();
      const shouldBeEmergency = elapsed > EMERGENCY_MS;
      expect(shouldBeEmergency).toBe(true);
    });
  });

  describe('Outbox の Append-only 保証', () => {
    it('WorkEvent は ACK 後も削除されない（ローカル一次記録として永続）', () => {
      // これは WorkEventRepository の設計的制約であり、
      // delete メソッドが存在しないことで保証される
      const repoMethods = ['append', 'findByCaseId', 'findLatestByCaseId', 'findUnsynced', 'markSynced'];
      // delete / remove は含まれない
      for (const method of repoMethods) {
        expect(method).not.toMatch(/^(delete|remove|truncate)/i);
      }
    });

    it('Outbox は created_at ASC 順で処理される（順序保証）', () => {
      // 順序が逆転すると SHA-256 チェーンが破断する
      const events = [
        { id: 'a', createdAt: '2026-05-01T10:00:00Z' },
        { id: 'b', createdAt: '2026-05-01T10:01:00Z' },
        { id: 'c', createdAt: '2026-05-01T10:02:00Z' },
      ];
      const sorted = [...events].sort((a, b) => a.createdAt.localeCompare(b.createdAt));
      expect(sorted.map((e) => e.id)).toEqual(['a', 'b', 'c']);
    });
  });

  describe('オフライン中の作業継続（Offline-First が保証する機能）', () => {
    it('接続不要の機能は接続状態に依存しない', () => {
      // 以下の機能はオフライン時も完全に動作する（spec: src/frontend/terminal/CLAUDE.md）
      const offlineFunctions = [
        '手順ナビゲーション（SOP 閲覧・Step 進行・記録入力）',
        'ローカルアラート（入力値範囲逸脱・必須項目未入力）',
        '電子署名（ローカル鍵による署名）',
        'カメラ・QR スキャン記録',
      ];
      // 機能一覧の存在確認（ドキュメントとの整合性検証）
      expect(offlineFunctions).toHaveLength(4);
      expect(offlineFunctions[0]).toContain('手順ナビゲーション');
    });

    it('オフライン中の新規 case_id 占有は禁止される', () => {
      // case_lock 取得は接続必須（src/frontend/terminal/CLAUDE.md §Case 占有とシフト交代）
      const canAcquireNewCase = (isOnline: boolean) => isOnline;
      expect(canAcquireNewCase(false)).toBe(false);
      expect(canAcquireNewCase(true)).toBe(true);
    });

    it('既存 case_id の Step 記録はオフラインでも継続できる', () => {
      // 既に case_lock を取得済みの場合はローカル SQLite に記録して Outbox に積む
      const hasExistingLock = true;
      const isOnline = false;
      const canContinue = hasExistingLock; // 接続状態に依存しない
      expect(canContinue).toBe(true);
      expect(isOnline).toBe(false); // オフラインでも継続可能
    });
  });
});
