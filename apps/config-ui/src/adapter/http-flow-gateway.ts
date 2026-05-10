// 対応 §: ロードマップ §10.2 §10.3.1 §11.4.1
// HTTP REST 経由でバックエンドの flows API を叩く Gateway 実装。

// ユースケース層のインタフェース
import type { FlowGateway } from '../usecase/publish-flow';
import type { Flow } from '../domain/flow';

/** HTTP Flow Gateway 実装 */
export class HttpFlowGateway implements FlowGateway {
  // ベース URL（例: http://localhost:8080）
  private readonly baseUrl: string;
  // 任意の fetch 実装（テスト容易性のため）
  private readonly fetchFn: typeof fetch;

  /** コンストラクタ */
  constructor(baseUrl: string, fetchFn: typeof fetch = fetch) {
    // 依存を保持
    this.baseUrl = baseUrl;
    this.fetchFn = fetchFn;
  }

  /** 試行版発行 */
  async publishTrial(flow: Flow, pilotDeviceIds: ReadonlyArray<string>): Promise<void> {
    // POST /flows/<id>/trials
    const url = `${this.baseUrl}/flows/${encodeURIComponent(flow.id)}/trials`;
    // §10.3.1 Idempotency-Key 必須
    const idempotencyKey = `trial-${flow.id}-${flow.version}`;
    // 送信
    const res = await this.fetchFn(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Idempotency-Key': idempotencyKey
      },
      body: JSON.stringify({
        version: flow.version,
        pilot_device_ids: pilotDeviceIds
      })
    });
    // ステータスコードチェック
    if (!res.ok) {
      // §20.1 エラーメッセージ規約: 人を責めない
      throw new Error(`試行版発行に失敗しました（HTTP ${res.status}）`);
    }
  }

  /** 本番昇格 */
  async promoteToProduction(flowId: string, version: number): Promise<void> {
    // POST /flows/<id>/promote
    const url = `${this.baseUrl}/flows/${encodeURIComponent(flowId)}/promote`;
    // Idempotency-Key
    const idempotencyKey = `promote-${flowId}-${version}`;
    // 送信
    const res = await this.fetchFn(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Idempotency-Key': idempotencyKey
      },
      body: JSON.stringify({ version })
    });
    // ステータスチェック
    if (!res.ok) {
      // ドメインエラー
      throw new Error(`本番昇格に失敗しました（HTTP ${res.status}）`);
    }
  }
}
