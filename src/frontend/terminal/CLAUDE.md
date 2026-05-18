# src/frontend/terminal — ハンディ APP 実装規約

React Native + Expo で構築する現場作業者向けタブレットアプリの実装規約。
横断共通原則は `src/CLAUDE.md`、フロントエンド共通規約は `src/frontend/CLAUDE.md` が権威。
本ファイルはハンディ APP 固有の規約を記す。

権威ドキュメント: `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` 第 2 節

---

## 技術スタック

| 要素 | 採用 |
|---|---|
| OS 対応 | Android / iOS / Windows タブレット（単一コードベース） |
| フレームワーク | React Native + Expo（react-native-windows で 3 OS 統合） |
| 言語 | TypeScript（strict mode） |
| ローカル DB | SQLite + TypeORM |
| カメラ / QR スキャン | react-native-camera + ML Kit |
| 電子署名 | Android: Keystore / iOS: Keychain / Windows: DPAPI |

---

## Offline-First 実装規約

権威ドキュメント: `docs/02_企画/システム化計画/05_アーキテクチャ原則.md` 第 1 節

### ネットワーク 4 段階制御

ネットワーク状態の判定・遷移ロジックは `network/` に集約する。
他のモジュールはこの層のインターフェースを通してのみ状態を参照する。

```typescript
type NetworkQuality = 'high' | 'low' | 'disconnected' | 'emergency';
```

**Emergency Mode 発動**: 切断が設定時間（デフォルト 5 分）を超過した場合

**Emergency Mode の必須 UI 要素**:
1. フルスクリーンバナー（他コンテンツより前面に表示）
2. 「最終同期時刻」の常時表示（UTC + ローカルタイムゾーン）
3. キャッシュ済み SOP がない場合は操作を防止し、その旨を明示する

### Offline-First が保証する機能（接続不要）

- 手順ナビゲーション（SOP 閲覧・Step 進行・記録入力）
- ローカルアラート（入力値範囲逸脱・必須項目未入力）
- 電子署名（ローカル鍵による署名）
- カメラ・QR スキャン記録

### 接続時のみ有効な機能

- SOP マスタ最新版取得（差分ダウンロード）
- マスタ新規追加・変更（`master/` アプリ経由）

---

## Outbox Pattern によるサーバー同期

権威ドキュメント: `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` 第 5 節

```
作業記録入力
  → local work_events テーブルに INSERT（イベント本体）
  → local outbox テーブルに INSERT（送信キュー）
  → バックグラウンドワーカーが created_at 昇順で順次送信
  → ACK 受領後に outbox レコードのみ削除（work_events は削除しない）
```

実装規約:
1. **Idempotency-Key**: UUID v4 でクライアントが採番し、outbox レコードに保存する
2. **SHA-256 ハッシュ**: 送信前に前イベントのハッシュを含めてチェーンを構成する
3. **送信順序保証**: `created_at` 昇順で必ず順次送信する（並列送信禁止）
4. **失敗リトライ**: 指数バックオフ（1s → 2s → 4s → ... 上限 5 分）
5. **ACK 後の work_events 削除禁止**: イベント本体は端末に永続的に残す

---

## Case 占有とシフト交代

権威ドキュメント: `docs/05_詳細設計/07_アルゴリズム詳細設計/08_Case端末占有アルゴリズム.md`（ALG-026〜028 / ADR-009）

**1 case_id = 1 端末**。以下のシーケンスを遵守する:

1. **占有獲得**: POST /work-executions または POST /resume で case_locks に原子登録。失敗時は ERR-BIZ-026（HTTP 409）を受け取り操作を中断する
2. **ハートビート**: 60 秒ごとに PUT /work-executions/{id}/heartbeat を送信する。送信失敗はローカルキューに積み、次回接続時に再送する
3. **正常解放**: suspend または complete の完了後に case_lock が解放される
4. **シフト交代**: 現端末で suspend → 解放確認 → 次端末で resume の順序で行う

**オフライン中の挙動**:
- オフライン中は **新規** case_id の占有を試みない（接続が必要）
- 既に占有済みの case_id での Step 記録はオフラインでも継続できる
- 5 分切断後、BAT-013 が自動 EXPIRED 化する（Emergency Mode 閾値と一致）

ハッシュチェーン破断時のクライアント resume 判定: `docs/04_概要設計/02_ソフトウェア方式設計/13a_ハッシュチェーン破断時のクライアント resume 判定.md`

---

## SQLite + TypeORM 規約

**スキーマ同期権威ドキュメント**: `docs/05_詳細設計/01_データベース詳細設計/07a_PG_SQLiteスキーマ同期戦略.md`（ADR-006）
**マイグレーション権威ドキュメント**: `docs/05_詳細設計/04_ハンディAPP詳細設計/07a_TypeORMマイグレーション設計.md`

### スキーマ設計原則

- **論理対応**: SQLite スキーマは PostgreSQL と論理的に対応を保つ（カラム名・型命名を統一する）
- **ミラー対象テーブル**: PG ↔ SQLite 同期対象テーブルは `07a_PG_SQLiteスキーマ同期戦略.md` で列挙する。リストにないテーブルは PG-only とみなす
- **UPDATE / DELETE 禁止**: 作業ログテーブルへの UPDATE / DELETE はアプリケーション層でも禁止。TypeORM のエンティティで `update` / `delete` メソッドを呼ばない
- **マイグレーション**: `07a_TypeORMマイグレーション設計.md` に従い TypeORM Migration で版管理する。命名: `{ts_ms}-{description}.ts`。ロールバックは前進修正のみ（端末では `migration:revert` 非サポート）

### ディレクトリ構成（`db/` 配下）

```
db/
  entities/       # TypeORM エンティティ定義
  migrations/     # タイムスタンプ付きマイグレーションファイル
  data-source.ts  # DataSource 設定（SQLite パス・エンティティ一覧）
```

### 型整合性

SQLite の型システムは PostgreSQL より弱い。TypeORM の型定義で補完し、サーバーへの送信時に再バリデーションを実施する。
数値は `number` 型で保持し、SQLite の NUMERIC アフィニティの暗黙変換に依存しない。

---

## 製造現場 UI/UX 規約

権威ドキュメント: `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` 第 2 節、`docs/02_企画/システム化計画/08_品質特性と非機能要件方針.md` 第 2 節

### Glanceable 設計（最重要 UI 原則）

現場作業員は画面を注視し続ける時間的余裕を持たない。「現在ステップ・異常アラート・次ステップ」の 3 要素を 1 秒以内に視認できる設計を最優先とする（Endsley SA モデル レベル 1）。

| 操作 | 応答時間上限 | 前提 |
|---|---|---|
| 現在ステップ表示 | 200ms 以内 | ローカル SQLite クエリ |
| ページ遷移 | 500ms 以内 | ローカル SQLite クエリ |
| バーコード / QR 認識 | 1 秒以内 | ML Kit オフライン推論 |

### タッチターゲット

- 最小 **72dp**（手袋着用前提）。Material Design 推奨の 48dp では不十分
- インタラクティブ要素間の最小マージン: 8dp

### 環境適応

- **夜間モード**: 輝度最小でも WCAG AA 以上のコントラスト比を保証する
- **騒音環境**: アラートはバイブレーションを必ず併用する（音声のみに依存しない）
- **高齢者対応**: ベースフォントサイズを可変にし、1.5 倍まで拡大しても UI が崩れない実装にする

---

## 電子署名

- **鍵の保管**: Android → Keystore / iOS → Keychain / Windows → DPAPI（ネイティブセキュアストレージ）
- **オフライン署名**: ネットワーク接続なしに署名可能
- **署名アルゴリズム**: Ed25519 または ECDSA P-256（要件定義フェーズで ADR として確定）
- 署名結果はイベントの `Attributes` に格納し、サーバー側で検証する

---

## Step エンジン

権威ドキュメント: `docs/02_企画/システム化計画/18_拡張可能Stepエンジン（アドオン機構）.md`

- Step の条件分岐・バリデーションロジックは **JSON Logic** で宣言的に表現する
- `eval()` / `new Function()` による動的コード実行を禁止する
- Step エンジンのコアロジックは `domain/step-engine/` に集約する
- `fallback_type` による縮退動作を必ず実装する（エンジン障害時にフルストップしない）

---

## テスト

| テスト種別 | ツール | 必須シナリオ |
|---|---|---|
| ユニット | Jest + Testing Library (RN) | Step エンジン・ハッシュチェーン・Outbox 状態管理 |
| 統合 E2E | Detox | 作業フロー（Step 開始→完了→Outbox 送信）の実機確認 |
| オフラインシナリオ | Jest + モックネットワーク | 切断状態での記録入力・Emergency Mode 遷移・Outbox 積み |

オフライン挙動テストは必須。「ネットワークが切れた状態でも主要操作が完遂できること」を自動テストで証明する。

---

## Expo OTA 運用

| 変更種別 | OTA 配信 | ストア審査 |
|---|---|---|
| JavaScript / TypeScript コード変更 | ○ | |
| UI・ビジネスロジック変更 | ○ | |
| ネイティブモジュールの追加・変更 | | ○ |
| Expo SDK メジャーアップグレード | | ○ |
| `app.json` / `app.config.js` の変更 | | ○（影響による） |

---

## ディレクトリ構成案（暫定）

```
src/frontend/terminal/
  app/          # 画面・ルーティング（Expo Router）
  network/      # ネットワーク 4 段階制御・Outbox ワーカー
  db/           # TypeORM エンティティ・マイグレーション・DataSource
  domain/       # Step エンジン・イベント生成・ハッシュチェーン
  ui/           # 共通 UI コンポーネント（72dp ボタン・バナー等）
  i18n/         # react-i18next 設定・JSONB ローカライズ
  crypto/       # SHA-256 ハッシュ・電子署名
  auth/         # JWT 管理・ローカルセッション
```

実装開始時に整合性を確認し、変更する場合は ADR に記録する。

---

## 参照ドキュメント

- `docs/02_企画/システム化計画/05_アーキテクチャ原則.md` — Offline-First / Append-only / Outbox / ハッシュチェーン
- `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` — React Native 選定根拠・却下された代替案
- `docs/02_企画/システム化計画/08_品質特性と非機能要件方針.md` — Glanceable 200ms / 製造現場 UX
- `docs/02_企画/システム化計画/18_拡張可能Stepエンジン（アドオン機構）.md` — Step エンジン設計
- `src/CLAUDE.md` — 横断共通原則
- `src/frontend/CLAUDE.md` — フロントエンド共通規約

---

最終更新: 2026-05-17
次回見直しトリガー: React Native 実装開始時、または Expo SDK バージョン確定時
