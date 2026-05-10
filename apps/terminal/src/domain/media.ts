// 対応 §: ロードマップ §10.4 §10.4.1〜§10.4.6 §3.1.1
// メディアドメイン。カメラ／音声／QR／OCR の最小型を §10.4.1 表に従って表現する。

/** メディア種別（§10.4.1） */
export type MediaKind =
  | 'photo'        // 写真
  | 'burst'        // 連写
  | 'video'        // 動画
  | 'timelapse'    // タイムラプス
  | 'audio'        // 音声
  | 'qr'           // QR/バーコード
  | 'ocr';         // OCR

/** メディア参照（不可逆な追記対象、§10.4.5） */
export interface MediaRef {
  // 端末内一意 ID
  readonly id: string;
  // 種別
  readonly kind: MediaKind;
  // SHA-256 ハッシュ（hex 文字列、§10.4.5）
  readonly sha256: string;
  // ローカルファイルパス（端末暗号化領域内）
  readonly path: string;
  // バイト数
  readonly bytes: number;
  // 取得時刻（UTC ISO 8601、§20.2）
  readonly capturedAt: string;
}

/** SHA-256 ハッシュの形式検証（hex 64 文字） */
export function isValidSha256(hex: string): boolean {
  // 64 文字、すべて 0-9a-f
  return /^[0-9a-f]{64}$/i.test(hex);
}

/**
 * MediaRef を構築するファクトリ
 *
 * @throws ハッシュ形式が不正な場合
 */
export function createMediaRef(input: {
  id: string;
  kind: MediaKind;
  sha256: string;
  path: string;
  bytes: number;
  capturedAt: string;
}): MediaRef {
  // ID 空チェック
  if (input.id.length === 0) {
    throw new Error('MediaRef.id が空です');
  }
  // ハッシュ形式
  if (!isValidSha256(input.sha256)) {
    throw new Error(`SHA-256 ハッシュ形式が不正です: ${input.sha256}`);
  }
  // バイト数
  if (input.bytes < 0) {
    throw new Error('MediaRef.bytes は非負である必要があります');
  }
  // 不変参照を返す
  return Object.freeze({
    id: input.id,
    kind: input.kind,
    sha256: input.sha256.toLowerCase(),
    path: input.path,
    bytes: input.bytes,
    capturedAt: input.capturedAt
  });
}
