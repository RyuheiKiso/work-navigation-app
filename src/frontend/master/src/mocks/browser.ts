// MSW のブラウザワーカーは shared/mocks/browser に集約されている（タスク #3）。
// 開発モードでのみ起動し、未ハンドルリクエストは bypass する（onUnhandledRequest: 'bypass'）。
import { worker } from '@wnav/shared/mocks/browser';

export async function enableMocking(): Promise<void> {
  if (import.meta.env['MODE'] !== 'development') return;
  if (import.meta.env['VITE_ENABLE_MSW'] !== 'true') return;
  await worker.start({
    onUnhandledRequest: 'bypass',
    serviceWorker: { url: '/mockServiceWorker.js' },
  });
}

export { worker };
