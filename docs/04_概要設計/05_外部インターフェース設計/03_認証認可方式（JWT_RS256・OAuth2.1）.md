# 03 認証認可方式（JWT RS256・OAuth 2.1）

本章の責務は、JWT RS256 による認証・RBAC 6 ロール × 全 API の認可マトリクス・OAuth 2.1 を使用した子機モード認証の設計を確定することである。

---

## 1. JWT RS256 設計（内部認証・IF-003）

### 1-1. JWT 構造

```json
// Header
{ "alg": "RS256", "typ": "JWT", "kid": "2026-Q2" }

// Payload
{
  "sub": "{user_id}",
  "iss": "wnav.factory.example",
  "aud": "wnav-api-v1",
  "iat": 1716000000,
  "exp": 1716028800,    // 8時間後（CFG-005）
  "roles": ["operator"],
  "factory_id": "{factory_uuid}",
  "device_id": "{device_uuid}"
}
```

### 1-2. JWT 運用規約

| 項目 | 設定 | 設定 ID |
|---|---|---|
| アルゴリズム | RS256（RSA 4096bit）| KEY-001 |
| 有効期限 | 8 時間（シフト 1 本）| CFG-005 |
| 鍵ローテーション | 90 日（grace period 24h 並行稼働）| CFG-006 / KEY-001 |
| リフレッシュ | `POST /api/v1/auth/refresh`（API-auth-002）|
| 失効 | TBL-032（auth_logs の logout 記録）+ TBL（JWT ブラックリスト相当）|

### 1-3. LDAP/AD フォールバック（IF-003）

```
認証フロー:
1. `POST /api/v1/auth/login`（API-auth-001）でユーザー名・パスワードを受け取る
2. LDAP（IF-003）で BIND 認証を試みる
3. LDAP 接続不可 → TBL-016 users のパスワードハッシュ（bcrypt）でローカル認証
4. 認証成功 → JWT を発行
5. TBL-032 auth_logs に LOG-003（success）または LOG-004（failure）を記録
```

---

## 2. RBAC 6 ロール × API 認可マトリクス（主要抜粋）

完全版は `07_セキュリティ方式設計/02_RBAC設計.md` と `付録/01_DTM.md` M6（PRM-NNN）で管理。

| API-ID | メソッド | URL | operator | supervisor | master_admin | quality_admin | system_admin | executive |
|---|---|---|---|---|---|---|---|---|
| API-auth-001 | POST | /auth/login | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| API-work-execs-001 | POST | /work-executions | ✅ | ✅ | — | — | — | — |
| API-step-events-001 | POST | /work-executions/{id}/events | ✅ | ✅ | — | — | — | — |
| API-master-001 | GET | /master-versions | ✅（参照のみ）| ✅ | ✅ | ✅ | — | — |
| API-master-005 | POST | /master-versions/{id}/approve | — | — | — | ✅ | — | — |
| API-electronic-signs-001 | POST | /electronic-signs | ✅ | ✅ | ✅ | ✅ | — | — |
| API-reports-002 | POST | /reports/audit-xes | — | — | — | ✅ | ✅ | — |
| API-ops-001 | GET | /ops/outbox/dlq | — | — | — | — | ✅ | — |
| API-system-001 | GET | /healthz | ✅（全員）| ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 3. OAuth 2.1 Client Credentials（子機モード・親機認証）

子機から親機への実績送信（IF-002）は OAuth 2.1 Client Credentials フローで認証する。

```
1. [子機バックエンド] クライアント ID + シークレットで親機 Token Endpoint にリクエスト
   POST {親機}/oauth/token
   Body: grant_type=client_credentials&scope=wnav.outbox.write

2. [親機] アクセストークン（JWT or Opaque）を返す

3. [子機バックエンド] アクセストークン付きで親機 API にリクエスト
   POST {親機}/api/v1/sync/outbox/inbound
   Authorization: Bearer {access_token}
   Idempotency-Key: {UUID v7}
```

代替: mTLS（`KEY-008` クライアント証明書）による相互認証。OAuth が使用不可な環境では mTLS で代替する（設定ファイルで切替え可能）。

---

**本節で確定した方針**
- **内部認証は JWT RS256（RSA 4096bit）・8 時間 TTL・90 日鍵ローテーションで確定した。LDAP 不可時のローカル認証フォールバックを§1-3 で確定した。**
- **RBAC 6 ロール × 全 API の認可マトリクスの骨格を確定し、完全版は 07_セキュリティ方式設計と DTM M6 で管理する。**
- **親機認証は OAuth 2.1 Client Credentials または mTLS の二択とし、設定ファイルで切替え可能な設計を確定した（計画 12 章のプラガブル認証アダプタ）。**

---

## 参照業界分析

### 必須
- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)

### 関連
- [`90_業界分析/13_安全文化と安全管理システム.md`](../../90_業界分析/13_安全文化と安全管理システム.md)
