// 対応 §: ロードマップ §10.2 §10.2.1 §10.6
// 試行版 → パイロット → 本番昇格のユースケース骨格。

// ドメイン依存
import type { Flow } from '../domain/flow';

/** Flow ゲートウェイ（adapter 層が実装） */
export interface FlowGateway {
  /** 試行版を発行する（特定パイロット端末群へ） */
  publishTrial(flow: Flow, pilotDeviceIds: ReadonlyArray<string>): Promise<void>;
  /** 本番昇格 */
  promoteToProduction(flowId: string, version: number): Promise<void>;
}

/** Publish Flow ユースケース */
export class PublishFlowUseCase {
  // ゲートウェイ依存
  private readonly gateway: FlowGateway;

  /** コンストラクタ */
  constructor(gateway: FlowGateway) {
    // 依存を保持
    this.gateway = gateway;
  }

  /** 試行版発行 */
  async publishTrial(
    flow: Flow,
    pilotDeviceIds: ReadonlyArray<string>
  ): Promise<void> {
    // §10.2.4 受入観点: 試行版発行前にストレス予測スコア検査が必要
    // ここでは骨格のみ。スコア計算ロジックは §10.2.3 実装で追加する。
    if (pilotDeviceIds.length === 0) {
      // ドメインエラー: 配信先ゼロは無意味
      throw new Error('パイロット端末の配信先が空です');
    }
    // ゲートウェイへ委譲
    await this.gateway.publishTrial(flow, pilotDeviceIds);
  }

  /** 本番昇格 */
  async promoteToProduction(flowId: string, version: number): Promise<void> {
    // ゲートウェイへ委譲
    await this.gateway.promoteToProduction(flowId, version);
  }
}
