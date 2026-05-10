// 対応 §: ロードマップ §10.2.1 §13.1
// 実 YAML テンプレート（examples/flow-templates/*）に対するラウンドトリップ検証。
// 単体テストではなく統合テストの位置付けだが、依存追加なしで動かすため vitest で実行する。

import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { flowFromYaml } from './yaml-flow-parser';

// repo root への相対パス
const TEMPLATES_DIR = join(__dirname, '..', '..', '..', '..', 'examples', 'flow-templates');

/** 業界ディレクトリ配下の *.yaml を再帰的に列挙する */
function listYamlFiles(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir)) {
    const p = join(dir, entry);
    if (statSync(p).isDirectory()) {
      out.push(...listYamlFiles(p));
    } else if (p.endsWith('.yaml')) {
      out.push(p);
    }
  }
  return out;
}

describe('yaml-flow-parser real templates', () => {
  // 全 YAML を 1 件ずつパース可能か検証
  const files = listYamlFiles(TEMPLATES_DIR);

  it('finds at least one template', () => {
    // テンプレが見つかる
    expect(files.length).toBeGreaterThan(0);
  });

  it.each(files)('parses %s without throwing', (filePath) => {
    // ファイル読込
    const yaml = readFileSync(filePath, 'utf-8');
    // パース実行（throw しないこと）
    const flow = flowFromYaml(yaml);
    // 最低限の不変条件
    expect(flow.id.length).toBeGreaterThan(0);
    expect(flow.name.length).toBeGreaterThan(0);
    expect(flow.nodeCount).toBeGreaterThan(0);
  });
});
