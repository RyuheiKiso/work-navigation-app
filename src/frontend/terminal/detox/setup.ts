// Detox グローバル型宣言と Jest セットアップ
import { device, element, by, waitFor, expect } from 'detox';

// Detox のグローバルをテストから参照できるように宣言
declare global {
  const device: typeof device;
  const element: typeof element;
  const by: typeof by;
  const waitFor: typeof waitFor;
  const expect: typeof expect;
}

beforeAll(async () => {
  await device.launchApp();
});

afterAll(async () => {
  await device.terminateApp();
});
