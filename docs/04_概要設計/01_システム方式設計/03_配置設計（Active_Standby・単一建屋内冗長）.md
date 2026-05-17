# 03 配置設計（Active-Standby・単一建屋内冗長）

本章は、IPA 共通フレーム 2013「2.3.4 配置設計タスク」および「2.3.6 可用性設計タスク」に対応し、単一建屋内における Active-Standby 構成・Docker Compose サービス配置・PostgreSQL ストリーミングレプリケーション・IIS リバースプロキシの配置を確定する。NFR-AVL-001〜010 の充足手段を設計として具体化する。

---

## 1. 配置前提と可用性設計方針

### 1-1. 可用性設計の基本方針

| 方針 ID | 内容 | 根拠 |
|---|---|---|
| POL-AVAIL-001 | 単一建屋内での Active-Standby（1+1 冗長）を採用する | NFR-AVL-001（99.5% 稼働率）・個人開発・IT 担当 1 名の制約 |
| POL-AVAIL-002 | フェイルオーバーは手動とする（自動フェイルオーバーは採用しない） | NFR-AVL-002（RTO 1 時間は手動切替で達成可能）・ADR-SYS-002 |
| POL-AVAIL-003 | Offline-First（SUB-001）により、サーバー障害時も作業継続を保証する | NFR-AVL-004・ADR-SYS-001 |
| POL-AVAIL-004 | Docker コンテナの異常終了は自動再起動ポリシー（`restart: unless-stopped`）で自動復旧する | NFR-AVL-006 |
| POL-AVAIL-005 | 地理的冗長（マルチサイト）は採用しない | P01（単一工場）・IT 担当 1 名制約 |

図: fig_des_deploy_active_standby（img/ 配下）を参照

### 1-2. RTO/RPO の充足手段マッピング

| NFR ID | 目標値 | 充足手段 |
|---|---|---|
| NFR-AVL-001 | 稼働率 99.5%（営業時間ベース）| Active-Standby + Docker 自動再起動 + Offline-First 縮退 |
| NFR-AVL-002 | RTO 1 時間 / RPO 15 分 | 手動切替 RTO 1 時間・WAL PITR 5 分間隔で RPO 15 分 |
| NFR-AVL-003 | 計画停止は夜間・月 1 回・2 時間以内 | メンテナンスウィンドウ定義（本章 §3 参照） |
| NFR-AVL-005 | PostgreSQL ストリーミングレプリケーション | NODE-001 → NODE-002 への非同期レプリケーション |
| NFR-AVL-006 | Docker 自動再起動ポリシー | `restart: unless-stopped`（全サービス共通） |
| NFR-AVL-009 | バックアップ 3 層（WAL/日次/週次）| BAT-002（WAL 転送）・BAT-003（pg_dump）・BAT-004（週次 AES-256） |

---

## 2. 物理ノード配置

### 2-1. ノード一覧

| NODE ID | 名称 | 役割 | ハードウェア仕様 |
|---|---|---|---|
| NODE-001 | Active サーバー | 通常稼働の主系。WSL2 + Docker Compose を稼働させる | §02 のサーバー最低仕様と同等 |
| NODE-002 | Standby サーバー | 待機系（ウォームスタンバイ）。PostgreSQL レプリカを稼働させる | NODE-001 と同等仕様 |
| NODE-003 | NAS | WAL アーカイブ・pg_dump・週次スナップショット保管 | 4TB 以上・Gigabit Ethernet |
| NODE-004 | Wi-Fi AP | 工場 LAN ゾーンの無線 LAN 提供（複数台配置） | 802.11ac/ax 対応（2.4GHz/5GHz） |
| NODE-005 | L2/L3 スイッチ | 管理 LAN ゾーンの有線接続・VLAN 分割 | Gigabit Ethernet 対応 |

### 2-2. ソフトウェア配置（NODE-001: Active サーバー）

NODE-001 上で稼働するソフトウェアスタックを層別に示す。

| 層 | ソフトウェア | 配置場所 |
|---|---|---|
| OS 層 | Windows Server 2022 LTSC（ENV-ITEM-001）| 物理 OS |
| Web サーバー層 | IIS 10（ENV-ITEM-005）| Windows 上（直接インストール）|
| 仮想化層 | WSL2（ENV-ITEM-002）| Windows 上（機能として有効化） |
| コンテナランタイム | Docker Engine 26.x（ENV-ITEM-003）| WSL2 ディストリビューション上 |
| オーケストレーション | Docker Compose v2（ENV-ITEM-004）| Docker Engine 上 |
| API コンテナ | `wnav_api`（Rust axum）| Docker Compose サービス |
| DB コンテナ | `postgres`（PostgreSQL 16）| Docker Compose サービス |
| スケジューラコンテナ | `wnav_scheduler`（BAT ジョブ）| Docker Compose サービス |

### 2-3. ソフトウェア配置（NODE-002: Standby サーバー）

| 層 | ソフトウェア | 配置場所 | 稼働状態 |
|---|---|---|---|
| OS 層 | Windows Server 2022 LTSC | 物理 OS | 常時稼働 |
| Web サーバー層 | IIS 10 | Windows 上 | 待機中（切替時に有効化） |
| 仮想化層 | WSL2 | Windows 上 | 常時稼働 |
| コンテナランタイム | Docker Engine 26.x | WSL2 上 | 常時稼働 |
| DB コンテナ | `postgres`（PostgreSQL 16: レプリカ）| Docker Compose サービス | 常時稼働（レプリカ受信中）|
| API コンテナ | `wnav_api`（停止中）| Docker Compose サービス | 停止中（切替後に起動）|
| スケジューラ | `wnav_scheduler`（停止中）| Docker Compose サービス | 停止中（切替後に起動）|

---

## 3. Docker Compose サービス配置（NODE-001 基準）

### 3-1. サービス一覧表

| サービス名 | イメージ | ポートマッピング | ボリュームマウント | 再起動ポリシー | 依存関係 |
|---|---|---|---|---|---|
| `wnav_api` | `wnav/api:x.y.z`（Rust axum）| `127.0.0.1:8080:8080`（IIS からのみアクセス可）| `./config:/app/config:ro`・`./certs:/app/certs:ro` | `unless-stopped` | `postgres`（healthcheck 成功後）|
| `postgres` | `postgres:16-alpine` | `127.0.0.1:5432:5432`（外部非公開）| `pgdata:/var/lib/postgresql/data`・`./wal-archive:/wal-archive` | `unless-stopped` | なし |
| `wnav_scheduler` | `wnav/scheduler:x.y.z`（Rust バイナリ）| なし（HTTP 非公開）| `./config:/app/config:ro`・`./wal-archive:/wal-archive`・`pgdata`（読取専用）| `unless-stopped` | `postgres`（healthcheck 成功後）・`wnav_api` |

**ポートマッピング方針**: すべてのサービスポートは `127.0.0.1`（ループバック）にバインドする。`0.0.0.0` バインドは禁止する（ADR-SYS-005）。外部からのアクセスは IIS リバースプロキシ経由に限定する。

### 3-2. Docker 内部ネットワーク

| ネットワーク名 | 種別 | 参加サービス | 用途 |
|---|---|---|---|
| `wnav_backend` | bridge | `wnav_api`・`postgres`・`wnav_scheduler` | コンテナ間通信（API → DB・スケジューラ → DB）|

PostgreSQL の 5432 ポートは `wnav_backend` ネットワーク内のみ疎通可能とし、ホスト OS（Windows/WSL2）からは `127.0.0.1:5432` 経由でのみアクセスを許可する（保守用途）。

### 3-3. IIS リバースプロキシ設定（概要）

| 項目 | 内容 |
|---|---|
| 受信ポート（外部）| HTTPS 443（TLS 1.3 終端：IIS 側）|
| 転送先（内部）| `http://127.0.0.1:8080`（`wnav_api` コンテナ）|
| SPA 配信 | IIS の別サイト（`/` ルート）から React ビルド済み静的ファイルを配信 |
| HTTP → HTTPS リダイレクト | IIS で 301 リダイレクト設定（HTTP 80 は HTTPS 443 に強制リダイレクト）|
| TLS 証明書 | 社内 CA 発行の自己署名証明書（ハンディ端末・PC に社内 CA ルート証明書を配布） |

詳細な TLS 設計は 05_外部インターフェース設計 §11 で確定する。

---

## 4. PostgreSQL ストリーミングレプリケーション

### 4-1. レプリケーション構成

| 項目 | 内容 |
|---|---|
| レプリケーション方式 | ストリーミングレプリケーション（非同期）|
| プライマリ | NODE-001 の `postgres` コンテナ |
| スタンバイ | NODE-002 の `postgres` コンテナ |
| レプリケーション遅延目標 | 同一建屋・Gigabit Ethernet のため、通常 1 秒未満（SLO 未設定） |
| WAL アーカイブ | NODE-001 から NODE-003（NAS）へ 5 分間隔で転送（NFR-OPS-030）|
| PITR 対応 | WAL アーカイブから PITR（RPO 15 分を充足）|

### 4-2. フェイルオーバー手順（概要）

障害発生から手動切替完了までの目標時間は 1 時間（NFR-AVL-002 RTO 1 時間）。

| ステップ | 操作内容 | 担当 | 想定所要時間 |
|---|---|---|---|
| 1. 障害確認 | 現場監督 → IT 担当への連絡（社内電話・内線）| 現場 → IT 担当 | 15 分以内（NFR-OPS-020）|
| 2. 稼働状況確認 | NODE-001 の Docker Compose ステータス確認・PostgreSQL 接続確認 | IT 担当 | 10 分 |
| 3. Standby 昇格 | NODE-002 の PostgreSQL を `pg_promote` コマンドで昇格（プライマリ化）| IT 担当 | 5 分 |
| 4. API 起動 | NODE-002 の `wnav_api` コンテナを起動 | IT 担当 | 5 分 |
| 5. IIS 切替 | NODE-002 の IIS のリバースプロキシ転送先を確認・有効化 | IT 担当 | 5 分 |
| 6. 疎通確認 | ハンディ端末・PC ブラウザからの疎通テスト | IT 担当 | 10 分 |
| 7. 完了通知 | 現場監督への復旧完了報告 | IT 担当 | 5 分 |
| **合計** | | | **55 分（RTO 1 時間を充足）** |

**手動切替を採用する根拠（ADR-SYS-002）**: 自動フェイルオーバー（Patroni 等）は PostgreSQL の split-brain リスクを回避するためにクォーラム投票（3 ノード以上）が推奨される。単一建屋 2 ノード構成でクォーラムを確立することは困難であり、自動フェイルオーバーは不採用とする。RTO 1 時間の目標値は Offline-First による業務継続性（NFR-AVL-004）で実質的影響を最小化しているため、手動切替で充足可能と判断する。

---

## 5. ボリュームと永続化設計

### 5-1. Docker ボリューム一覧

| ボリューム名 | 種別 | マウント先（コンテナ内）| データ内容 | バックアップ対象 |
|---|---|---|---|---|
| `pgdata` | Named Volume | `/var/lib/postgresql/data` | PostgreSQL データファイル・WAL ファイル | 対象（WAL アーカイブ + pg_dump）|
| `./wal-archive` | Bind Mount（NAS マウントポイント）| `/wal-archive` | WAL アーカイブファイル（5 分間隔）| NODE-003（NAS）に直接書込み |
| `./config` | Bind Mount（読取専用）| `/app/config` | `wnav_api` 設定ファイル（環境変数を注入）| Git 管理（機密値は除外）|
| `./certs` | Bind Mount（読取専用）| `/app/certs` | TLS クライアント証明書（mTLS 用）| 別途鍵管理手順（KEY-001/007/008）|
| `./evidence-files` | Bind Mount | `/app/evidence-files` | 証拠ファイル（写真・PDF）| 対象（週次 AES-256 含む）|

### 5-2. メンテナンスウィンドウ

計画停止は NFR-AVL-003 に従い以下の条件で実施する。

| 項目 | 条件 |
|---|---|
| 実施時間帯 | 夜間（22:00 以降）または休日 |
| 最大停止時間 | 1 回あたり 2 時間以内 |
| 頻度上限 | 月 1 回 |
| 事前通知 | 実施 3 日前までに全作業員・現場監督に社内通知 |

---

**本節で確定した方針**
- Active-Standby（手動フェイルオーバー・RTO 55 分）と PostgreSQL ストリーミングレプリケーション（非同期・RPO 15 分 WAL アーカイブで充足）を可用性設計として確定し、NFR-AVL-001〜010 の充足手段を明示する。
- Docker Compose 3 サービス（`wnav_api`・`postgres`・`wnav_scheduler`）のポートマッピング（`127.0.0.1` バインド必須）・ボリューム・再起動ポリシー（`unless-stopped`）を確定し、IIS リバースプロキシ経由の外部公開を設計基盤とする（ADR-SYS-005）。
- 自動フェイルオーバーを単一建屋 2 ノード構成における split-brain リスクから非採用とし（ADR-SYS-002）、手動切替 55 分 + Offline-First 業務継続により RTO 1 時間 SLO を充足することを確定する。

---

## 参照業界分析

### 必須
- [`90_業界分析/07_スマートファクトリーと作業のデジタル化.md`](../../90_業界分析/07_スマートファクトリーと作業のデジタル化.md)
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)

### 関連
- [`90_業界分析/10_信頼性工学と保全活動.md`](../../90_業界分析/10_信頼性工学と保全活動.md)
- [`90_業界分析/38_災害・BCP・緊急時手順と作業継続.md`](../../90_業界分析/38_災害・BCP・緊急時手順と作業継続.md)
