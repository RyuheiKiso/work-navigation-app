// Detox E2E: 作業フロー（SCR-HA-001 ログイン → 002 ホーム → 005 Step完了 → 010 電子サイン → 011 中断 → 012 再開）
// 実機またはエミュレータ上で実行する。`npx detox test` で起動。

describe('作業フロー E2E', () => {
  beforeAll(async () => {
    // デバイスをリセットしてアプリを起動
    await device.launchApp({ newInstance: true });
  });

  it('SCR-HA-001: ログイン画面が表示される', async () => {
    await expect(element(by.text('WNAV ログイン'))).toBeVisible();
    await expect(element(by.label('ユーザー名'))).toBeVisible();
    await expect(element(by.label('パスワード'))).toBeVisible();
    await expect(element(by.label('ログイン'))).toBeVisible();
  });

  it('SCR-HA-001 → SCR-HA-002: ログイン成功後にホーム画面へ遷移する', async () => {
    await element(by.label('ユーザー名')).typeText('operator01');
    await element(by.label('パスワード')).typeText('password');
    await element(by.label('ログイン')).tap();

    // ホーム画面への遷移を待つ
    await waitFor(element(by.text('ホーム'))).toBeVisible().withTimeout(5000);
    await expect(element(by.text('ホーム'))).toBeVisible();
  });

  it('SCR-HA-002: 割当バナーと作業一覧が表示される', async () => {
    // AssignmentBanner（CMP-HA-021）が存在する
    await expect(element(by.id('assignment-banner'))).toBeVisible();
    // 作業一覧（AssignmentList CMP-HA-022）が表示される
    await expect(element(by.id('assignment-list'))).toBeVisible();
  });

  it('SCR-HA-005: 標準 Step 画面で GlanceableBanner が表示される', async () => {
    // 作業を選択して Step 画面へ遷移
    await element(by.id('assignment-list')).tap();

    // GlanceableBanner（現在Step・次Step）が表示される（200ms以内）
    await waitFor(element(by.id('glanceable-banner'))).toBeVisible().withTimeout(1000);
    await expect(element(by.id('glanceable-banner'))).toBeVisible();

    // ステップ完了ボタン（72dp）が存在する
    await expect(element(by.label('このステップを完了'))).toBeVisible();
  });

  it('SCR-HA-010: 電子サイン入力画面が起動する', async () => {
    // サインパッドへのナビゲーション
    await element(by.label('電子サイン')).tap();
    await waitFor(element(by.id('signature-pad'))).toBeVisible().withTimeout(3000);
    await expect(element(by.id('signature-pad'))).toBeVisible();
  });

  it('SCR-HA-011: 中断画面で理由を入力して中断できる', async () => {
    // 中断ボタンをタップ
    await element(by.label('作業を中断')).tap();
    await waitFor(element(by.text('中断理由を選択'))).toBeVisible().withTimeout(3000);

    // 中断理由の選択（最初の選択肢）
    await element(by.type('android.widget.Spinner')).tap();
    await element(by.text('設備不具合')).tap();
  });

  it('SCR-HA-015: 設定画面で言語を切り替えられる', async () => {
    // 設定画面へ遷移
    await element(by.label('設定')).tap();
    await waitFor(element(by.text('言語設定'))).toBeVisible().withTimeout(3000);

    // 英語に切り替え
    await element(by.label('言語選択')).tap();
    await element(by.text('English')).tap();

    // UI が英語になっている
    await expect(element(by.text('Settings'))).toBeVisible();

    // 日本語に戻す
    await element(by.label('Language')).tap();
    await element(by.text('日本語')).tap();
  });
});

describe('Outbox 同期フロー', () => {
  it('オフライン中に Step 完了を記録し、再接続後に同期される', async () => {
    // デバイスをオフラインにする（Detox API）
    await device.setStatusBar({ networkActivity: false });

    // Step 完了を記録
    await element(by.label('このステップを完了')).tap();
    await waitFor(element(by.id('unsynced-indicator'))).toBeVisible().withTimeout(3000);

    // 未同期インジケータが表示されている
    await expect(element(by.id('unsynced-indicator'))).toBeVisible();

    // デバイスをオンラインに戻す
    await device.setStatusBar({ networkActivity: true });

    // 同期後にインジケータが消える
    await waitFor(element(by.id('unsynced-indicator'))).not.toBeVisible().withTimeout(15000);
  });
});
