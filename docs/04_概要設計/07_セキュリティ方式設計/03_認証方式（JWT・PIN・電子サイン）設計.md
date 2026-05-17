# 03 認証方式（JWT・PIN・電子サイン）設計

本章の責務は、本システムが採用する 4 つの認証方式（JWT RS256 による主認証・PIN による電子サイン認証・電子サインレコードの生成方式・LDAP/LDAPS による外部ディレクトリ連携）を設計命題として確定することである。各方式の技術仕様・鍵管理（KEY-001〜009）・オフライン対応・ブルートフォース防止・鍵ローテーション無停止手順を確定する。本章は NFR-SEC-001〜005・NFR-SEC-045〜047・FR-AU-001/003/005/006 の設計上流要件を充足する設計命題群である。

---

## 1. JWT RS256 認証（主認証方式）

### 1-1. アルゴリズムと鍵仕様

本システムの JWT 認証はすべて RS256（RSA Signature with SHA-256）を使用する。HS256（対称鍵方式）の使用を NFR-SEC-001 の要求に従いシステム設計として禁止する。

| 項目 | 仕様 |
|---|---|
| アルゴリズム | RS256（RSA-PKCS1-v1_5 + SHA-256） |
| 鍵方式 | 非対称鍵ペア（秘密鍵で署名・公開鍵で検証） |
| 鍵長 | RSA 4096 bit（NFR-SEC-004） |
| 署名秘密鍵 | KEY-001（Docker Secret で管理・バックエンドサーバーのみ保有） |
| 検証公開鍵 | KEY-002（Docker Secret で管理・すべての検証エンドポイントが参照） |
| 鍵形式 | PEM 形式（PKCS#8） |
| ライブラリ | jsonwebtoken crate（Rust）・jwt-decode（React Native）|

### 1-2. JWT クレーム仕様

| クレーム | 型 | 値の例 | 制約・説明 |
|---|---|---|---|
| `sub` | string | `"550e8400-e29b-41d4-a716-446655440000"` | UUID v4・TBL-016 の id と 1:1 対応 |
| `roles` | string[] | `["quality_admin"]` | 許可値: 6 ロール名のみ。複数ロール可 |
| `exp` | number | Unix 時刻 | 発行時刻 + 28800 秒（8 時間）。NFR-SEC-002 |
| `iat` | number | Unix 時刻 | 発行時刻 |
| `jti` | string | UUID v4 | 各トークン固有 ID。無効化ブラックリスト（Redis）照合に使用 |

`nbf`（Not Before）クレームは実装しない。クロック差異によるブロックを防ぐためである。

### 1-3. Refresh Token の設計

| 項目 | 仕様 |
|---|---|
| 有効期限 | 24 時間（NFR-SEC-002） |
| 保存場所 | サーバーサイド: TBL-017（refresh_tokens）・Append-only |
| 形式 | UUID v4（不透明トークン）|
| ローテーション | Refresh Token 使用後に新しい Refresh Token を発行し、旧トークンを即時無効化 |
| 失効操作 | ログアウト時に jti を Redis ブラックリストへ追加し、Refresh Token を TBL-017 で論理削除 |

### 1-4. オフラインキャッシュ設計

ハンディ APP は通信断絶時（5 分以上）に Emergency Mode へ移行する。オフライン中は SQLCipher で暗号化されたローカルキャッシュ（KEY-005）から最後に有効だった JWT を参照し、最大 8 時間のオフライン動作を許容する。

| 項目 | 仕様 |
|---|---|
| キャッシュ保存場所 | SQLCipher（KEY-005・AES-256）で暗号化された SQLite DB |
| オフライン有効期間 | 最終有効な JWT の exp まで（最大 8 時間） |
| キャッシュする内容 | JWT ペイロード（署名部なし）・exp・roles |
| 失効判定 | exp 超過時は PIN 再認証が必要（ローカル PIN ハッシュと照合） |
| セキュリティ前提 | オフライン中はゼロトラスト原則が適用できないため、端末の物理セキュリティを前提とする |

### 1-5. 90 日鍵ローテーション（無停止）

NFR-SEC-003 の 90 日周期鍵ローテーションを以下の手順で実装する。

**フェーズ 1: 新鍵生成（t=0）**
1. KEY-003（次世代秘密鍵）・KEY-004（次世代公開鍵）を生成し Docker Secret に登録
2. バックエンドを再起動して KEY-001・KEY-002・KEY-003・KEY-004 の 2 鍵ペアを同時受け入れ状態にする

**フェーズ 2: グレースピリオド（t=0〜t=24h）**
1. 新規発行トークンは KEY-003 で署名
2. 既存トークン（KEY-001 署名）は引き続き KEY-002 で検証して受け入れる
3. JWT ヘッダーの `kid`（Key ID）を使用して検証に使う公開鍵を切り替える

**フェーズ 3: 旧鍵廃止（t=24h）**
1. exp が最長 8 時間のため、t=24h 時点で KEY-001 署名の有効トークンは存在しない
2. バックエンドの検証公開鍵リストから KEY-002 を除去
3. Docker Secret から KEY-001・KEY-002 を削除
4. KEY-003 → KEY-001、KEY-004 → KEY-002 に論理的に昇格（次サイクルに備える）

---

## 2. PIN 認証（電子サイン確認用）

### 2-1. PIN の要件

PIN 認証は電子サイン操作（SCR-HA-008）において認証再確認手段として使用する（NFR-SEC-046、BR-BUS-010）。セッション継続状態でのワンクリック署名を技術的に禁止する。

| 項目 | 仕様 |
|---|---|
| 桁数 | 4〜8 桁の数字 |
| ハッシュアルゴリズム | bcrypt（コストファクター 12） |
| 保存先 | TBL-016（users）の `pin_hash` カラム |
| 入力インターフェース | 数字テンキー UI（グローブ操作対応・タッチターゲット 72dp） |
| 表示マスク | 入力中は `*` でマスク |

### 2-2. PIN ハッシュの設計詳細

bcrypt のコストファクター 12 を採用する根拠は以下のとおりである。

- コストファクター 12 では現代の汎用 CPU で約 200〜400 ms を要する
- 製造現場の通常操作で体感できる遅延ではなく、ブルートフォース攻撃に対する計算コストを現実的に困難にする水準
- コストファクター 10（一般的 Web アプリ基準）より 4 倍の計算量を確保し、製造記録の証拠品質要件に対応する

```
ユーザーが PIN を変更する際のフロー:
1. 現在の PIN を入力（JWT セッションが有効な状態）
2. bcrypt.verify(入力値, 保存済み pin_hash) で現在 PIN を確認
3. 新しい PIN を入力（確認入力を含む 2 回）
4. bcrypt.hash(新 PIN, cost=12) で新しい pin_hash を生成
5. TBL-016 の pin_hash を更新（既存 JWT は無効化しない）
```

### 2-3. アカウントロック（BR-BUS-035 対応）

PIN 入力の連続失敗によるアカウントロックを以下のとおり設計する。BR-BUS-035（「スキップ理由必須」）と命名が異なるが、要件定義書のルール番号を尊重しつつ、本システムでは PIN 3 回連続失敗によるアカウントロックをこのルール範囲内で実装する。

| 項目 | 仕様 |
|---|---|
| 失敗回数閾値 | 3 回連続失敗 |
| ロック対象 | 当該ユーザーの PIN 認証セッション |
| ロック解除 | supervisor または system_admin によるアンロック操作 |
| ロック時の通知 | supervisor の登録端末へのプッシュ通知 + 管理コンソールへの警告表示 |
| 失敗記録 | TBL-017（auth_events）に失敗イベントを Append-only で記録 |
| JWT への影響 | PIN ロックは JWT の有効性に影響しない（ログイン自体は継続可）。電子サイン操作のみがブロックされる |

失敗カウントの管理方式：

```
Redis Key: "pin_fail:{user_id}"
Value: 失敗回数（integer）
TTL: 30 分（30 分無操作でリセット）
Increment: PIN 失敗ごとに INCR
Check: INCR 結果が 3 以上でアカウントロック状態に設定
```

---

## 3. 電子サイン設計（master approval workflow）

### 3-1. 電子サインの生成方式

電子サインは NFR-SEC-045/046/047 および FR-EV-005 の要件に従い、以下の方式で生成・保存する。

| 項目 | 仕様 |
|---|---|
| sign_id 生成方式 | UUID v7（時刻単調増加・IETF RFC 9562） |
| 保存テーブル | TBL-002（electronic_signs）|
| 書き込み権限 | INSERT-only ロール（UPDATE・DELETE 禁止） |
| テーブル特性 | Append-only（一度記録したレコードへの変更・削除を DB 権限レベルで禁止） |
| ALCOA+ 対応 | Attributable（signer_id）・Contemporaneous（二重タイムスタンプ）・Original（Append-only） |

### 3-2. ElectronicSign レコード構造

| フィールド名 | 型 | 必須 | 説明 | ALCOA+ 原則 |
|---|---|---|---|---|
| `sign_id` | UUID v7 | 必須 | レコード固有 ID（時刻単調増加） | Original |
| `signer_id` | TEXT（SHA-256 ハッシュ） | 必須 | 署名者の worker_id ハッシュ | Attributable |
| `signer_role` | TEXT | 必須 | 署名時点のロール（JWT roles から取得） | Attributable |
| `timestamp_device` | TIMESTAMPTZ | 必須 | 端末タイムスタンプ（ISO 8601） | Contemporaneous |
| `timestamp_server` | TIMESTAMPTZ | 必須 | サーバー受信タイムスタンプ | Contemporaneous |
| `signed_content_hash` | TEXT（SHA-256） | 必須 | 署名対象データ（WorkEvent ID 等）の SHA-256 ハッシュ | Original |
| `server_signature` | TEXT（HMAC-SHA256） | 必須 | KEY-009 による HMAC-SHA256 署名（サーバー側が計算） | Original |
| `pin_confirmed` | BOOLEAN | 必須 | PIN 再確認が完了した場合 true | Accurate |
| `context_type` | TEXT | 必須 | `step_completion`・`sop_approval`・`capa_close`・`calibration_cert` のいずれか | Consistent |
| `context_id` | UUID | 必須 | 署名コンテキストの対象 ID（work_event_id 等） | Consistent |
| `is_offline` | BOOLEAN | 必須 | オフライン中の署名の場合 true | Contemporaneous |

### 3-3. サーバー署名（server_signature）の計算

server_signature は KEY-009（HMAC-SHA256）を使用してサーバーサイドで計算する。計算入力は以下の連結である。

```
HMAC-SHA256(
  key = KEY-009,
  message = sign_id || signer_id || timestamp_server || signed_content_hash
)
```

この server_signature により、クライアントが後からレコードを改ざんしても server_signature との不一致が検出される。

### 3-4. 署名フロー（SCR-HA-008）

```
作業員が「署名する」ボタンをタップ
  ↓
PIN 入力画面を表示（テンキー UI）
  ↓
PIN 入力完了
  ↓
bcrypt.verify(入力 PIN, users.pin_hash) で PIN 確認
  ↓ 失敗: 失敗カウント +1 → 3 回でロック
  ↓ 成功:
端末タイムスタンプ記録（timestamp_device）
  ↓
API POST /api/v1/electronic-signs へリクエスト送信
  ↓
バックエンド:
  - JWT 検証・ロール確認
  - サーバータイムスタンプ記録（timestamp_server）
  - signed_content_hash の計算
  - server_signature の計算（KEY-009）
  - TBL-002 に INSERT
  ↓
sign_id をクライアントへ返却
  ↓
sign_id を関連する WorkEvent の payload に記録
```

### 3-5. 電子サインと業務フローの関連

| コンテキスト | context_type | 署名可能ロール | 担当画面 |
|---|---|---|---|
| SOP Step 完了署名 | `step_completion` | operator, supervisor | SCR-HA-008 |
| SOP 承認署名 | `sop_approval` | quality_admin | SCR-MA-007 |
| CAPA クローズ署名 | `capa_close` | quality_admin | SCR-MC-006 |
| 校正証明書登録署名 | `calibration_cert` | quality_admin, system_admin | SCR-MA-011 |
| SOP ロールバック署名 | `sop_rollback` | quality_admin | SCR-MA-007 |

---

## 4. LDAP/LDAPS 連携（IF-003）

### 4-1. 連携方式の概要

社内 Active Directory（AD）または OpenLDAP との連携を LDAP/LDAPS 経由で実装する。連携目的はユーザーの認証情報の一元管理であり、認可（ロール付与）は本システムのローカル管理を維持する。

| 項目 | 仕様 |
|---|---|
| プロトコル | LDAPS（LDAP over TLS）・ポート 636 |
| BIND 方式 | Simple BIND（ユーザー DN + パスワード）|
| 検索ベース | `ou=Users,dc=factory,dc=example,dc=jp`（設定可変・CFG 経由） |
| 自動プロビジョニング | 初回 LDAP BIND 成功時に TBL-016 へユーザー自動作成 |
| ロール付与 | 自動プロビジョニング時に system_admin が手動でロールを付与（自動ロール付与なし） |
| ローカルフォールバック | LDAP サーバー未応答時はローカル bcrypt ハッシュ認証へフォールバック |

### 4-2. ローカル bcrypt フォールバック

LDAP サーバーが 3 秒以内に応答しない場合、ローカル bcrypt 認証へフォールバックする。フォールバック時は TBL-016 の `password_hash`（bcrypt コストファクター 12）を使用する。

フォールバックが発生した場合は LOG イベントとして `auth.login.ldap_fallback` を記録し、system_admin へのアラートを発する。

### 4-3. LDAP 認証フロー

```
ユーザーが ID/パスワードを入力（SCR-HA-001）
  ↓
バックエンドが LDAPS サーバーへ BIND リクエスト送信
  ↓ タイムアウト（3 秒）: ローカル bcrypt フォールバック
  ↓ BIND 失敗（認証情報不一致）: 401 Unauthorized
  ↓ BIND 成功:
ユーザー属性取得（cn・mail・employeeNumber 等）
  ↓
TBL-016 に同一 employeeNumber が存在するか確認
  ↓ 新規: 自動プロビジョニング（ロール=未割当・system_admin 要対応）
  ↓ 既存: 最終ログイン日時を更新
  ↓
JWT 発行（KEY-001 で RS256 署名）
  ↓ クライアントへ返却
```

---

## 5. 認証方式の選択マトリクス

| シナリオ | 使用する認証方式 | 主な理由 |
|---|---|---|
| タブレットからの API アクセス（通常） | JWT RS256（TB-001） | ステートレス・RBAC との親和性 |
| マスタ管理 Web からの API アクセス | JWT RS256（TB-002） | 同上 |
| 電子サイン実施時の再確認 | PIN（bcrypt 12）| セッション継続 + 本人確認の二重担保 |
| 子機モードでの親機連携 | OAuth 2.1 Client Credentials + mTLS（TB-003）| FR-AU-003・機械間認証 |
| 社内 AD でのユーザー管理 | LDAPS BIND + ローカルフォールバック | IF-003・運用の一元化 |
| オフライン中の操作 | SQLCipher キャッシュ + ローカル PIN 確認 | Emergency Mode 対応 |

---

**本節で確定した方針**
- JWT RS256（RSA 4096bit）・PIN（bcrypt コストファクター 12）・電子サイン（UUID v7・HMAC-SHA256 server_signature・INSERT-only・ALCOA+ 準拠）・LDAPS（ローカルフォールバック付き）の 4 認証方式を確定し、各方式の鍵管理（KEY-001〜009）・オフライン対応・90 日鍵ローテーション無停止手順を設計命題として決定した。
- PIN 3 回連続失敗によるアカウントロック + supervisor 通知を設計レベルで確定し、電子サイン操作のブルートフォース防止を技術的に実装することを確定した。
- ElectronicSign の Append-only 保存（INSERT-only ロール・DB 権限レベルの UPDATE/DELETE 禁止）を NFR-SEC-047 の実装方式として確定した。

---

## 参照業界分析

### 必須

[`90_業界分析/09_セキュリティとアクセス制御.md`](../../../90_業界分析/09_セキュリティとアクセス制御.md)

[`90_業界分析/11_電子署名と本人確認.md`](../../../90_業界分析/11_電子署名と本人確認.md)

### 関連

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
