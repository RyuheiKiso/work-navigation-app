# 10 ライセンス・OSS・コンプライアンス方式

本章は、本システムが使用するオープンソースソフトウェア（OSS）のライセンス適合性検証方式、SBOM（Software Bill of Materials）の生成・管理方針、および許可・禁止ライセンスの分類基準を確定する。本システムはクローズドソース・オンプレミス展開を前提とするため、コピーレフトライセンス（GPL系・AGPL）の採用は著作権法上の義務の連鎖リスクを持ちスコープ外と判断する。すべての依存クレート・パッケージはリリース時にライセンス検証を通過することを必須要件とし、CIパイプラインでの自動検証を確定する。

---

## 1. ライセンス分類基準

### 1-1. 許可ライセンス一覧

| ライセンス識別子 | ライセンス名 | 採用可否 | 主な条件 |
|---|---|---|---|
| MIT | MIT License | **許可** | 著作権表示・ライセンス文の保持のみ。バイナリ配布に制限なし |
| Apache-2.0 | Apache License 2.0 | **許可** | 著作権表示・変更箇所の明示・特許ライセンス付与。NOTICE文書の保持 |
| BSD-2-Clause | 2-Clause BSD License | **許可** | 著作権表示・ライセンス文の保持のみ（広告条項なし） |
| BSD-3-Clause | 3-Clause BSD License | **許可** | BSD-2-Clause + 製品宣伝への名称使用禁止条項 |
| ISC | ISC License | **許可** | 実質的にMIT同等。著作権表示の保持のみ |
| PostgreSQL License | PostgreSQL License | **許可** | BSDスタイルの許容ライセンス。コピーレフト条項なし |
| Unlicense / CC0 | Public Domain相当 | **許可** | 著作権放棄。商用・クローズドソース利用に制限なし |

### 1-2. 禁止ライセンス一覧

| ライセンス識別子 | 禁止理由 | 代替方針 |
|---|---|---|
| GPL-2.0 / GPL-3.0 | コピーレフト。GPLライブラリとリンクしたソフトウェア全体のGPL公開義務が発生する。クローズドソース展開と非互換 | GPL依存のクレート/パッケージはMIT/Apache-2.0代替物への差し替えを必須とする |
| LGPL-2.0 / LGPL-2.1 / LGPL-3.0 | 静的リンクにはGPL同等の制限あり。動的リンクのみで回避できるが、Rustの静的リンクモデルとの整合性リスク | LGPL依存は原則禁止。採用が不可避の場合は法的リスク評価後にarchitect承認を必須とする |
| AGPL-3.0 | ネットワーク越しのサービス提供でもソース公開義務が発生（クラウドのコピーレフトトリガー）。オンプレミスでも将来的なSaaS転用リスク | 採用禁止 |
| EUPL-1.1 / EUPL-1.2 | EUオープンソースライセンス。コピーレフト条項あり | 採用禁止 |
| CC-BY-SA / CC-BY-NC | ソフトウェアライセンスとして不適切。NC（非商用）条項は工場業務利用と非互換 | 採用禁止 |
| 商用有料ライセンス（閉源） | ベンダーロックイン・データロックインリスク。個人開発者のライセンスコストリスク | 採用禁止（OSS代替物への置換を必須とする） |

### 1-3. 評価保留ライセンス（要確認）

以下のライセンスは採用前にarchitectが個別評価を行い、採用判断を記録する。

| ライセンス | 評価ポイント |
|---|---|
| MPL-2.0（Mozilla Public License） | ファイル単位コピーレフト。変更したMPLファイルのみ公開義務。リンク先の伝染なし → 個別評価で採用可とする場合あり |
| CDDL-1.0 | MPL類似のファイル単位コピーレフト → MPL-2.0と同様に個別評価 |

---

## 2. 主要依存コンポーネントのライセンス一覧

本システムの中核依存コンポーネントのライセンスを以下に確定する。

### 2-1. バックエンド（Rust crates）

| クレート名 | バージョン（概算） | ライセンス | 確認済みバージョン |
|---|---|---|---|
| axum | 0.7.x | MIT | MIT（tokio-rsプロジェクト） |
| tokio | 1.x | MIT | MIT |
| sqlx | 0.7.x | MIT / Apache-2.0 | MIT OR Apache-2.0 |
| serde | 1.x | MIT / Apache-2.0 | MIT OR Apache-2.0 |
| serde_json | 1.x | MIT / Apache-2.0 | MIT OR Apache-2.0 |
| jsonwebtoken | 9.x | MIT | MIT |
| argon2 | 0.5.x | MIT / Apache-2.0 | MIT OR Apache-2.0 |
| uuid | 1.x | MIT / Apache-2.0 | MIT OR Apache-2.0 |
| tracing | 0.1.x | MIT | MIT |
| tower | 0.4.x | MIT | MIT |
| json-logic-rs | 0.x | Apache-2.0 | Apache-2.0（確認必須） |

### 2-2. フロントエンド（React / React Native）

| パッケージ名 | バージョン（概算） | ライセンス |
|---|---|---|
| react | 18.x | MIT |
| react-native | 0.73.x | MIT |
| react-navigation | 6.x | MIT |
| TypeORM | 0.3.x | MIT |
| react-query（TanStack Query） | 5.x | MIT |
| zustand | 4.x | MIT |
| expo | 50.x | MIT |

### 2-3. インフラ・ランタイム

| ソフトウェア | ライセンス | 備考 |
|---|---|---|
| PostgreSQL 16 | PostgreSQL License（BSDスタイル、許可） | コピーレフトなし |
| Docker / Docker Compose | Apache-2.0 | Docker Engineのライセンス |
| SQLite | Public Domain | SQLite本体は著作権放棄 |
| SQLCipher | BSD | Zetetic LLCのBSD-style License |
| Rust Compiler (rustc) | MIT / Apache-2.0 | Rustコンパイラ自体のライセンス |

---

## 3. 禁止コンポーネント

### 3-1. 動的コード実行の禁止

本システムは製造現場の作業手順・品質判定ロジックを扱うため、任意コード実行が可能な依存を禁止する。

| 禁止事項 | 対象例 | 代替 |
|---|---|---|
| `eval()`を提供するパッケージ | `vm2`・`eval`・動的JSエンジン系 | json-logic-rs（宣言的条件評価、Turing不完全） |
| 動的コード生成・実行ライブラリ | `unsafe`なコード生成クレート（各種proc-macroは除外） | コンパイル時マクロのみ許可 |
| サンドボックスなしの外部スクリプト実行 | 任意シェルコマンド実行API | システム設計上の要件がない |

json-logic-rsはjson-logic仕様（Turing不完全な宣言的ルールエンジン）に準拠し、任意コード実行を不可能とした構造であるため採用を許可する（Apache-2.0ライセンス確認済み）。

---

## 4. SBOM（Software Bill of Materials）方式

### 4-1. SBOM生成タイミングと形式

| 項目 | 確定値 | 根拠 |
|---|---|---|
| 生成タイミング | 各リリース（v1.x.x）時。CIパイプラインで自動生成 | EO 14028（米国大統領令）・NTIA勧告準拠の精神に則る |
| 形式 | CycloneDX 1.5 JSON形式 | 機械可読性・業界標準（SBOMツールとの互換性） |
| 保管場所 | `docs/sbom/wnav-vX.X.X-sbom.cdx.json` | プロジェクトリポジトリ内に版管理 |
| カバー範囲 | バックエンドRust crates（Cargo.lock） + フロントエンドnpm packages（package-lock.json） + Dockerイメージのベースイメージ層 | 直接依存 + 間接依存の全件 |

### 4-2. ライセンス検証CIパイプライン

| ツール | 対象 | 実行タイミング | 失敗条件 |
|---|---|---|---|
| `cargo license` | Rust crates（Cargo.lock）の全依存ライセンスを列挙 | PRマージ時・リリースビルド時 | 禁止ライセンス（GPL-2.0/3.0・LGPL・AGPL等）を検出した場合はCIをFAIL |
| `license-checker` (npm) | Node.js packages（package-lock.json）の全依存ライセンスを列挙 | PRマージ時・リリースビルド時 | 同上 |
| `cyclonedx-bom` | CycloneDX形式のSBOMを生成 | リリースビルド時 | 生成失敗でCIをFAIL |

### 4-3. ライセンス表示義務の履行

MIT・Apache-2.0等の許可ライセンスは著作権表示の保持を義務付ける。

| 履行方法 | 対象 | 実装 |
|---|---|---|
| バックエンドバイナリへのライセンス同梱 | Rust crates | `cargo about generate` でTHIRD_PARTY_LICENSES.htmlを生成し、リリースアーカイブに同梱 |
| フロントエンドへのライセンス表示 | npm packages | `license-checker --out THIRD_PARTY_LICENSES.txt` でリスト生成。APPのAboutスクリーン（SCR-HA-ABOUT）から参照可能にする |
| Apache-2.0依存のNOTICEファイル保持 | axum・json-logic-rs等 | NOTICEファイルをリリースアーカイブに含める |

---

## 5. データオーナーシップとエクスポート方式

本システムは工場データの完全なオーナーシップを顧客（工場オーナー）に保証する。データのクラウドロックイン・ベンダーロックインは設計上禁止する。

| エクスポート方式 | 識別子 | 実装方式 | 利用者 |
|---|---|---|---|
| Webダウンロード | IF-EXPORT-001 | マスタメンテナンス画面（SCR-MA-EXP-001）からCSV / JSON一括ダウンロード | 工場管理者 |
| ファイルシステムアクセス | IF-EXPORT-002 | Windows Serverのエビデンスファイルディレクトリ・PostgreSQLバックアップ（pg_dump）への直接アクセス | system_admin |
| REST APIエクスポート | IF-EXPORT-003 | `GET /api/export/work-events`・`GET /api/export/work-instances`（ページネーション付きJSONストリーム） | system_admin・外部システム |

---

**本節で確定した方針**

- **許可ライセンスはMIT・Apache-2.0・BSD-2/3-Clause・PostgreSQL Licenseを確定し、GPL-2.0/3.0・LGPL・AGPL・商用クローズドライセンスおよびeval()を提供するパッケージを禁止事項として確定する。**
- **SBOMはCycloneDX 1.5 JSON形式でリリースごとに自動生成し、cargo license + license-checkerをCIパイプラインに組み込んで禁止ライセンスを自動検出・CIFAILする方式を確定する。**
- **すべての工場データは3方式（Webダウンロード・ファイルシステム・REST API）でエクスポート可能とし、クラウドロックイン・ベンダーロックインを設計上排除したデータオーナーシップ保証を確定する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../90_業界分析/06_品質管理とトレーサビリティ.md) — 製造業でのデータオーナーシップ・長期保管要件の業界標準

### 関連
- [`90_業界分析/07_スマートファクトリーと作業のデジタル化.md`](../90_業界分析/07_スマートファクトリーと作業のデジタル化.md) — OSSスタック選定の業界動向
- [`90_業界分析/30_国内製造業IT調達慣行とSI構造.md`](../90_業界分析/30_国内製造業IT調達慣行とSI構造.md) — 国内製造業のOSS採用慣行と商用ライセンスリスク管理
