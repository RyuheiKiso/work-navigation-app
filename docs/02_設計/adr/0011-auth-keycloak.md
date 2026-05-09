# ADR-0011. 認証基盤: Keycloak（OIDC 委譲）

| 項目 | 内容 |
|---|---|
| Status | Accepted |
| Date | 2026-05-09 |
| Deciders | RyuheiKiso |
| 関連 | 企画書 §8.3 ／ §13.9（v0.7.1 で本決定）、上流制約 C-5 ／ C-6 |

## Context and Problem Statement

端末アプリ・メンテナンス Web の双方で **ID／パスワード認証**（企画書 §13.9、v0.7.1 で本決定）を提供する必要がある。認証・認可は再発明せず OSS の IdP に委譲する方針（企画書 §9 Wardley Map「G 認証・認可」を Product 領域に置く判断）が確定しており、具体的な IdP を 1 つに絞る。Keycloak と Authentik 等の OSS IdP 候補から、単一メンテナでの保守性・OIDC 完全対応・Apache-2.0 互換ライセンス・採用層の認知度を基準に選定する。

## Decision Drivers

- **OIDC 完全対応**: メンテナンス Web の React SPA と API、端末 Tauri アプリの双方で標準 OIDC フローを利用できること。
- **ライセンス互換**: Apache-2.0 と互換のライセンスであること（企画書 §14.1）。
- **単一メンテナの保守限界**: ドキュメントの厚さ・デプロイの単純さ・LTS 期間・脆弱性対応の頻度。
- **採用層の認知度**: 企業情シス・社内エンジニア層で前例があり、導入時の法務・セキュリティ確認が短縮されること。
- **パスワードポリシー・ブルートフォース対策の組込**: 企画書 §13.9 が要求する機能が標準で備わっていること。
- **Docker 1 コマンド配布との整合**: docker-compose.yml に同梱可能なこと（企画書 §14.4）。
- **オフライン端末との整合**: 端末側で長期トークン保持・リフレッシュトークン運用が成立すること（企画書 §6.4 オフライン完全動作との整合）。

## Considered Options

1. **Keycloak**（Apache-2.0、Red Hat / CNCF Sandbox）
2. Authentik（MIT、ベンチャー由来）
3. Ory Hydra ＋ Kratos（Apache-2.0、コンポーネント分離）
4. Zitadel（Apache-2.0、新興）
5. 自作（パスワード保管・OIDC エンドポイント実装）

## Decision Outcome

**選定: Keycloak**

OSS IdP の中で最も枯れており、企業情シスでの前例が多い。Apache-2.0 ライセンスで本プロジェクトと互換、ドキュメント・LTS の厚みで単一メンテナの保守負担を最小化できる。OIDC・パスワードポリシー・ブルートフォース対策・パスワード強度ポリシー・MFA（将来拡張用）が標準搭載で、企画書 §13.9 の要件をそのまま満たす。

### Consequences

- **Good**: OIDC が完全実装されており、React SPA（PKCE 付き Authorization Code Flow）と Tauri 端末（同フロー＋ OS Keychain）の双方で標準ライブラリ（oidc-client-ts ／ tauri-plugin-oauth 等）が利用できる。
- **Good**: パスワードポリシー（最小長・複雑度・履歴・有効期限）とブルートフォース対策（試行回数制限・一時ロック）が標準機能として搭載され、企画書 §13.9 の要件を満たす（自作不要）。
- **Good**: 企業情シスでの導入実績が多く、企画書 §15.1 H3（OSS 製であることが採用にプラス）に整合。
- **Good**: 将来の MFA／SSO（社内 LDAP・Active Directory 連携）拡張余地を持つ（本企画書では NS-8 として非スコープだが、再評価時の選択肢を保持）。
- **Bad**: 起動時メモリ消費が他候補より大きい（〜600MB 程度）。中小製造業の自前ホスト環境（NUC 等）では検証が必要。docker-compose.yml で `JAVA_OPTS_APPEND` を調整する設計とする。
- **Bad**: 設定項目が多く、初期セットアップで誤設定リスクがある。導入チェックリスト（企画書 §14.4）に Keycloak 初期設定の必須項目（realm／client／パスワードポリシー／ブルートフォース閾値）を含める。
- **Bad**: バージョンアップ時のスキーマ移行が必要な場合がある。本プロジェクトは Keycloak の LTS バージョン系列（Quarkus ベース）に追従する。

### 設計上の合意事項

- **Realm 構成**: 1 インスタンス＝1 工場（企画書 §14.10）に対応し、Realm は `work-nav` の単一構成。論理分離は Group ／ Role で行う。
- **Client 構成**:
  - `work-nav-web`（React SPA、Public Client、PKCE 必須）
  - `work-nav-api`（Bearer Only、メンテナンス Web の API バックエンド）
  - `work-nav-terminal`（Tauri 端末、Public Client、PKCE 必須、Refresh Token 長期保持）
- **トークン**: Access Token 短期（5〜15 分、設計書 §11 で確定）、Refresh Token 中期（端末オフライン許容の数日〜数週間）。具体値は設計書で確定。
- **オフライン認証**: 端末は最後に取得した Access Token の検証を JWT 公開鍵キャッシュで行い、Refresh Token のみオンライン時にローテーションする（企画書 §6.4 オフライン完全動作と整合）。
- **監査ログ**: Keycloak のイベントログを PostgreSQL に永続化し、本アプリの監査ログ（企画書 §13.9 ハッシュチェーン）と統合する設計とする（設計書 §11／§15）。

### 採用しない選択肢の取扱い

- Authentik / Ory / Zitadel / 自作はすべて却下。**単一メンテナの検証コスト集中**（複数候補の評価を続ける余裕がない）と、Keycloak が要件を充足することが理由。

## Pros and Cons of the Options

### Keycloak（採用）

- **Good**: OIDC 完全対応、ドキュメント厚、企業導入実績豊富、Apache-2.0、パスワードポリシー・ブルートフォース対策標準搭載。
- **Bad**: メモリ消費大、設定項目多。

### Authentik（却下）

- **Good**: モダンな UI、設定が直感的、軽量。
- **Bad**: コミュニティ規模が Keycloak より小さく、企業情シスでの認知度が劣る。
- **Bad**: 単一メンテナでの長期保守時のリソース不安（プロジェクト寿命がまだ短い）。

### Ory Hydra ＋ Kratos（却下）

- **Good**: コンポーネント分離で軽量、OIDC 完全対応。
- **Bad**: コンポーネントが分かれており、初期セットアップが複雑（Hydra: OAuth/OIDC、Kratos: Identity）。
- **Bad**: 単一メンテナでの構成管理負担が増える。

### Zitadel（却下）

- **Good**: モダン、Multi-tenant 完備、Apache-2.0。
- **Bad**: 新興でドキュメント・実績が Keycloak より薄い。
- **Bad**: 1 インスタンス＝1 工場の本プロジェクト要件に対し、Multi-tenant 機能が過剰。

### 自作（却下）

- **Good**: 完全な制御。
- **Bad**: パスワード保管・ハッシュ・ブルートフォース対策・OIDC 仕様準拠を自作することは、企画書 §7 アンチパターン「既製品の再発明」と Wardley Map（Product 領域での自前構築）の両方に違反する。
- **Bad**: セキュリティ脆弱性のリスクが単一メンテナでは許容できない。

## Links

- 企画書 §8.3 メンテナンス Web（認証基盤）
- 企画書 §13.9 セキュリティ
- 企画書 §15.1 H4（楔現場の物理制約下での ID／パスワード運用仮説）
- 企画書 §15.2 K9（認証起因の作業中断観測）
- ADR-0005 メディアストレージ pg_largeobject（アクセス制御の整合点）
- Keycloak Documentation: https://www.keycloak.org/documentation
- OpenID Connect Core 1.0: https://openid.net/specs/openid-connect-core-1_0.html
