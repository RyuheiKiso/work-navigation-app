// 対応 §: ロードマップ §17.2 §17.3
// AssemblyScript 二次 SDK の最小型定義。
// AssemblyScript は TypeScript ライクだが完全なサブセットなので、interface ではなく @abstract class を用いる。

// ホスト側が提供する関数群（外部宣言）
// 実装は wasmtime 側で `wna.*` 名前で resolve される。

/** 現在の作業 ID を取得する（capability: task.read） */
@external("wna", "get_current_task_id")
declare function getCurrentTaskIdRaw(): string;

/** 通知を送る（capability: notify:<channel>） */
@external("wna", "notify")
declare function notifyRaw(channel: string, message: string): void;

/** ロギング（既定許可） */
@external("wna", "log")
declare function logRaw(level: string, message: string): void;

/** 時刻（unix epoch 秒、既定許可） */
@external("wna", "now")
declare function nowRaw(): i64;

/** addon SDK のエクスポートクラス */
export class Host {
  /** 現在の作業 ID を取得する */
  static getCurrentTaskId(): string {
    // ホスト関数を経由する
    return getCurrentTaskIdRaw();
  }

  /** 通知を送る */
  static notify(channel: string, message: string): void {
    // ホスト関数を経由する
    notifyRaw(channel, message);
  }

  /** ログ出力 */
  static log(level: string, message: string): void {
    // ホスト関数を経由する
    logRaw(level, message);
  }

  /** 時刻を取得する */
  static now(): i64 {
    // ホスト関数を経由する
    return nowRaw();
  }
}
