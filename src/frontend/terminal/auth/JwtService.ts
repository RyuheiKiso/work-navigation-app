// JWT サービス。アクセストークン取得・SecureStore 保存・リフレッシュ
import { KeystoreAdapter, KEY_JWT, KEY_REFRESH } from '../crypto/KeystoreAdapter';
import type {
  AuthLoginRequest,
  AuthLoginResponse,
  AuthRefreshResponse,
} from '@wnav/shared';

export interface JwtServiceDeps {
  baseApiUrl: string;
}

export class JwtService {
  private readonly keystore = new KeystoreAdapter();

  constructor(private readonly deps: JwtServiceDeps) {}

  async login(payload: AuthLoginRequest): Promise<AuthLoginResponse> {
    const res = await fetch(`${this.deps.baseApiUrl}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      throw new Error(`login failed: HTTP ${res.status}`);
    }
    const body = (await res.json()) as { data: AuthLoginResponse };
    await this.keystore.setItem(KEY_JWT, body.data.accessToken);
    await this.keystore.setItem(KEY_REFRESH, body.data.refreshToken);
    return body.data;
  }

  // アクセストークン取得。期限切れ時はリフレッシュを試みる
  async getAccessToken(): Promise<string> {
    const token = await this.keystore.getItem(KEY_JWT);
    if (token !== null) return token;
    return this.refresh();
  }

  async refresh(): Promise<string> {
    const refreshToken = await this.keystore.getItem(KEY_REFRESH);
    if (refreshToken === null) throw new Error('no refresh token');
    const res = await fetch(`${this.deps.baseApiUrl}/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refreshToken }),
    });
    if (!res.ok) throw new Error(`refresh failed: HTTP ${res.status}`);
    const body = (await res.json()) as { data: AuthRefreshResponse };
    await this.keystore.setItem(KEY_JWT, body.data.accessToken);
    return body.data.accessToken;
  }

  async logout(): Promise<void> {
    await this.keystore.deleteItem(KEY_JWT);
    await this.keystore.deleteItem(KEY_REFRESH);
  }
}
