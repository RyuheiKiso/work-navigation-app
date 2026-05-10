// 対応 §: ロードマップ §10.2.1 §13.1
// YAML パーサと flowFromYaml の単体テスト。

import { describe, it, expect } from 'vitest';
import { flowFromYaml, parseYaml } from './yaml-flow-parser';

describe('parseYaml', () => {
  it('parses simple key/value', () => {
    expect(parseYaml('foo: bar')).toEqual({ foo: 'bar' });
  });

  it('parses numeric and boolean scalars', () => {
    const r = parseYaml('a: 1\nb: true\nc: 3.14') as { a: number; b: boolean; c: number };
    expect(r.a).toBe(1);
    expect(r.b).toBe(true);
    expect(r.c).toBe(3.14);
  });

  it('parses nested dict', () => {
    const r = parseYaml('outer:\n  inner: 1') as { outer: { inner: number } };
    expect(r.outer.inner).toBe(1);
  });

  it('parses list of inline scalars', () => {
    const r = parseYaml('list:\n  - a\n  - b\n  - c') as { list: string[] };
    expect(r.list).toEqual(['a', 'b', 'c']);
  });

  it('parses list of inline maps', () => {
    const yaml = 'items:\n  - id: n1\n    label: A\n  - id: n2\n    label: B';
    const r = parseYaml(yaml) as { items: { id: string; label: string }[] };
    expect(r.items.length).toBe(2);
    expect(r.items[0]).toEqual({ id: 'n1', label: 'A' });
  });
});

describe('flowFromYaml', () => {
  it('parses minimal flow', () => {
    const yaml = `flow:
  id: f1
  name: テスト
  schema_version: 1
  nodes:
    - id: start
      kind: start
      label: 開始
    - id: end
      kind: end
      label: 終了
  edges:
    - from: start
      to: end`;
    const f = flowFromYaml(yaml);
    expect(f.id).toBe('f1');
    expect(f.name).toBe('テスト');
    expect(f.version).toBe(1);
    expect(f.nodeCount).toBe(2);
    expect(f.edgeCount).toBe(1);
  });

  it('reads industry field', () => {
    const yaml = `flow:
  id: f2
  name: 自動車
  industry: automotive
  nodes:
    - id: start
      kind: start
      label: 開始
  edges: []`;
    const f = flowFromYaml(yaml);
    expect(f.industry).toBe('automotive');
  });

  it('rejects flow without start node', () => {
    const yaml = `flow:
  id: f3
  name: bad
  nodes:
    - id: only
      kind: step
      label: 単独
  edges: []`;
    expect(() => flowFromYaml(yaml)).toThrow(/開始ノード/);
  });

  it('rejects unknown node kind', () => {
    const yaml = `flow:
  id: f4
  name: bad
  nodes:
    - id: x
      kind: invalid
      label: x
  edges: []`;
    expect(() => flowFromYaml(yaml)).toThrow(/不正なノード種別/);
  });
});
