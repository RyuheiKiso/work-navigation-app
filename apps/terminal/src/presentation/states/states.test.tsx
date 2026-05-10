// 対応 §: ロードマップ §13.1 §3.6.4
// 三状態 UI コンポーネントの単体テスト。

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { LoadingState } from './loading-state';
import { EmptyState } from './empty-state';
import { ErrorPanel } from './error-panel';

describe('LoadingState', () => {
  it('exposes role=status with the given label', () => {
    render(<LoadingState label="読込中" />);
    expect(screen.getByRole('status')).toHaveTextContent('読込中');
  });
});

describe('EmptyState', () => {
  it('renders title and optional description', () => {
    render(<EmptyState icon="📭" title="0 件です" description="補助文" />);
    expect(screen.getByText('0 件です')).toBeInTheDocument();
    expect(screen.getByText('補助文')).toBeInTheDocument();
  });

  it('renders an action when provided', () => {
    render(
      <EmptyState
        title="empty"
        action={<button type="button">追加</button>}
      />
    );
    expect(screen.getByRole('button', { name: '追加' })).toBeInTheDocument();
  });
});

describe('ErrorPanel', () => {
  it('shows the message and dismiss/retry buttons', () => {
    const onRetry = vi.fn();
    const onDismiss = vi.fn();
    render(<ErrorPanel message="失敗しました" onRetry={onRetry} onDismiss={onDismiss} />);
    expect(screen.getByRole('alert')).toHaveTextContent('失敗しました');
    fireEvent.click(screen.getByText('再試行'));
    expect(onRetry).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByText('閉じる'));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('toggles detail block when provided', () => {
    render(<ErrorPanel message="msg" detail="HTTP 500" />);
    expect(screen.queryByText('HTTP 500')).toBeNull();
    fireEvent.click(screen.getByText('詳細'));
    expect(screen.getByText('HTTP 500')).toBeInTheDocument();
  });
});
