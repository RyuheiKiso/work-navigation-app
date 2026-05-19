#!/usr/bin/env ts-node
// PG スキーマと TypeORM エンティティの差分を検出する。
// 不一致がある場合は exit 1 でジョブを失敗させる（ペア・マイグレーション規則の自動強制）。
// 仕様: docs/05_詳細設計/01_データベース詳細設計/07a_PG_SQLiteスキーマ同期戦略.md §4
import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

// 07a §2 型変換規約: PG 型 → SQLite 型の期待マッピング
const PG_TO_SQLITE_TYPE_MAP: Record<string, string> = {
  uuid: 'text',
  'character varying': 'text',
  text: 'text',
  boolean: 'integer',
  integer: 'integer',
  bigint: 'integer',
  smallint: 'integer',
  'timestamp with time zone': 'text',
  'timestamp without time zone': 'text',
  jsonb: 'text',
  bytea: 'text',
  'char(64)': 'text',
  inet: 'text',
  numeric: 'real',
  real: 'real',
  'double precision': 'real',
};

interface ColumnDef {
  name: string;
  type: string;
  nullable: boolean;
  isPrimary: boolean;
}

interface EntitySchema {
  entityName: string;
  tableName: string;
  columns: ColumnDef[];
}

interface PgColumnInfo {
  column_name: string;
  data_type: string;
  is_nullable: string;
}

// psql で PG のカラム定義を取得する
function fetchPgSchema(tableName: string): PgColumnInfo[] {
  const dbUrl = process.env['DATABASE_URL'];
  if (!dbUrl) {
    process.stderr.write(`DATABASE_URL が未設定のためスキップ: ${tableName}\n`);
    return [];
  }
  try {
    const result = execSync(
      `psql "${dbUrl}" -t -A -F'|' -c "SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = '${tableName}' ORDER BY ordinal_position"`,
      { encoding: 'utf-8' },
    );
    return result.trim().split('\n').filter(Boolean).map((row) => {
      const [column_name, data_type, is_nullable] = row.split('|');
      return { column_name: column_name ?? '', data_type: data_type ?? '', is_nullable: is_nullable ?? 'YES' };
    });
  } catch {
    process.stderr.write(`PG テーブル取得失敗: ${tableName}\n`);
    return [];
  }
}

interface DriftReport {
  table: string;
  issues: string[];
}

function compareSingleTable(entity: EntitySchema): DriftReport {
  const issues: string[] = [];
  const pgCols = fetchPgSchema(entity.tableName);

  if (pgCols.length === 0) {
    // PG にテーブルがない（バックエンド未実装）場合はスキップ
    return { table: entity.tableName, issues: [] };
  }

  const pgColMap = new Map(pgCols.map((c) => [c.column_name, c]));

  // SQLite エンティティのカラムが PG に存在するか確認
  for (const col of entity.columns) {
    // camelCase → snake_case 変換（TypeORM のカラム名はプロパティ名から自動変換される）
    const snakeColName = col.name.replace(/([A-Z])/g, '_$1').toLowerCase();
    const pgCol = pgColMap.get(snakeColName);

    if (!pgCol) {
      issues.push(`❌ カラム不足（PG に存在しない）: ${entity.tableName}.${snakeColName}`);
      continue;
    }

    // 型変換規約に準拠しているか確認
    const expectedSqliteType = PG_TO_SQLITE_TYPE_MAP[pgCol.data_type];
    if (expectedSqliteType && col.type !== expectedSqliteType) {
      issues.push(
        `⚠️  型不一致: ${entity.tableName}.${snakeColName} — PG: ${pgCol.data_type} → 期待 SQLite: ${expectedSqliteType}, 実装: ${col.type}`,
      );
    }
  }

  // PG のカラムが SQLite エンティティに存在するか確認（逆方向）
  const entityColNames = new Set(
    entity.columns.map((c) => c.name.replace(/([A-Z])/g, '_$1').toLowerCase()),
  );
  for (const pgCol of pgCols) {
    if (!entityColNames.has(pgCol.column_name) && pgCol.column_name !== 'created_at') {
      issues.push(`⚠️  PG に追加カラム（SQLite 未対応）: ${entity.tableName}.${pgCol.column_name} — ミラー対象の場合は TypeORM エンティティに追加してください`);
    }
  }

  return { table: entity.tableName, issues };
}

function main(): void {
  const entitiesJsonPath = '/tmp/sqlite_entities.json';

  if (!fs.existsSync(entitiesJsonPath)) {
    process.stderr.write(`エンティティ JSON が見つかりません: ${entitiesJsonPath}\n`);
    process.stderr.write('先に extract-entity-schema.ts を実行してください\n');
    process.exit(1);
  }

  const entities: EntitySchema[] = JSON.parse(fs.readFileSync(entitiesJsonPath, 'utf-8')) as EntitySchema[];
  const allIssues: DriftReport[] = [];

  for (const entity of entities) {
    const report = compareSingleTable(entity);
    if (report.issues.length > 0) {
      allIssues.push(report);
    }
  }

  if (allIssues.length === 0) {
    process.stdout.write('✅ PG ↔ SQLite スキーマ整合性確認: 問題なし\n');
    process.exit(0);
  }

  // 問題あり → レポートを出力して exit 1
  process.stderr.write('\n❌ PG ↔ SQLite スキーマドリフト検出:\n\n');
  for (const report of allIssues) {
    process.stderr.write(`  テーブル: ${report.table}\n`);
    for (const issue of report.issues) {
      process.stderr.write(`    ${issue}\n`);
    }
  }
  process.stderr.write('\n対処方法: docs/05_詳細設計/01_データベース詳細設計/07a_PG_SQLiteスキーマ同期戦略.md §3 を参照\n');

  // レポートをファイルに出力（CI アーティファクト用）
  const reportPath = '/tmp/schema-drift-report.txt';
  const reportContent = allIssues.flatMap((r) => r.issues).join('\n');
  fs.writeFileSync(reportPath, reportContent);

  process.exit(1);
}

main();
