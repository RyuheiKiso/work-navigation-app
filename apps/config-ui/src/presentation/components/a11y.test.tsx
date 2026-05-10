// 対応 §: ロードマップ §11.2 §13.1 §13.2 ／ ADR-0012
// 主要 UI コンポーネントの a11y 自動検査（axe-core）。
// 実機 e2e ではなく、レンダリング時に明らかな違反（label 欠落・コントラスト・
// role 不整合など）を CI で検知する。
//
// vitest-axe v0.1 のカスタムマッチャは vitest 1.6 の型に追随しておらず、
// `toHaveNoViolations` を tsc に教えるのが難しい。代わりに `axe()` の
// 結果を直接 expect する。実機能（違反検知）は変わらない。

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { axe } from 'vitest-axe';
import { ConfirmDialog } from './confirm-dialog';
import { ErrorPanel } from '../states/error-panel';
import { EmptyState } from '../states/empty-state';

async function noViolations(node: Element): Promise<void> {
  const results = await axe(node);
  if (results.violations.length > 0) {
    const summary = results.violations
      .map((v) => `${v.id}: ${v.help} (${v.nodes.length} node(s))`)
      .join('\n');
    throw new Error(`a11y violations:\n${summary}`);
  }
  expect(results.violations).toHaveLength(0);
}

describe('a11y (axe-core)', () => {
  it('ConfirmDialog has no violations when open', async () => {
    const { container } = render(
      <ConfirmDialog
        open={true}
        title="重要な操作"
        description="本当に実行しますか？"
        confirmLabel="実行"
        cancelLabel="取消"
        variant="danger"
        onConfirm={() => undefined}
        onCancel={() => undefined}
      />
    );
    await noViolations(container);
  });

  it('ErrorPanel has no violations', async () => {
    const { container } = render(
      <ErrorPanel
        message="サーバに接続できません"
        onRetry={() => undefined}
        onDismiss={() => undefined}
      />
    );
    await noViolations(container);
  });

  it('EmptyState has no violations', async () => {
    const { container } = render(
      <EmptyState icon="📭" title="データがありません" description="補助文" />
    );
    await noViolations(container);
  });
});
