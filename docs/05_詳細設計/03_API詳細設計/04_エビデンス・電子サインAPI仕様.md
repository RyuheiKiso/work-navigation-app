# 04 エビデンス・電子サイン API 仕様

本章は API-evidences-001（写真アップロード）・API-electronic-signs-001〜003（電子サイン作成・取得・一覧）を確定する。ファイルサイズ上限・解像度検証・SHA-256 整合性確認・RBAC 要件を網羅する。

---

## 1. API-evidences-001: POST /api/v1/evidences

### 1-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-evidences-001 |
| HTTP メソッド | POST |
| URL | `/api/v1/evidences` |
| 認証要否 | 必須 |
| Content-Type | `multipart/form-data` |
| Idempotency-Key | 必須 |
| レート制限カテゴリ | 書き込み（500 req / 60s）|
| 関連 FR | FR-EV-002 |

### 1-2. リクエスト形式

```
POST /api/v1/evidences
Authorization: Bearer eyJhbGci...
Idempotency-Key: 019682ab-7c1f-7000-g1h2-3i4j5k6l7m8n
Content-Type: multipart/form-data; boundary=--WNavBoundary

----WNavBoundary
Content-Disposition: form-data; name="metadata"
Content-Type: application/json

{
  "work_execution_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
  "step_id": "019682ab-7c1f-7000-0000-000000000602",
  "evidence_type": "photo",
  "description": "溶接部外観確認",
  "timestamp_client": "2026-05-17T08:14:00.123Z",
  "sha256_client": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
}

----WNavBoundary
Content-Disposition: form-data; name="file"; filename="evidence_001.jpg"
Content-Type: image/jpeg

[バイナリデータ]
----WNavBoundary--
```

### 1-3. metadata パート スキーマ

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `work_execution_id` | string (UUID v7) | 必須 | TBL-005 に `in_progress` で存在 | 対象作業実行 ID |
| `step_id` | string (UUID v7) | 必須 | TBL-008 に存在 | 対象ステップ ID |
| `evidence_type` | string | 必須 | `photo` / `document` / `measurement_sheet` | エビデンス種別 |
| `description` | string | 任意 | 最大 500 文字 | 説明 |
| `timestamp_client` | string (ISO 8601) | 必須 | — | クライアント側の撮影・生成時刻 |
| `sha256_client` | string | 必須 | hex 64 文字 | クライアントが計算したファイルの SHA-256 ハッシュ |

### 1-4. file パート 制約

| 制約項目 | 制約値 | 設定 ID |
|---|---|---|
| 最大ファイルサイズ | 20 MB | CFG（wnav.evidence.max_file_size_mb = 20）|
| 許可 MIME type | `image/jpeg` / `image/png` / `image/webp` / `application/pdf` | — |
| 最大解像度（JPEG/PNG/WebP）| 8000 × 8000 px | CFG（wnav.evidence.max_resolution_px = 8000）|
| ファイル名 | 最大 255 文字、英数字・ハイフン・アンダースコア・ドット | — |

### 1-5. サーバー側検証フロー

1. `sha256_client` で宣言されたハッシュとサーバーが受信したファイルの SHA-256 を比較する。不一致の場合: ERR-VAL-003（`evidence.hash_mismatch`）。
2. MIME type・ファイルサイズ・解像度を検証する。違反の場合: ERR-VAL-002 / ERR-VAL-004。
3. ファイルをサーバーのエビデンスストレージ（IIS 配下のファイルシステムまたは共有ストレージ）に保存する。
4. TBL-009（evidence_files）にレコードを挿入する。`file_hash_sha256` に検証済みハッシュを記録する。

### 1-6. レスポンススキーマ（HTTP 201）

```json
{
  "data": {
    "evidence_id": "019682ab-7c1f-7000-f1a2-3b4c5d6e7f8a",
    "file_hash_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "file_path": "/evidences/2026/05/17/019682ab-7c1f-7000-f1a2-3b4c5d6e7f8a.jpg",
    "file_size_bytes": 1234567,
    "evidence_type": "photo",
    "width_px": 4000,
    "height_px": 3000,
    "work_execution_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
    "step_id": "019682ab-7c1f-7000-0000-000000000602",
    "uploaded_by": "019682ab-7c1f-7000-0000-000000000002",
    "uploaded_at": "2026-05-17T08:14:00.000Z"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7020-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T08:14:00.000Z",
    "api_version": "v1"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `evidence_id` | string (UUID v7) | エビデンス ID（TBL-009）|
| `file_hash_sha256` | string | SHA-256 ハッシュ（hex）|
| `file_path` | string | サーバー上の保存パス（相対）|
| `file_size_bytes` | integer | ファイルサイズ（バイト）|
| `width_px` | integer / null | 画像幅（px）画像以外は null |
| `height_px` | integer / null | 画像高さ（px）画像以外は null |
| `uploaded_by` | string (UUID v7) | アップロードしたユーザー ID |
| `uploaded_at` | string (ISO 8601 UTC) | アップロード時刻 |

### 1-7. RBAC

| ロール | アクセス |
|---|---|
| operator | 自工場の実行中 work_execution のみアップロード可 |
| supervisor | 自工場の全 work_execution にアップロード可 |

### 1-8. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-001 | 401 | JWT 無効 |
| ERR-AUTH-004 | 403 | 権限不足 |
| ERR-BIZ-001 | 409 | work_execution が `in_progress` 以外 |
| ERR-VAL-001 | 422 | metadata の必須フィールド不足 |
| ERR-VAL-002 | 422 | ファイルサイズ超過・解像度超過 |
| ERR-VAL-003 | 422 | SHA-256 ハッシュ不一致・MIME type 不正・形式不正 |
| ERR-VAL-004 | 422 | ファイル名が 255 文字超 |

---

## 2. API-electronic-signs-001: POST /api/v1/electronic-signs

### 2-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-electronic-signs-001 |
| HTTP メソッド | POST |
| URL | `/api/v1/electronic-signs` |
| 認証要否 | 必須 |
| Content-Type | `application/json` |
| Idempotency-Key | 必須 |
| レート制限カテゴリ | 書き込み（500 req / 60s）|
| 関連 FR | FR-AU-001 |

### 2-2. リクエストスキーマ

```json
{
  "signer_id": "019682ab-7c1f-7000-0000-000000000002",
  "signed_content_hash": "sha256:abc123def456...",
  "pin_hash": "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/Lew2qx0e.2vF...",
  "context_type": "step_sign",
  "context_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
  "step_id": "019682ab-7c1f-7000-0000-000000000602",
  "timestamp_client": "2026-05-17T08:14:30.123Z",
  "device_signature": "base64:EdDSA署名データ..."
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `signer_id` | string (UUID v7) | 必須 | TBL-016 に存在 | 署名者ユーザー ID |
| `signed_content_hash` | string | 必須 | `sha256:` プレフィックス + hex 64 文字 | 署名対象コンテンツの SHA-256 |
| `pin_hash` | string | 必須 | bcrypt ハッシュ（`$2b$...` または `$2a$...`）| PIN の bcrypt ハッシュ（クライアント側でハッシュ化）|
| `context_type` | string | 必須 | `step_sign` / `work_complete_sign` / `approval_sign` / `quality_check_sign` | 署名コンテキスト種別 |
| `context_id` | string (UUID v7) | 必須 | context_type に対応するレコードが存在 | 署名対象リソース ID |
| `step_id` | string (UUID v7) | 任意 | TBL-008 に存在。context_type が step_sign の場合は必須 | 対象ステップ ID |
| `timestamp_client` | string (ISO 8601) | 必須 | — | クライアント側の署名時刻 |
| `device_signature` | string | 必須 | base64 エンコードされた Ed25519 署名 | 端末秘密鍵（KEY-003〜005）による本文署名 |

### 2-3. PIN 検証フロー

1. TBL-016 の `pin_hash` と受信した `pin_hash` を bcrypt で比較する。
2. 不一致: ERR-AUTH-002（401）。連続 3 回失敗でアカウントロック。
3. `device_signature` を TBL-033 の公開鍵（Ed25519）で検証する。
4. 検証成功: TBL-002（electronic_signs）にレコードを挿入し、TBL-031 にハッシュチェーンブロックを追記する。

### 2-4. レスポンススキーマ（HTTP 201）

```json
{
  "data": {
    "sign_id": "019682ab-7c1f-7000-a1b2-3c4d5e6f7890",
    "signer_id": "019682ab-7c1f-7000-0000-000000000002",
    "signed_content_hash": "sha256:abc123def456...",
    "context_type": "step_sign",
    "context_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
    "signed_at": "2026-05-17T08:14:30.000Z",
    "hash_chain_block_id": "019682ab-7c1f-7000-e1f2-3a4b5c6d7e8f",
    "hash_chain_value": "sha256:ccc333..."
  },
  "meta": {
    "request_id": "019682ab-7c1f-7021-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T08:14:30.000Z",
    "api_version": "v1"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `sign_id` | string (UUID v7) | 電子サイン ID（TBL-002）|
| `signed_at` | string (ISO 8601 UTC) | サーバー側の署名時刻 |
| `hash_chain_block_id` | string (UUID v7) | ハッシュチェーンブロック ID（TBL-031）|
| `hash_chain_value` | string | 今回ブロックのハッシュ値 |

### 2-5. RBAC

`operator` / `supervisor` / `master_admin` / `quality_admin` が使用可。`system_admin` と `executive` は不可。

### 2-6. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-001 | 401 | JWT 無効 |
| ERR-AUTH-002 | 401 | PIN 検証失敗 |
| ERR-AUTH-003 | 423 | アカウントロック（PIN 連続失敗）|
| ERR-AUTH-004 | 403 | system_admin / executive ロールでアクセス |
| ERR-DB-003 | 500 | ハッシュチェーン整合性エラー |
| ERR-VAL-001 | 422 | 必須フィールド不足 |
| ERR-VAL-003 | 422 | device_signature 検証失敗・形式不正 |

---

## 3. API-electronic-signs-002: GET /api/v1/electronic-signs/{id}

### 3-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-electronic-signs-002 |
| HTTP メソッド | GET |
| URL | `/api/v1/electronic-signs/{id}` |
| 認証要否 | 必須 |
| Idempotency-Key | 不要（GET）|
| 関連 FR | FR-AU-003 |

### 3-2. パスパラメータ

| パラメータ | 型 | 必須 | 説明 |
|---|---|---|---|
| `id` | string (UUID v7) | 必須 | 電子サイン ID（TBL-002）|

### 3-3. レスポンススキーマ（HTTP 200）

```json
{
  "data": {
    "sign_id": "019682ab-7c1f-7000-a1b2-3c4d5e6f7890",
    "signer_id": "019682ab-7c1f-7000-0000-000000000002",
    "signer_name": "山田 太郎",
    "signer_role": "operator",
    "signed_content_hash": "sha256:abc123def456...",
    "context_type": "step_sign",
    "context_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
    "step_id": "019682ab-7c1f-7000-0000-000000000602",
    "signed_at": "2026-05-17T08:14:30.000Z",
    "hash_chain_block_id": "019682ab-7c1f-7000-e1f2-3a4b5c6d7e8f",
    "hash_chain_value": "sha256:ccc333...",
    "hash_chain_prev": "sha256:bbb222...",
    "verification_status": "valid",
    "device_id": "019682ab-7c1f-7000-0000-000000000010"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7022-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T10:30:00.000Z",
    "api_version": "v1"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `verification_status` | string | `valid` / `hash_chain_broken` / `device_key_revoked` |
| `hash_chain_prev` | string | 前ブロックのハッシュ値（連鎖性確認用）|

`verification_status: "hash_chain_broken"` の場合、バックエンドは LOG-007（SECURITY レベル）を出力し、MET-006 カウンタをインクリメントする。

### 3-4. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-001 | 401 | JWT 無効 |
| ERR-DB-003 | 500 | ハッシュチェーン検証中に整合性エラーが確認された場合 |
| `404 Not Found` | 404 | id が存在しない |

---

## 4. API-electronic-signs-003: GET /api/v1/electronic-signs

### 4-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-electronic-signs-003 |
| HTTP メソッド | GET |
| URL | `/api/v1/electronic-signs` |
| 認証要否 | 必須 |
| Idempotency-Key | 不要（GET）|
| 関連 FR | FR-AU-003 |

### 4-2. クエリパラメータ

| パラメータ | 型 | 必須 | 説明 |
|---|---|---|---|
| `work_execution_id` | string (UUID v7) | 任意 | 作業実行 ID でフィルタ |
| `signer_id` | string (UUID v7) | 任意 | 署名者 ID でフィルタ |
| `context_type` | string | 任意 | コンテキスト種別でフィルタ |
| `signed_from` | string (ISO 8601) | 任意 | 署名時刻の開始（UTC）|
| `signed_to` | string (ISO 8601) | 任意 | 署名時刻の終了（UTC）|
| `page` | integer | 任意 | ページ番号（デフォルト 1）|
| `per_page` | integer | 任意 | 件数（デフォルト 50 / 最大 200）|

### 4-3. RBAC

`quality_admin` / `system_admin` のみ一覧取得可。`operator` は自身の署名のみ `GET /electronic-signs?signer_id={self}` で参照可。

---

**本節で確定した方針**
- **API-evidences-001 はファイルアップロードとメタデータを multipart で受け取り、クライアント側 SHA-256 とサーバー側の照合を必須とし、不一致の場合は ERR-VAL-003 で拒否することを確定した。**
- **API-electronic-signs-001 は PIN の bcrypt 検証と Ed25519 デバイス署名の両方を合格して初めてサイン記録を許可し、TBL-031 ハッシュチェーンにブロックを追記することを確定した。**
- **電子サイン取得（API-electronic-signs-002）は `verification_status` フィールドでハッシュチェーンの健全性を返し、`hash_chain_broken` を検出した場合は LOG-007 / MET-006 をトリガーすることを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
