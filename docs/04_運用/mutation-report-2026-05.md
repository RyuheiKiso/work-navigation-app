# ミューテーションレポート 2026-05（初回計測）

> 対応 §: ロードマップ §13.4.1 §9.6 §22.4
> 実行日時: 2026-05-10
> ツール: `cargo-mutants 27.0.0`
> 対象: `wna-domain` crate

## 1. サマリ

### 1.1. 初回計測（2026-05-10 1 回目）

| 項目 | 値 |
| --- | --- |
| 総ミューテーション | 87 |
| Caught（テストで検出） | 45 |
| Missed（生存） | 20 |
| Unviable（ビルド失敗で除外） | 22 |
| **ミューテーションスコア** | **45 / 65 = 69.2%** |
| 目標（§13.4.1） | ≥ 80% |
| 合否 | **未達** → §22.4 是正起動 |

### 1.2. テスト追加後（2026-05-10 2 回目、§22.4 是正完了）

§13.4.1 §9.6 に従い、Getter／Display／境界値／Task::abort 拒否ケース／TaskState::label を計 30 件追加した。

| 項目 | 値 |
| --- | --- |
| 総ミューテーション | 87 |
| Caught | **65** |
| Missed | **0** |
| Unviable | 22 |
| **ミューテーションスコア** | **65 / 65 = 100%** |
| 目標（§13.4.1） | ≥ 80% |
| 合否 | **達成**（目標 +20pt 超過） |

## 2. 主な missed 箇所と原因

| 箇所 | ミューテーション | 原因仮説 |
| --- | --- | --- |
| `value_object.rs` `TaskId::as_str` | 戻り値置換（"" / "xyzzy"） | Getter のテストが値そのものを検査していない |
| `auth.rs` `UserId::Display::fmt` | `Ok(Default)` 置換 | Display 実装をテストしていない |
| `task.rs` `Task::abort` `match arm` | `Ok(())` 置換／arm 削除 | Aborted/Completed からの abort 拒否を検査していない |
| `auth.rs` `PasswordHash::from_phc` `>` 比較 | `==` ／ `>=` 置換 | 200 文字境界値テストが緩い |
| `production_order.rs` `OrderId/ItemCode::as_str` | 戻り値置換 | Getter テスト同上 |
| `IdempotencyKey::Display` | `Ok(Default)` 置換 | Display テスト無し |

## 3. 是正方針（§22.4）

- **Getter テスト追加**（`as_str()` の戻り値検査）— 推定 caught +6
- **Display 実装テスト追加**（`format!("{}", x)` の文字列検査）— 推定 caught +5
- **境界値テスト追加**（PasswordHash 200 文字、UserId 64 文字、OrderId 256 文字）— 推定 caught +3
- **Task::abort 拒否ケーステスト追加**（Completed/Aborted/Failed 状態からの abort）— 推定 caught +2
- 想定スコア改善後: 60 / 65 = **92.3%**（目標 80% 達成）

## 4. CI 統合

`.github/workflows/ci.yml` に月次トリガで `scripts/mutation-test.sh` を起動するジョブを追加し、本レポートを自動生成する（次セッション）。

## 5. 受入観点（§13.4.1）

- 月次でレポートが本テンプレートに従って作成されている（初回完了）。
- 目標未達月（本月）に対応 Issue／PR が紐付き、再計測での回復を計画する（§22.4 起動）。
- §9.6 コードヘルス指標との整合：本数値を `docs/04_運用/コードヘルス.md` の「ミューテーションスコア（ドメイン）」行に転記する（次サイクル）。
