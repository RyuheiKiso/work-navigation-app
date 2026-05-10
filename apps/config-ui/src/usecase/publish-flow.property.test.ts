// 対応 §: ロードマップ §13.1 §10.2 §10.2.4 ／ ルート CLAUDE.md
// PublishFlowUseCase の不変条件を fast-check で検証する。
// - パイロット端末空配列は常に例外で gateway は呼ばれない
// - 非空配列ならゲートウェイへ exactly once 委譲される
// - promoteToProduction はゲートウェイへ素通し

import { describe, it, expect, vi } from 'vitest';
import fc from 'fast-check';
import { PublishFlowUseCase, type FlowGateway } from './publish-flow';
import { Flow, type FlowEdge, type FlowNode } from '../domain/flow';

function sampleFlow(): Flow {
  const nodes: FlowNode[] = [
    { id: 'start', kind: 'start', label: '開始' },
    { id: 'end', kind: 'end', label: '終了' }
  ];
  const edges: FlowEdge[] = [{ from: 'start', to: 'end' }];
  return Flow.create('f1', 'sample', nodes, edges);
}

function spyGateway(): { gateway: FlowGateway; trial: ReturnType<typeof vi.fn>; promote: ReturnType<typeof vi.fn> } {
  const trial = vi.fn().mockResolvedValue(undefined);
  const promote = vi.fn().mockResolvedValue(undefined);
  return { gateway: { publishTrial: trial, promoteToProduction: promote }, trial, promote };
}

describe('PublishFlowUseCase (property-based)', () => {
  it('パイロット端末ゼロでは gateway を呼ばず例外', async () => {
    const { gateway, trial } = spyGateway();
    const uc = new PublishFlowUseCase(gateway);
    await expect(uc.publishTrial(sampleFlow(), [])).rejects.toThrow(/配信先が空/);
    expect(trial).not.toHaveBeenCalled();
  });

  it('非空配列なら gateway.publishTrial が exactly once 呼ばれる', async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.array(fc.string({ minLength: 1, maxLength: 16 }), { minLength: 1, maxLength: 5 }),
        async (devices) => {
          const { gateway, trial } = spyGateway();
          const uc = new PublishFlowUseCase(gateway);
          await uc.publishTrial(sampleFlow(), devices);
          expect(trial).toHaveBeenCalledTimes(1);
          expect(trial.mock.calls[0]?.[1]).toEqual(devices);
        }
      )
    );
  });

  it('promoteToProduction は gateway へ素通し', async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.string({ minLength: 1, maxLength: 16 }),
        fc.integer({ min: 1, max: 1000 }),
        async (flowId, version) => {
          const { gateway, promote } = spyGateway();
          const uc = new PublishFlowUseCase(gateway);
          await uc.promoteToProduction(flowId, version);
          expect(promote).toHaveBeenCalledTimes(1);
          expect(promote.mock.calls[0]).toEqual([flowId, version]);
        }
      )
    );
  });
});
