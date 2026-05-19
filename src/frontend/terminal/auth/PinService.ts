// ローカル PIN サービス。PIN は SHA-256 でハッシュ化して SecureStore に保管する
import { KeystoreAdapter, KEY_PIN_HASH } from '../crypto/KeystoreAdapter';
import { sha256Hex } from '../crypto/sha256';

export class PinService {
  private readonly keystore = new KeystoreAdapter();

  async setPin(pin: string): Promise<void> {
    if (pin.length < 4) throw new Error('PIN は 4 桁以上必要です');
    await this.keystore.setItem(KEY_PIN_HASH, sha256Hex(pin));
  }

  // 入力 PIN を保存ハッシュと比較する。一致時に true
  async verifyPin(pin: string): Promise<boolean> {
    const stored = await this.keystore.getItem(KEY_PIN_HASH);
    if (stored === null) return false;
    return sha256Hex(pin) === stored;
  }

  async clearPin(): Promise<void> {
    await this.keystore.deleteItem(KEY_PIN_HASH);
  }

  // 署名イベントの payload に格納するための PIN ハッシュを取得する
  async getPinHash(): Promise<string | null> {
    return this.keystore.getItem(KEY_PIN_HASH);
  }
}
