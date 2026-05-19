import { setupWorker } from 'msw/browser';
import { handlers } from './handlers';

// master アプリ（ブラウザ）から起動する Service Worker ベースのモックランナー
export const worker = setupWorker(...handlers);
