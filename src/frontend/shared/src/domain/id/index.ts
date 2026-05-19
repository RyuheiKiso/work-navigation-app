import { v7 as uuidv7 } from 'uuid';

// UUID v7 は時刻ソート可能なため Idempotency-Key・カーソルページングの基準に使用する
export function generateId(): string {
  return uuidv7();
}

// UUID v7 形式（version=7, variant=10xx）を検証する
const UUIDV7_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export function isValidUuidV7(value: string): boolean {
  return UUIDV7_REGEX.test(value);
}

// UUID v7 の上位 48bit にタイムスタンプ（ミリ秒）が埋め込まれているため抽出可能
export function extractTimestampFromUuidV7(uuid: string): number | null {
  if (!isValidUuidV7(uuid)) return null;
  const hex = uuid.replace(/-/g, '').slice(0, 12);
  return parseInt(hex, 16);
}
