import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { UnSyncedBadge } from '@/components/UnSyncedBadge';

describe('UnSyncedBadge', () => {
  it('未受信時は「未同期（サーバ未受信）」を表示する', () => {
    render(<UnSyncedBadge serverReceivedAt={null} clientRecordedAt="2026-05-19T00:00:00Z" />);
    expect(screen.getByText(/未同期（サーバ未受信）/)).toBeInTheDocument();
  });

  it('1 分以上の遅延がある場合は遅延付きで未同期表示', () => {
    render(
      <UnSyncedBadge
        serverReceivedAt="2026-05-19T00:10:00Z"
        clientRecordedAt="2026-05-19T00:00:00Z"
      />,
    );
    expect(screen.getByText(/未同期/)).toBeInTheDocument();
  });

  it('1 分未満の遅延では何も描画しない（同期済み相当）', () => {
    const { container } = render(
      <UnSyncedBadge
        serverReceivedAt="2026-05-19T00:00:30Z"
        clientRecordedAt="2026-05-19T00:00:00Z"
      />,
    );
    expect(container.firstChild).toBeNull();
  });
});
