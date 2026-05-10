# 業界別フローテンプレート

> 対応 §: ロードマップ §10.2.1（テンプレート） §3.1.5.4（業界別語感） §22.1

§10.2.1 で要請される「業界別（自動車／医薬／食品／電子）の初期テンプレ同梱」の格納先。
本ディレクトリの YAML／JSON テンプレートを設定 UI（§7.2）からインポートして利用する。

## 同梱テンプレ

| ディレクトリ | 業界 | 規制適合 | 状態 |
| --- | --- | --- | --- |
| `automotive/` | 自動車部品 | IATF 16949 §8.3 | 初版整備済 |
| `pharma/` | 医薬品 | FDA 21 CFR Part 11／EU GMP Annex 11 | 未整備（次セッション） |
| `food/` | 食品 | HACCP／ISO 22000 | 未整備（次セッション） |
| `electronics/` | 電子部品 | ISO 9001／IPC | 未整備（次セッション） |

## YAML スキーマ

```yaml
# 例: 自動車部品の組立フロー
flow:
  id: auto-assembly-v1
  name: 自動車部品 組立ライン基本フロー
  industry: automotive
  schema_version: 1
  nodes:
    - id: start
      kind: start
      label: 着手
    - id: receive_parts
      kind: step
      label: 部材受領
      completion_criteria: photo  # §3.1.1 完了条件
      standard_time_seconds: 30
    - id: assemble
      kind: step
      label: 組立
      completion_criteria: manual
      standard_time_seconds: 120
    - id: torque_check
      kind: step
      label: 締結トルク検査
      completion_criteria: photo
      standard_time_seconds: 30
    - id: end
      kind: end
      label: 完了
  edges:
    - from: start
      to: receive_parts
    - from: receive_parts
      to: assemble
    - from: assemble
      to: torque_check
    - from: torque_check
      to: end
```

## 受入観点（§10.2.1）

- 5 階層ネスト（工場・ライン・セル・工程・手順）まで遅延無く編集可能であること（テンプレート側でこの 5 階層を例示）。
- 業界別テンプレで現場用語（§3.1.5.4）に整合していること。
- 適用時の項目漏れがウィザードで補完できること。
