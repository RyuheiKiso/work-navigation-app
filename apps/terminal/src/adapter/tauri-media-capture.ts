// 対応 §: ロードマップ §10.4 §7.1
// Tauri command 経由のメディアキャプチャ実装。
// Tauri 側の Rust コマンドが実装されたらそこへ委譲する（現状は擬似返却）。

import type { CaptureOptions, MediaCapture } from '../usecase/capture-media';
import type { MediaKind, MediaRef } from '../domain/media';
import { createMediaRef } from '../domain/media';

/** Tauri invoke 関数の型 */
type InvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

/** Tauri 経由の MediaCapture 実装 */
export class TauriMediaCapture implements MediaCapture {
  // invoke 関数（差替可能）
  private readonly invokeFn: InvokeFn;

  /** コンストラクタ */
  constructor(invokeFn: InvokeFn) {
    // 依存を保持
    this.invokeFn = invokeFn;
  }

  /** capture 実装 */
  async capture(kind: MediaKind, options?: CaptureOptions): Promise<MediaRef> {
    // Tauri command を呼び出す
    const dto = await this.invokeFn<{
      id: string;
      kind: MediaKind;
      sha256: string;
      path: string;
      bytes: number;
      captured_at: string;
    }>('capture_media', {
      kind,
      maxDurationSeconds: options?.maxDurationSeconds,
      recognitionTimeoutMs: options?.recognitionTimeoutMs
    });
    // ドメイン値オブジェクトに射影
    return createMediaRef({
      id: dto.id,
      kind: dto.kind,
      sha256: dto.sha256,
      path: dto.path,
      bytes: dto.bytes,
      capturedAt: dto.captured_at
    });
  }
}
