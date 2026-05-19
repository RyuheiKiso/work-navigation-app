// 開発環境でのみ MSW サーバを起動してモック API レスポンスを提供する
export async function startMocks(): Promise<void> {
  if (typeof __DEV__ !== 'undefined' && !__DEV__) return;
  const { server } = await import('@wnav/shared/mocks/native');
  server.listen({ onUnhandledRequest: 'bypass' });
}

export async function stopMocks(): Promise<void> {
  const { server } = await import('@wnav/shared/mocks/native');
  server.close();
}
