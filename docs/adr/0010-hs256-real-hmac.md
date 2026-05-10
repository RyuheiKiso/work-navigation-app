# ADR-0010: セッショントークン署名の HMAC-SHA256 化

> 提案日: 2026-05-10
> 採用日: 2026-05-10
> 提案者: @RyuheiKiso
> 関連 §: ロードマップ §10.5 §11.4 §27 F-008 §29 R-008

## Status

Accepted

## Type

Type 1（暗号方式の選択は不可逆。鍵保護方針／監査証跡の前提条件となるため）

## Context

- ADR-0007 で「セッション発行は短寿命の不透明トークン」と決めたが、初期実装（Session 3）では
  依存追加コストを避けるため自前 FNV-1a の簡易ハッシュを使用していた。
- FNV-1a は **暗号学的ハッシュではなく**、§11.4 STRIDE Tampering（改ざん）／Spoofing（なりすまし）
  の脅威を防げない。本番運用前に必ず置換が必要。

## Decision

- セッショントークンの署名は **HMAC-SHA256**（`hmac` 0.12 + `sha2` 0.10 crate）を採用する。
- トークン形式は `<base64url(payload)>.<base64url(hmac)>`、payload は `<user_id>.<unix_ts>`。
- 検証時は **定数時間比較**（タイミング攻撃対策、§11.4.1 STRIDE Spoofing）。
- 秘密鍵は環境変数 `WNA_SESSION_SECRET` から取得し、未設定時は起動拒否。

## Alternatives

| 代替案 | 概要 | 却下理由 |
| --- | --- | --- |
| FNV-1a 自前ハッシュ（旧実装） | 依存追加なし | 暗号学的にゼロ／改ざん耐性なし |
| JWT（jsonwebtoken crate） | RFC 7519 標準 | クレーム形式が JWT に縛られ、不透明トークンの設計と齟齬／§17.3 アドオン互換性が複雑化 |
| Paseto | JWT より厳格 | エコシステムが小さい／本用途に対し過剰 |
| RS256（非対称） | 鍵分離が容易 | 端末側に公開鍵配布が必要、§14.2 セットアップ目標と不整合 |

## Consequences

- **正の帰結**: 改ざん耐性／なりすまし耐性／タイミング攻撃耐性が確保される。F-008 関連リスクの実質的低減。
- **負の帰結**: `hmac`／`sha2`／`base64` の 3 crate が追加。依存ツリーは小さい（いずれも RustCrypto エコシステム）。
- **影響範囲**: §10.5（認証）、§11.4（セキュリティ）、§17（アドオンの crypto.sign capability で同方針を採用予定）。

## Type 1 撤退条件

- HMAC-SHA256 が量子コンピュータ攻撃で実効破られる時代（NIST PQC 標準化に追従）。
- RustCrypto `hmac` crate が OSS としてメンテナンス停止。
- 撤退時の代替: NIST PQC 標準（CRYSTALS-Dilithium 等）への移行を ADR で再判断。

## §24.2 出所表への追記

- 追記済: Yes（§24.2 「認証方式選択根拠と認証拡張アドオン化 → §10.5.0」行に内包）

## References

- ロードマップ §10.5 §11.4 §27 F-008 §29 R-008
- RFC 2104 HMAC, RFC 6234 SHA-2
- NIST FIPS 180-4 SHA-256
- RustCrypto: <https://github.com/RustCrypto/MACs>
- 関連 ADR: ADR-0004（端末暗号化）／ADR-0007（既定認証）
