// 対応 §: ロードマップ §10.2.1 §10.2.4 §3.1.5.4 §28
// examples/flow-templates の YAML ファイルを Flow Aggregate に変換する。
// 依存追加を避けるため、本リポジトリで使用する **限定的 YAML サブセット** を手書きで解析する。
// 対応構造:
//   - インデント 2 スペース固定
//   - キー: スカラー（文字列／数値／真偽）
//   - リストは "- " 始まり
//   - ネスト dict／list を再帰的に展開
//   - クォートは "..." と '...' を許容
//   - 真偽: true/false
//   - インライン JSON は未対応

import {
  Flow,
  type FlowEdge,
  type FlowNode,
  type FlowNodeKind,
  type FlowCompletionCriteria
} from '../domain/flow';

// ---------------------------------------------------------------------
// 型: 解析中間表現（generic な YAML ツリー）
// ---------------------------------------------------------------------
type YamlValue =
  | string
  | number
  | boolean
  | null
  | YamlValue[]
  | { [key: string]: YamlValue };

// ---------------------------------------------------------------------
// パース本体（限定 YAML）
// ---------------------------------------------------------------------

/** YAML 文字列を YamlValue へ解析する */
export function parseYaml(yaml: string): YamlValue {
  // 行ごとに分解（コメント除去、空行スキップ）
  const lines: { indent: number; text: string }[] = [];
  for (const raw of yaml.split('\n')) {
    // 末尾空白を先に除去
    let stripped = raw.replace(/\s+$/, '');
    // 行全体がコメント（先頭スペース 0 個以上 → "#"）であれば破棄
    if (/^\s*#/.test(stripped)) continue;
    // 末尾コメント（"  # comment"）を削除（文字列内 # は本サブセットでは扱わない）
    stripped = stripped.replace(/\s+#.*$/, '').replace(/\s+$/, '');
    // 空行はスキップ
    if (stripped === '') continue;
    // インデント幅（noUncheckedIndexedAccess に対応）
    const m = /^( *)/.exec(stripped);
    const indent = m && m[1] !== undefined ? m[1].length : 0;
    // 行追加
    lines.push({ indent, text: stripped.slice(indent) });
  }
  // ルートを解析（parentIndent=-1 で「親より深い」を全インデントで成立させる）
  const [val] = parseBlock(lines, 0, -1);
  // 結果を返す
  return val;
}

/** ブロックを解析する（再帰下降） */
function parseBlock(
  lines: { indent: number; text: string }[],
  start: number,
  parentIndent: number
): [YamlValue, number] {
  // 範囲外
  if (start >= lines.length) {
    return [null, start];
  }
  // noUncheckedIndexedAccess: 直接アクセスは undefined 可能
  const head = lines[start];
  if (!head) {
    return [null, start];
  }
  const childIndent = head.indent;
  // 親より深いインデントの行が存在しない → null
  if (childIndent <= parentIndent) {
    return [null, start];
  }
  // 先頭が "- " ならリスト
  if (head.text.startsWith('- ') || head.text === '-') {
    return parseList(lines, start, childIndent);
  }
  // それ以外はマップ
  return parseMap(lines, start, childIndent);
}

/** マップを解析する */
function parseMap(
  lines: { indent: number; text: string }[],
  start: number,
  myIndent: number
): [{ [k: string]: YamlValue }, number] {
  // 結果オブジェクト
  const obj: { [k: string]: YamlValue } = {};
  // カーソル
  let i = start;
  // 同レベル行を順に処理（noUncheckedIndexedAccess に対応）
  while (i < lines.length) {
    const cur = lines[i];
    if (!cur || cur.indent !== myIndent) break;
    // 末尾コロン（ブロック値）か "key: scalar" か
    const text = cur.text;
    const colonIdx = text.indexOf(':');
    if (colonIdx < 0) {
      // 不正
      throw new Error(`YAML 解析: コロンが無い行: ${text}`);
    }
    // キーと残り
    const key = text.slice(0, colonIdx).trim();
    const rest = text.slice(colonIdx + 1).trim();
    if (rest === '') {
      // 子ブロックを解析
      const [val, next] = parseBlock(lines, i + 1, myIndent);
      obj[key] = val;
      i = next;
    } else if (rest.startsWith('[') && rest.endsWith(']')) {
      // インラインリスト [a, b, c]
      const inner = rest.slice(1, -1);
      const items = inner
        .split(',')
        .map((s) => parseScalar(s.trim()))
        .filter((v) => v !== null) as YamlValue[];
      obj[key] = items;
      i++;
    } else {
      // スカラー
      obj[key] = parseScalar(rest);
      i++;
    }
  }
  // 結果
  return [obj, i];
}

/** リストを解析する */
function parseList(
  lines: { indent: number; text: string }[],
  start: number,
  myIndent: number
): [YamlValue[], number] {
  // 結果配列
  const arr: YamlValue[] = [];
  // カーソル
  let i = start;
  // 同レベル "- " 行を順に処理
  // noUncheckedIndexedAccess に対応するためループ内で line を取得し undefined ガードする
  while (i < lines.length) {
    const line = lines[i];
    if (!line || line.indent !== myIndent) break;
    if (!(line.text.startsWith('- ') || line.text === '-')) break;
    const text = line.text;
    if (text === '-') {
      // 子ブロック
      const [val, next] = parseBlock(lines, i + 1, myIndent);
      arr.push(val);
      i = next;
    } else {
      // "- " の後ろを取得
      const afterDash = text.slice(2);
      if (afterDash.includes(':')) {
        // インラインマップ "- key: val\n  key2: val2"
        // 最初のキー＝値ペアを擬似行に変換し、追加マップ部分は通常パースに委ねる
        const synth: { indent: number; text: string }[] = [
          { indent: myIndent + 2, text: afterDash }
        ];
        // 続く同インデント行（"  key: ..."）を取り込む
        let j = i + 1;
        while (j < lines.length) {
          const nl = lines[j];
          if (!nl || nl.indent !== myIndent + 2) break;
          synth.push(nl);
          j++;
        }
        const [val] = parseMap(synth, 0, myIndent + 2);
        arr.push(val);
        i = j;
      } else {
        // 純粋スカラー要素
        arr.push(parseScalar(afterDash));
        i++;
      }
    }
  }
  // 結果
  return [arr, i];
}

/** スカラーを解釈する */
function parseScalar(s: string): YamlValue {
  // クォート文字列
  if ((s.startsWith('"') && s.endsWith('"')) || (s.startsWith("'") && s.endsWith("'"))) {
    return s.slice(1, -1);
  }
  // 真偽
  if (s === 'true') return true;
  if (s === 'false') return false;
  // null
  if (s === 'null' || s === '~' || s === '') return null;
  // 数値
  const n = Number(s);
  if (!Number.isNaN(n) && /^-?[\d.]+$/.test(s)) return n;
  // それ以外は文字列
  return s;
}

// ---------------------------------------------------------------------
// YAML → Flow 変換
// ---------------------------------------------------------------------

/**
 * YAML 文字列を Flow Aggregate に変換する。
 *
 * @throws YAML 構造が `examples/flow-templates/` のスキーマと整合しない場合
 */
export function flowFromYaml(yaml: string): Flow {
  // YAML を中間ツリーへ
  const tree = parseYaml(yaml);
  // ルート flow キーを取り出す
  if (tree === null || typeof tree !== 'object' || Array.isArray(tree)) {
    throw new Error('YAML ルートはオブジェクトである必要があります');
  }
  const root = tree as { [k: string]: YamlValue };
  const flowRaw = root['flow'];
  if (!flowRaw || typeof flowRaw !== 'object' || Array.isArray(flowRaw)) {
    throw new Error('YAML ルートに `flow:` キーが必要です');
  }
  const flow = flowRaw as { [k: string]: YamlValue };
  // 必須フィールド
  const id = stringField(flow, 'id');
  const name = stringField(flow, 'name');
  const industry = optionalString(flow, 'industry');
  const version = optionalNumber(flow, 'schema_version') ?? 1;
  // ノード／辺
  const nodesRaw = flow['nodes'];
  const edgesRaw = flow['edges'];
  if (!Array.isArray(nodesRaw)) {
    throw new Error('flow.nodes はリストである必要があります');
  }
  const nodes: FlowNode[] = nodesRaw.map(toNode);
  const edges: FlowEdge[] = Array.isArray(edgesRaw) ? edgesRaw.map(toEdge) : [];
  // Aggregate を構築（業界・バージョン同梱）
  return Flow.create(id, name, nodes, edges, industry, version);
}

/** YamlValue → FlowNode */
function toNode(raw: YamlValue): FlowNode {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    throw new Error('node はオブジェクトである必要があります');
  }
  const o = raw as { [k: string]: YamlValue };
  const kind = stringField(o, 'kind') as FlowNodeKind;
  // 妥当性
  if (!['start', 'step', 'decision', 'parallel', 'end'].includes(kind)) {
    throw new Error(`不正なノード種別: ${kind}`);
  }
  // 戻り値の組み立て（exactOptionalPropertyTypes に従い undefined フィールドを含めない）
  const completion = optionalString(o, 'completion_criteria') as
    | FlowCompletionCriteria
    | undefined;
  const stdTime = optionalNumber(o, 'standard_time_seconds');
  const eSig = optionalString(o, 'e_signature') as 'required' | 'optional' | undefined;
  const auditReq = optionalBool(o, 'audit_required');
  const ccp = optionalBool(o, 'critical_control_point');
  const sev = optionalNumber(o, 'andon_severity');
  const node: FlowNode = {
    id: stringField(o, 'id'),
    kind,
    label: stringField(o, 'label'),
    ...(completion !== undefined ? { completion_criteria: completion } : {}),
    ...(stdTime !== undefined ? { standard_time_seconds: stdTime } : {}),
    ...(eSig !== undefined ? { e_signature: eSig } : {}),
    ...(auditReq !== undefined ? { audit_required: auditReq } : {}),
    ...(ccp !== undefined ? { critical_control_point: ccp } : {}),
    ...(sev !== undefined ? { andon_severity: sev } : {})
  };
  return node;
}

/** YamlValue → FlowEdge */
function toEdge(raw: YamlValue): FlowEdge {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    throw new Error('edge はオブジェクトである必要があります');
  }
  const o = raw as { [k: string]: YamlValue };
  // 同じく undefined を含めない
  const cond = optionalString(o, 'condition');
  return {
    from: stringField(o, 'from'),
    to: stringField(o, 'to'),
    ...(cond !== undefined ? { condition: cond } : {})
  };
}

// ---------------------------------------------------------------------
// 補助
// ---------------------------------------------------------------------
function stringField(o: { [k: string]: YamlValue }, key: string): string {
  const v = o[key];
  if (typeof v !== 'string') {
    throw new Error(`必須フィールド ${key} が文字列ではありません`);
  }
  return v;
}

function optionalString(o: { [k: string]: YamlValue }, key: string): string | undefined {
  const v = o[key];
  return typeof v === 'string' ? v : undefined;
}

function optionalNumber(o: { [k: string]: YamlValue }, key: string): number | undefined {
  const v = o[key];
  return typeof v === 'number' ? v : undefined;
}

function optionalBool(o: { [k: string]: YamlValue }, key: string): boolean | undefined {
  const v = o[key];
  return typeof v === 'boolean' ? v : undefined;
}
