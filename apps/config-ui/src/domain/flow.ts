// 対応 §: ロードマップ §10.2 §10.2.1 §3.6.3 §28
// 設定 UI 側の Flow Aggregate（作業フロー定義）。

/**
 * フローノード種別。HSM 状態の対応関係は §3.4.1 と整合する。
 */
export type FlowNodeKind = 'start' | 'step' | 'decision' | 'parallel' | 'end';

/** 完了条件種別（§3.1.1） */
export type FlowCompletionCriteria = 'manual' | 'photo';

/** 編集中フローのストレス予測スコア（§9.2.1 5 軸／§10.2.3） */
export interface FlowStressScore {
  // 認知負荷
  readonly cognitive_load: number;
  // 時間圧
  readonly time_pressure: number;
  // 不確実性
  readonly uncertainty: number;
  // 自律性喪失
  readonly autonomy_loss: number;
  // 失敗コスト
  readonly failure_cost: number;
}

/** SMED 段取り種別（§9.3.1） */
export type SmedType = 'internal' | 'external';

/** CCP 関連の数値しきい値（§12 HACCP／§3.1.5.4 業界別） */
export interface FlowCcpThresholds {
  // 加熱中心温度
  readonly min_core_temperature_c?: number;
  // 加熱保持秒数
  readonly min_holding_seconds?: number;
  // 冷却最大秒数
  readonly max_cooling_seconds?: number;
  // 冷却到達温度
  readonly target_temperature_c?: number;
  // 保管温度上限
  readonly max_storage_temperature_c?: number;
  // ESD 帯抵抗（電子）
  readonly min_resistance_ohms?: number;
  readonly max_resistance_ohms?: number;
  // SMT リフロー ピーク温度
  readonly peak_temperature_c?: number;
}

/** フローノード */
export interface FlowNode {
  // ノード ID（フロー内一意）
  readonly id: string;
  // ノード種別
  readonly kind: FlowNodeKind;
  // 表示名（UI で使用、§28 用語と整合）
  readonly label: string;
  // 完了条件（§3.1.1）
  readonly completion_criteria?: FlowCompletionCriteria;
  // 標準時間（秒、§9.3.1 タクトタイム）
  readonly standard_time_seconds?: number;
  // ストレス予測スコア（§10.2.3）
  readonly stress?: FlowStressScore;
  // 電子署名要否（GMP §10.5）
  readonly e_signature?: 'required' | 'optional';
  // 監査必須（§11.4.1）
  readonly audit_required?: boolean;
  // CCP 指定（HACCP §12）
  readonly critical_control_point?: boolean;
  // CCP 数値しきい値
  readonly ccp_thresholds?: FlowCcpThresholds;
  // SMED 段取り種別
  readonly smed?: { type: SmedType };
  // §17 アドオン capability 要請
  readonly capabilities_required?: ReadonlyArray<string>;
  // アンドン重大度（1〜5、§9.3.4 Andon 5 段階）
  readonly andon_severity?: number;
  // MES 連携指示（§10.3.2）
  readonly mes_integration?: { action: string; endpoint?: string };
}

/** フロー辺（ノード間遷移） */
export interface FlowEdge {
  // 出発ノード ID
  readonly from: string;
  // 到着ノード ID
  readonly to: string;
  // 条件式（オプション、決定ノードから出る辺で使用）
  readonly condition?: string;
}

/**
 * Flow Aggregate（§10.2.1 フロー編集）
 *
 * 不変条件:
 * - ノード ID はフロー内で一意
 * - 開始ノード（kind === 'start'）は 1 つ以上
 */
export class Flow {
  // 識別子
  readonly id: string;
  // 表示名
  readonly name: string;
  // バージョン番号（SemVer の MINOR 相当）
  readonly version: number;
  // 業界（§3.1.5.4／§10.2.1 業界別テンプレ）
  readonly industry?: string;
  // ノード集合
  private readonly _nodes: ReadonlyArray<FlowNode>;
  // 辺集合
  private readonly _edges: ReadonlyArray<FlowEdge>;

  // private コンストラクタ
  private constructor(
    id: string,
    name: string,
    version: number,
    nodes: ReadonlyArray<FlowNode>,
    edges: ReadonlyArray<FlowEdge>,
    industry?: string
  ) {
    // 値を保持する（exactOptionalPropertyTypes に従い undefined を入れない）
    this.id = id;
    this.name = name;
    this.version = version;
    this._nodes = nodes;
    this._edges = edges;
    if (industry !== undefined) {
      this.industry = industry;
    }
  }

  /** 新規 Flow を生成する */
  static create(
    id: string,
    name: string,
    nodes: ReadonlyArray<FlowNode>,
    edges: ReadonlyArray<FlowEdge>,
    industry?: string,
    version: number = 1
  ): Flow {
    // 不変条件を検査
    Flow.validate(nodes, edges);
    // 妥当値で構築
    return new Flow(id, name, version, nodes, edges, industry);
  }

  /** ノード一覧を取得する（不変参照） */
  get nodes(): ReadonlyArray<FlowNode> {
    // 内部配列を返す
    return this._nodes;
  }

  /** 辺一覧を取得する（不変参照） */
  get edges(): ReadonlyArray<FlowEdge> {
    // 内部配列を返す
    return this._edges;
  }

  /** ノード数を取得する */
  get nodeCount(): number {
    // 内部配列の長さ
    return this._nodes.length;
  }

  /** 辺数を取得する */
  get edgeCount(): number {
    // 内部配列の長さ
    return this._edges.length;
  }

  /** 不変条件チェック */
  private static validate(
    nodes: ReadonlyArray<FlowNode>,
    edges: ReadonlyArray<FlowEdge>
  ): void {
    // ノード ID の重複検出
    const ids = new Set<string>();
    for (const n of nodes) {
      // 重複チェック
      if (ids.has(n.id)) {
        // ドメインエラー
        throw new Error(`ノード ID が重複しています: ${n.id}`);
      }
      // 集合に追加
      ids.add(n.id);
    }

    // 開始ノードが 1 つ以上存在すること
    const hasStart = nodes.some((n) => n.kind === 'start');
    if (!hasStart) {
      // ドメインエラー
      throw new Error('開始ノード（kind=start）が存在しません');
    }

    // 辺の参照整合性
    for (const e of edges) {
      // from ノードの存在チェック
      if (!ids.has(e.from)) {
        throw new Error(`辺の from ノードが存在しません: ${e.from}`);
      }
      // to ノードの存在チェック
      if (!ids.has(e.to)) {
        throw new Error(`辺の to ノードが存在しません: ${e.to}`);
      }
    }
  }
}
