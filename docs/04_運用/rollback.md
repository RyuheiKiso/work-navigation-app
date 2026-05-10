# リリースロールバック手順

> 対応 §: ロードマップ §32.5 §22.2 §22.4 §15 §10.3.1
> 対象読者: メンテナ、導入企業のリリース担当
> 改訂サイクル: §22.1 半期サイクル

§32.5 受入観点「直前安定版へ戻す手順を `docs/04_運用/rollback.md` に常備」を満たすため本書を版管理する。サーバ側 DB マイグレーションは **必ず可逆 or 二段階**（add then remove）で行う。

## 1. ロールバックの種類

| 種類 | 対象 | 主な手段 |
| --- | --- | --- |
| アプリケーションロールバック | コンテナ／APK／MSI | 直前安定版イメージへ切替 |
| DB マイグレーションロールバック | PostgreSQL／SQLite スキーマ | down マイグレーション or 二段階削除のロール |
| アドオンロールバック | `*.wnaddon` | 設定 UI からアドオン無効化／古い版に戻す |
| 設定（フロー定義）ロールバック | 業務フロー | §10.2 ロールバック導線（操作 3 ステップ以内） |

## 2. 共通前提

- リリースアーティファクトには署名（OIDC）が付与されており、ロールバック対象も署名検証を経ること（§19.3）。
- 直前安定版のコンテナイメージは GHCR／OCI レジストリに保持されること。
- 本書の手順は **リハーサル済み** であること（§22.5／§14.4 受入観点）。

## 3. アプリケーションロールバック

### 3.1. サーバ（Docker Compose）

```bash
# 1. 現行版を停止
docker compose -f /opt/wna/docker-compose.yml down

# 2. 直前安定版（例: v1.2.3）に env を切り替え
sed -i 's/IMAGE_TAG=.*/IMAGE_TAG=v1.2.3/' /opt/wna/.env

# 3. 起動
docker compose -f /opt/wna/docker-compose.yml up -d

# 4. ヘルスチェック
curl -fsSL http://localhost/healthz
```

### 3.2. 端末（Android APK）

```bash
# 1. APK をリポジトリ（信頼ネットワーク内）から取得し署名検証
apksigner verify --print-certs work-navigation-app-v1.2.3.apk

# 2. 端末へ配布（MDM 経由を推奨）
adb install -r work-navigation-app-v1.2.3.apk
```

### 3.3. 端末（Windows MSI）

```powershell
# 1. MSI 署名検証
Get-AuthenticodeSignature .\work-navigation-app-v1.2.3.msi

# 2. インストール
msiexec /i work-navigation-app-v1.2.3.msi /qn
```

## 4. DB マイグレーションロールバック

### 4.1. 二段階削除原則

破壊的なスキーマ変更（カラム削除・名称変更）は **add then remove** で 2 リリースに分割する（§32.5）。

```
Release N:    新カラム追加（後方互換）／旧カラム維持
Release N+1:  旧カラム使用箇所を新カラムへ移行
Release N+2:  旧カラム削除（事前予告済）
```

### 4.2. 可逆マイグレーション

可逆なマイグレーションは `down` を必ず用意する（sqlx／diesel／sea-orm の標準機能）。

```sql
-- migrations/<タイムスタンプ>_add_lwww_clock.up.sql
ALTER TABLE user_settings ADD COLUMN lamport_clock BIGINT NOT NULL DEFAULT 0;

-- migrations/<タイムスタンプ>_add_lwww_clock.down.sql
ALTER TABLE user_settings DROP COLUMN lamport_clock;
```

ロールバックコマンド:

```bash
sqlx migrate revert --database-url "$DATABASE_URL"
```

### 4.3. 端末 SQLite

- WAL モードで運用しているため、`*.wal`／`*.shm` を含めて整合性を保つ（§11.6 受入観点）。
- スキーマダウングレードは端末アプリ自身が起動時に判定し、不可逆変更検出時は **業務操作を拒否し管理者通知** する（§10.5.1 と同方針）。

## 5. アドオンロールバック

```
1. 設定 Web UI → アドオン管理画面
2. 該当アドオンを無効化（capability 即時剥奪）
3. 旧バージョンの `*.wnaddon` を選択し再インストール
4. 監査ログに「アドオンロールバック」行が追記されることを確認
```

## 6. 設定（フロー定義）ロールバック

§10.2 で 3 ステップ以内の操作を保証する。

```
1. フロー編集画面 → バージョン履歴
2. 戻したい版を選択
3. 「この版に戻す」確認モーダルで確定
   → ロールバック適用前の影響範囲（参照する作業件数）が表示される
```

## 7. 緊急ロールバック（CVSS ≥ 7.0 のホットフィックス時）

§32.2 ホットフィックスの逆方向。

```
1. インシデント Issue を起票（ラベル `regression`）
2. メンテナ承認（24 時間以内）
3. 上記 3.1〜3.3 を実行
4. ポストモーテムを `docs/04_運用/postmortem-<YYYY-MM-DD>.md` に記録
5. §22.4 是正フローへ
```

## 8. リハーサル記録

| 日付 | 種類 | 結果 | 所要時間 | 備考 |
| --- | --- | --- | --- | --- |
| TBD | アプリ（サーバ） | TBD | — | 公開前にリハーサル必須 |

## 9. 受入観点（§32.5／§14.4）

- 本書の手順がリハーサル済みであること（コード初期実装後・公開前）。
- DB マイグレーションが可逆 or 二段階であることが CI で検査されること（CI 整備後）。
- ロールバック失敗時のエラーコードと推奨対処が `make doctor` 出力に含まれること（§14.2 整合）。
- §22.5 §14.4 受入観点との整合。
- 緊急ロールバックがホットフィックス SLA（72 時間以内）と整合（§32.2／`SECURITY.md`）。
