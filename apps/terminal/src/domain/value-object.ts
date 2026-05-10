// 対応 §: ロードマップ §3.1.1 §10.6.1 §28
// 端末側ドメイン層の値オブジェクト。バックエンドの Rust 側と同義語禁止規約で揃える（§28）。

/**
 * TaskId — 作業（Task）のグローバル一意 ID。
 *
 * 値オブジェクト。空文字を不正とする（§3.1.1 「識別」）。
 */
export class TaskId {
  // 内部に文字列値を保持する不変フィールド
  private readonly value: string;

  // コンストラクタは validate 後にしか呼べないよう private にする
  private constructor(value: string) {
    // 値を保持する
    this.value = value;
  }

  /**
   * 文字列から TaskId を構築する。
   * 空・1024 文字超は例外。
   */
  static of(raw: string): TaskId {
    // 空文字を弾く
    if (raw.length === 0) {
      // ドメインエラーを投げる
      throw new Error('TaskId が空です');
    }
    // 長さ制限
    if (raw.length > 1024) {
      // ドメインエラーを投げる
      throw new Error('TaskId が長すぎます');
    }
    // 妥当値で構築
    return new TaskId(raw);
  }

  /** 文字列表現を取得する */
  toString(): string {
    // 内部値を返す
    return this.value;
  }

  /** 値の等価性 */
  equals(other: TaskId): boolean {
    // 文字列比較
    return this.value === other.value;
  }
}

/**
 * DeviceId — 端末識別子（§10.6.1）。バックエンドでは UUID v7 を採用するが、
 * 端末側では文字列として受け取り検証のみを行う。
 */
export class DeviceId {
  // 内部値
  private readonly value: string;

  // private コンストラクタ
  private constructor(value: string) {
    // 値を保持する
    this.value = value;
  }

  /** 文字列から DeviceId を構築する */
  static of(raw: string): DeviceId {
    // 空文字を弾く
    if (raw.length === 0) {
      throw new Error('DeviceId が空です');
    }
    // 長さ制限
    if (raw.length > 64) {
      throw new Error('DeviceId が長すぎます');
    }
    // 妥当値で構築
    return new DeviceId(raw);
  }

  /** 文字列表現を取得する */
  toString(): string {
    // 内部値を返す
    return this.value;
  }
}

/**
 * LamportTimestamp — Lamport タイムスタンプ（§10.6.1 INV-08）。
 *
 * 単調増加する非負整数。境界では `bigint` を使用。
 */
export class LamportTimestamp {
  // 内部値
  private readonly value: bigint;

  // private コンストラクタ
  private constructor(value: bigint) {
    // 値を保持する
    this.value = value;
  }

  /** ゼロ値 */
  static zero(): LamportTimestamp {
    // 0n で初期化
    return new LamportTimestamp(0n);
  }

  /** bigint から構築する */
  static fromBigInt(v: bigint): LamportTimestamp {
    // 負値を弾く
    if (v < 0n) {
      // 不正値
      throw new Error('LamportTimestamp は非負である必要があります');
    }
    // 妥当値で構築
    return new LamportTimestamp(v);
  }

  /** 単調インクリメント */
  next(): LamportTimestamp {
    // bigint なので u64 オーバーフローは事実上発生しない
    return new LamportTimestamp(this.value + 1n);
  }

  /** 値を取り出す */
  toBigInt(): bigint {
    // 内部値を返す
    return this.value;
  }

  /** 大小比較 */
  isAfter(other: LamportTimestamp): boolean {
    // bigint 比較
    return this.value > other.value;
  }
}
