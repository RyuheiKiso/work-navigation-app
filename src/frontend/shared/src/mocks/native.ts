import { setupServer } from 'msw/node';
import { handlers } from './handlers';

// terminal アプリ（React Native / Node テスト）から起動する Node ベースのモックランナー
export const server = setupServer(...handlers);
