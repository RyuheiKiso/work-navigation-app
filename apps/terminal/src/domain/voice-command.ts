// 対応 §: ロードマップ §10.4.3 §11.2.2 §28
// ハンズフリー音声コマンド：6 既定セット（§10.4.3）。
// 認識結果を VoiceCommand 列挙型へ正規化する。

/** 既定音声コマンド（§10.4.3） */
export type VoiceCommand = 'start' | 'complete' | 'suspend' | 'back' | 'memo' | 'capture';

/** 各ロケールの読み合わせ表（§28 用語と整合） */
const RECOGNITION_TABLE: Record<VoiceCommand, ReadonlyArray<string>> = {
  // 「開始」「スタート」「はじめ」
  start: ['開始', 'はじめ', 'スタート', 'start', '시작', '开始', 'starten', 'iniciar'],
  // 「完了」「終わり」
  complete: ['完了', 'おわり', 'コンプリート', 'complete', '완료', '完成', 'fertig', 'completar'],
  // 「中断」「ストップ」
  suspend: ['中断', 'いったん', 'サスペンド', 'suspend', '중단', '暂停', 'pause', 'pausar'],
  // 「戻る」「バック」
  back: ['戻る', 'もどる', 'バック', 'back', '뒤로', '返回', 'zurück', 'volver'],
  // 「メモ」
  memo: ['メモ', 'めも', 'memo', '메모', '备注', 'notiz', 'nota'],
  // 「撮影」
  capture: ['撮影', 'さつえい', 'シャッター', 'capture', 'shutter', '촬영', '拍照', 'aufnahme', 'capturar']
};

/**
 * 認識文字列を VoiceCommand に正規化する。
 *
 * 大文字小文字を区別しない簡易マッチ。多言語に対応するため §28 用語を含む。
 *
 * @returns マッチした VoiceCommand、未マッチなら null
 */
export function normalize(recognized: string): VoiceCommand | null {
  // 入力をトリム＆小文字化
  const t = recognized.trim().toLowerCase();
  // 空文字は null
  if (t.length === 0) return null;
  // テーブル走査
  for (const [cmd, aliases] of Object.entries(RECOGNITION_TABLE) as [
    VoiceCommand,
    ReadonlyArray<string>
  ][]) {
    for (const a of aliases) {
      // 完全一致 or 部分一致（「開始してください」等を吸収）
      if (t === a.toLowerCase() || t.includes(a.toLowerCase())) {
        return cmd;
      }
    }
  }
  // 未マッチ
  return null;
}

/** 音声認識器の抽象（実装は Tauri 側プラグインで提供） */
export interface VoiceRecognizer {
  /** 録音→認識→文字列を返す（タイムアウト超は null） */
  listen(timeoutMs: number): Promise<string | null>;
}
