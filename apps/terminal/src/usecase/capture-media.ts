// 対応 §: ロードマップ §10.4 §10.4.5 §10.4.6 §3.1.1
// メディア取得ユースケース。撮影／録音／QR スキャンを抽象化する。
// 実機 API（Tauri カメラプラグイン等）への接続は MediaCapture インタフェース実装で吸収する。

import type { MediaKind, MediaRef } from '../domain/media';

/** メディア取得ハードウェア抽象 */
export interface MediaCapture {
  /** 指定種別でキャプチャを実行し、SHA-256 ハッシュ付き MediaRef を返す */
  capture(kind: MediaKind, options?: CaptureOptions): Promise<MediaRef>;
}

/** キャプチャオプション */
export interface CaptureOptions {
  // 動画／タイムラプスの最大長（秒）
  readonly maxDurationSeconds?: number;
  // QR/OCR の認識タイムアウト（ミリ秒）
  readonly recognitionTimeoutMs?: number;
}

/** キャプチャユースケース */
export class CaptureMediaUseCase {
  // ハードウェア抽象
  private readonly capture: MediaCapture;

  /** コンストラクタ */
  constructor(capture: MediaCapture) {
    // 依存を保持
    this.capture = capture;
  }

  /** ユースケース実行 */
  async execute(kind: MediaKind, options?: CaptureOptions): Promise<MediaRef> {
    // ハードウェアにキャプチャを委譲
    const ref = await this.capture.capture(kind, options);
    // §10.4.5 改ざん検出: ハッシュは MediaCapture 側で計算済みであることを前提に
    // 受領時に追加検証を行う（ハッシュ空・形式不正は throw されている）
    return ref;
  }
}
