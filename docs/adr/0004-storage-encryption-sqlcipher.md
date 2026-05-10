# ADR-0004: 端末暗号化: SQLCipher（AES-256）

> 提案日: 2026-05-09
> 採用日: 2026-05-09
> 提案者: @RyuheiKiso
> 関連 §: ロードマップ §11.4.2 §11.4.1 §27 F-008 §29 R-008 R-015

## Status

Accepted

## Type

Type 1（暗号化方式の変更は既存端末データの再暗号化を伴うため不可逆）

## Context

- §11.4.1 STRIDE「Information Disclosure」「Tampering」を端末ローカル DB（SQLite）レベルで防ぐ必要がある。
- §29 R-015（端末紛失・盗難での実績流出、優先度 12 M）／R-008（暗号鍵漏洩、優先度 8 L）の主たる緩和手段。
- §27 F-008（暗号鍵漏洩、AP=H）のリリースブロッカー対応。

## Decision

- **SQLCipher**（AES-256）を採用する。
- 鍵は OS Keystore（Android）／DPAPI（Windows）に保護する（端末紛失時のデータ復号を困難化）。
- パスワードキャッシュは Argon2id 派生鍵のみを保護保存する（§10.5.1）。
- 端末紛失時のリモートワイプは §16 観測ダッシュボードからトリガー可能（オプション）。

## Alternatives

| 代替案 | 概要 | 却下理由 |
| --- | --- | --- |
| 暗号化なし | 単純 | §11.4.1 Information Disclosure に対応不能 |
| OS の FDE（Full Disk Encryption）のみ依存 | 既定の機能 | 端末ロック解除済みでアプリが動作中の場合、アプリ層暗号化が無いと SQLite ファイルへの直接アクセスで漏洩 |
| 自前 AES-GCM 実装 | カスタム制御 | 暗号実装の自製は脆弱性リスクが高い／OSS 監査を経た SQLCipher の方が安全 |
| Realm Encryption | NoSQL 系 | DB エンジン置換が大きい／PostgreSQL バックエンド連携と SQL 互換性で SQLite が優位 |

## Consequences

- **正の帰結**: 端末紛失時の実績漏洩を構造的に防止。OS Keystore／DPAPI と統合され、鍵管理の責務を OS に委譲。
- **負の帰結**: SQLCipher は標準 SQLite よりわずかに性能オーバーヘッド（5〜15%）。§5.2 性能目標に影響しない範囲で運用。
- **影響範囲**: §11.4.2（端末暗号化）、§10.5（オフライン継続ログイン）、§14（端末アプリ配布）、§13.1 セキュリティテスト、§27 F-008、§29 R-008／R-015。

## Type 1 撤退条件

- SQLCipher の OSS メンテナンス停止が公式に告知された場合（直近 12 ヶ月の活動履歴で判断）。
- AES-256 が量子コンピュータ攻撃で実効破られる時代（NIST PQC 標準化に追従）。
- 撤退時の代替: NIST PQC 標準アルゴリズム（CRYSTALS-Kyber 等）への移行を ADR で再判断。

## §24.2 出所表への追記

- 追記済: Yes（§24.2「SQLCipher（AES-256）端末暗号化 → §11.4.2」行）

## References

- ロードマップ §11.4.2 §11.4.1 §10.5
- SQLCipher: <https://www.zetetic.net/sqlcipher/>
- NIST FIPS 197（AES）／FIPS 180-4（SHA-256）
- Argon2id（RFC 9106）
- 関連 FMEA: F-008
- 関連リスク: R-008／R-015
