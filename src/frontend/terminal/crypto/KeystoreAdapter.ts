// expo-secure-store を使ってプラットフォーム最適なセキュアストレージに鍵を保管する
import * as SecureStore from 'expo-secure-store';

export const KEY_PRIVATE = 'wnav.ed25519.privateKey';
export const KEY_PUBLIC = 'wnav.ed25519.publicKey';
export const KEY_JWT = 'wnav.jwt.accessToken';
export const KEY_REFRESH = 'wnav.jwt.refreshToken';
export const KEY_PIN_HASH = 'wnav.auth.pinHash';

export class KeystoreAdapter {
  // セキュアストレージへ書き込む。OS のキーチェーン/Keystore/DPAPI に紐付く
  async setItem(key: string, value: string): Promise<void> {
    await SecureStore.setItemAsync(key, value, {
      keychainAccessible: SecureStore.WHEN_UNLOCKED,
    });
  }

  async getItem(key: string): Promise<string | null> {
    return SecureStore.getItemAsync(key);
  }

  async deleteItem(key: string): Promise<void> {
    await SecureStore.deleteItemAsync(key);
  }
}
