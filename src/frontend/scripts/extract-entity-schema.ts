#!/usr/bin/env ts-node
// TypeORM エンティティから AST を抽出し、PG スキーマとの比較用 JSON を出力する。
// 仕様: docs/05_詳細設計/01_データベース詳細設計/07a_PG_SQLiteスキーマ同期戦略.md §4
import * as fs from 'fs';
import * as path from 'path';

// ミラー対象テーブルのエンティティファイル一覧（07a §1 の 12テーブル + TBL-051）
const MIRROR_ENTITIES = [
  'LocalWorkEvent',
  'LocalOutboxEvent',
  'LocalWorkExecution',
  'LocalSop',
  'LocalStep',
  'LocalEvidenceFile',
  'LocalSuspension',
  'LocalAndonAlert',
  'LocalNonconformity',
  'LocalLot',
  'LocalInstrument',
  'LocalCaseLock',
] as const;

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

// TypeORM デコレータのアノテーションをパースしてカラム定義を抽出する
function parseEntityFile(filePath: string): EntitySchema | null {
  if (!fs.existsSync(filePath)) return null;
  const content = fs.readFileSync(filePath, 'utf-8');

  // テーブル名の抽出（@Entity('table_name') または @Entity()）
  const tableMatch = content.match(/@Entity\(['"]([^'"]+)['"]\)/);
  const entityMatch = content.match(/export class (\w+)/);
  if (!entityMatch) return null;

  const entityName = entityMatch[1] ?? '';
  const tableName = tableMatch?.[1] ?? entityName.replace(/^Local/, '').replace(/([A-Z])/g, '_$1').toLowerCase().slice(1);

  const columns: ColumnDef[] = [];

  // @PrimaryColumn と @Column のパース
  const columnPattern = /@(PrimaryColumn|Column)\(([^)]*)\)\s+(\w+)!: (\w+)/g;
  let match: RegExpExecArray | null;

  while ((match = columnPattern.exec(content)) !== null) {
    const isPrimary = match[1] === 'PrimaryColumn';
    const decoratorArgs = match[2] ?? '';
    const colName = match[3] ?? '';

    // SQLite 型の抽出（'text', 'integer', 'real' 等）
    const typeMatch = decoratorArgs.match(/'(\w+)'/);
    const colType = typeMatch?.[1] ?? 'text';

    // { nullable: true } パターンの検出
    const isNullable = decoratorArgs.includes('nullable: true');

    columns.push({ name: colName, type: colType, nullable: isNullable, isPrimary });
  }

  return { entityName, tableName, columns };
}

function main(): void {
  const entitiesDir = path.resolve(__dirname, '../terminal/db/entities');
  const results: EntitySchema[] = [];

  for (const entityName of MIRROR_ENTITIES) {
    const filePath = path.join(entitiesDir, `${entityName}.ts`);
    const schema = parseEntityFile(filePath);
    if (schema) {
      results.push(schema);
    } else {
      process.stderr.write(`⚠️  エンティティファイルが見つかりません: ${filePath}\n`);
    }
  }

  // JSON を stdout に出力（CI の次ステップが /tmp/sqlite_entities.json に保存する）
  process.stdout.write(JSON.stringify(results, null, 2));
}

main();
