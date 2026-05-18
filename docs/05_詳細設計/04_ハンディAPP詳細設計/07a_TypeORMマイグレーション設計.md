# 07a TypeORM マイグレーション設計（端末側スキーマ進化管理）

本章の責務は、ハンディ APP の SQLite スキーマをアプリアップデートを通じて安全に進化させるための TypeORM Migration 規約を確定することである。ADR-006 のクライアント側実装規約。

---

## 1. ツール選定

`typeorm migration:generate` を採用する。

**却下した代替案**:
- **Flyway**: PostgreSQL 向けに最適化されており、TypeORM エンティティとの型整合が自動保証されない。SQLite サポートが限定的
- **Liquibase**: XML/YAML 形式はエンジニアの認知コストが高く、TypeScript プロジェクトとの統合が煩雑
- **手動 SQL**: スキーマと TypeORM エンティティの乖離を防ぐ仕組みがなく、型安全性が失われる

## 2. 命名規約

```
{ts_ms}-{description}.ts
```

例: `1716034496000-AddRetryCountToOutbox.ts`

**PG 側（sqlx）との命名差異**: PG は `V{YYYYMMDDHHMMSS}__{desc}.sql` 形式。TypeScript の `.ts` 拡張子と `{ts_ms}` プレフィックスにより、ファイル種別の誤認識を防止する（ADR-006）。

## 3. ペア・マイグレーション規則

ミラー対象テーブル（`01_データベース詳細設計/07a_PG_SQLiteスキーマ同期戦略.md` §1）の DDL 変更時は、PG sqlx migration と同一 PR で本マイグレーションファイルを生成する。

**生成コマンド（参考）**:
```bash
npx typeorm migration:generate src/db/migrations/{ts_ms}-{description}
```

## 4. ロールバック方針

TypeORM 公式の `migration:revert` は端末では非サポート（前進修正のみ）。

理由:
- 端末データは Append-only 原則に従い削除しない
- アプリ配布の単方向性（App Store / MDM）によりバージョンダウングレードは実質不可
- 破壊的な変更が必要な場合は「端末初期化 + サーバーから再同期」を緊急手順とする

## 5. SQLCipher 鍵ローテーション時のマイグレーション手順

SQLCipher を使用する場合、マイグレーションは必ず鍵 unlock 後のセッションでのみ実行する。

鍵ローテーション時のチェックリスト:
- [ ] 旧鍵でデータベースを unlock する
- [ ] `PRAGMA rekey = '新鍵'` を実行する
- [ ] 新鍵で再接続して正常動作を確認する
- [ ] バックアップを別パスに保存する（`VACUUM INTO '...'`）
- [ ] マイグレーションを実行する
- [ ] マイグレーション後の整合性を検証する

## 6. TypeORM Entity 規約

```typescript
// 端末専用テーブルは @Entity() デコレータに明示的にテーブル名を指定する
@Entity('case_lock_local')
export class LocalCaseLock {
  @PrimaryColumn('text')
  case_id: string; // UUID v7 as TEXT

  @Column('text')
  terminal_id: string;

  @Column('text')
  user_id: string;

  @Column('text')
  acquired_at: string; // ISO 8601 UTC

  @Column('text')
  heartbeat_at: string; // ISO 8601 UTC

  @Column('text')
  lock_status: 'ACTIVE' | 'RELEASED' | 'EXPIRED';
}
```

`synchronize: false` を `data-source.ts` で必ず維持する（自動スキーマ更新禁止）。

## 参照

- ADR-006（PG↔SQLite ペア・マイグレーション戦略）
- `01_データベース詳細設計/07a_PG_SQLiteスキーマ同期戦略.md` — 同期戦略全体方針
- `01_データベース詳細設計/07_マイグレーションスクリプト設計.md` — PG 側マイグレーション規約
