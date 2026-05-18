# 07a PG ↔ SQLite スキーマ同期戦略

本章の責務は、PostgreSQL（バックエンド）と SQLite（ハンディ APP）の間でスキーマ整合性を維持するための規約・CI チェック・テスト設計を確定することである。ADR-006 の技術的実装規約。

---

## 1. ミラー対象テーブル（PG → SQLite 同期対象）

以下の 12 テーブル（+ 端末専用テーブル 2 件）が SQLite にミラーされる。

| TBL-ID | PG テーブル名 | SQLite エンティティ名 | 備考 |
|---|---|---|---|
| TBL-001 | work_events | WorkEvent | Append-only |
| TBL-003 | outbox_events | OutboxEvent | Append-only + status UPDATE |
| TBL-005 | work_executions | WorkExecution | 更新可 |
| TBL-007 | sops | Sop | マスタ（版管理）|
| TBL-008 | steps | Step | マスタ（版管理）|
| TBL-009 | evidence_files | EvidenceFile | Append-only |
| TBL-011 | suspensions | Suspension | Append-only |
| TBL-012 | andon_alerts | AndonAlert | 更新可 |
| TBL-013 | nonconformities | Nonconformity | 更新可 |
| TBL-024 | lots | Lot | マスタ |
| TBL-026 | instruments | Instrument | マスタ |
| TBL-051 | case_locks | LocalCaseLock | 制御テーブル（例外: UPDATE/DELETE 許可）|
| （端末専用）| — | AppSettings | ローカル設定 |

**PG-only テーブル（SQLite に同期しない）**: TBL-002/004/006/010/014〜023/025/027〜050

## 2. 型変換規約

| PG 型 | SQLite アフィニティ | TypeORM 型 | 注意点 |
|---|---|---|---|
| UUID | TEXT | string | UUID v7 は TEXT として保存 |
| TIMESTAMPTZ | TEXT | string | ISO 8601 UTC 形式（例: `2026-05-18T12:34:56.000Z`）|
| JSONB | TEXT | string | canonical JSON として保存 |
| BOOLEAN | INTEGER | boolean | 0/1 → TypeORM が変換 |
| SMALLINT/INT | INTEGER | number | — |
| BIGSERIAL | （端末非対応）| — | PG-only。端末では UUID を使用 |
| BYTEA | TEXT | string | hex エンコード |
| TEXT/VARCHAR | TEXT | string | — |
| CHAR(64) | TEXT | string | SHA-256 ハッシュ値（64 hex chars）|
| INET | TEXT | string | IP アドレスを文字列として保存 |

## 3. ペア・マイグレーション規則

ミラー対象テーブルの DDL を変更する PR には、以下の両方を含めること:

1. **sqlx migration（PG 側）**: `V{YYYYMMDDHHMMSS}__{description}.sql` 形式
2. **TypeORM migration（SQLite 側）**: `{ts_ms}-{description}.ts` 形式

どちらか一方のみの PR は CI の `pg-sqlite-drift-check` ジョブがブロックする。

### PR チェックリスト（レビュアー確認項目）

- [ ] 変更する TBL が「ミラー対象テーブル」に含まれるかを確認した
- [ ] PG と SQLite 両方のマイグレーションファイルを作成した
- [ ] TypeORM エンティティの型定義が §2 型変換規約に準拠している
- [ ] 新規カラムがある場合、NULL デフォルト値で追加している（端末の前進修正方針）
- [ ] TST-intg の型ラウンドトリップテストが通過している
- [ ] PG マイグレーション名と TypeORM マイグレーション名に意図的な命名差異がある（ツール混同防止）
- [ ] `sqlx::query!` マクロが `cargo sqlx prepare --check` をパスしている
- [ ] TypeORM `synchronize: false` が data-source.ts で維持されている（自動スキーマ更新禁止）

## 4. CI ドリフト検出（`pg-sqlite-drift-check` ジョブ）

GitHub Actions のジョブとして以下を実装する:

```yaml
# .github/workflows/schema-sync-check.yml
pg-sqlite-drift-check:
  steps:
    - name: Start PG container
      # testcontainers or docker compose でテスト用 PG を起動
    - name: Run PG migrations
      run: cargo sqlx migrate run
    - name: Extract PG schema (mirror tables only)
      run: |
        psql -c "\d+ work_events" > /tmp/pg_work_events.txt
        # 12 ミラーテーブル + TBL-051 を抽出
    - name: Extract TypeORM Entity AST
      run: npx ts-node scripts/extract-entity-schema.ts > /tmp/sqlite_entities.json
    - name: Compare
      run: npx ts-node scripts/compare-pg-sqlite-schema.ts
      # 不一致がある場合: exit 1 でジョブを失敗させる
```

比較スクリプトは「カラム名の過不足」「NULL 制約の差異」「型変換規約に違反した型」を検出する。

## 5. データ型ラウンドトリップテスト設計

以下の 3 件を統合テスト（TST-intg-021〜023）として実装する。テスト経路: PG → API JSON レスポンス → TypeORM で SQLite INSERT → TypeORM で SELECT → API JSON リクエスト → PG INSERT 後 SELECT して一致確認。

| TST-ID | 対象型 | テスト値の例 | 確認観点 |
|---|---|---|---|
| TST-intg-021 | UUID v7 | `019571a3-7c4f-7e46-9b28-1234567890ab` | テキスト形式往復で一致する |
| TST-intg-022 | TIMESTAMPTZ | `2026-05-18T12:34:56.123Z` | UTC ISO 8601 形式往復で一致する（マイクロ秒は許容丸め）|
| TST-intg-023 | JSONB | `{"ja":"作業","en":"Work","zh":"作业"}` | canonical JSON 往復でキー順と値が一致する |

## 参照

- ADR-006（PG↔SQLite ペア・マイグレーション戦略）
- `07_マイグレーションスクリプト設計.md` — PG 側マイグレーション規約（権威）
- `04_ハンディAPP詳細設計/07a_TypeORMマイグレーション設計.md` — SQLite 側マイグレーション規約（権威）
