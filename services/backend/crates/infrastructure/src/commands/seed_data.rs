//! デモシードの固定データ仕様
//!
//! 対応 §: ロードマップ §14.2 §10.2.1
//!
//! ロジックを持たない宣言的なデータのみを保持する。
//! `seed.rs` は本モジュールの値を adapter の upsert API に渡すだけにし、
//! 1 ファイル ≤ 500 行制約を満たす。

/// 投入対象のデモユーザ
pub struct DemoUser {
    /// ログイン ID
    pub user_id: &'static str,
    /// 表示名（既存 scripts/seed-demo.sh の慣習に揃える）
    pub display_name: &'static str,
}

/// すべての preset で投入する 3 ユーザ。
/// パスワードは全員 `hello-world` で、i18n 文言と一致させる。
pub const USERS: &[DemoUser] = &[
    DemoUser { user_id: "alice",   display_name: "Alice Operator" },
    DemoUser { user_id: "bob",     display_name: "Bob 班長" },
    DemoUser { user_id: "charlie", display_name: "Charlie 生産技術" },
];

/// すべての seed ユーザに割り当てる平文パスワード
pub const SEED_PASSWORD: &str = "hello-world";

/// マスタ行（products／equipments／parts に共通の三つ組）
pub struct MasterEntry {
    /// PK となるコード
    pub code: &'static str,
    /// 表示名
    pub name: &'static str,
    /// industry / location / unit のいずれか（テーブルにより意味が異なる）
    pub extra: Option<&'static str>,
}

/// 製品マスタ
pub const PRODUCTS: &[MasterEntry] = &[
    MasterEntry { code: "P-A001", name: "ベアリングユニット A1",   extra: Some("機械加工") },
    MasterEntry { code: "P-A002", name: "ベアリングユニット A2",   extra: Some("機械加工") },
    MasterEntry { code: "P-B100", name: "センサーモジュール B100", extra: Some("電子組立") },
];

/// 設備マスタ
pub const EQUIPMENTS: &[MasterEntry] = &[
    MasterEntry { code: "EQ-LINE-1", name: "組立ライン 1", extra: Some("F1 棟 1F") },
    MasterEntry { code: "EQ-LINE-2", name: "組立ライン 2", extra: Some("F1 棟 1F") },
    MasterEntry { code: "EQ-INSP-1", name: "検査台 1",     extra: Some("F1 棟 2F") },
];

/// 部材マスタ
pub const PARTS: &[MasterEntry] = &[
    MasterEntry { code: "PT-BRG-001", name: "玉軸受 6204",  extra: Some("個") },
    MasterEntry { code: "PT-BLT-M6",  name: "M6 ボルト",   extra: Some("本") },
    MasterEntry { code: "PT-FRM-A",   name: "フレーム A",   extra: Some("個") },
    MasterEntry { code: "PT-OIL",     name: "グリス",       extra: Some("g") },
    MasterEntry { code: "PT-LBL-A",   name: "ラベル A",     extra: Some("枚") },
];

/// デモフロー定義（最小 `ReactFlow` JSON）
pub struct DemoFlow {
    /// フロー ID
    pub id: &'static str,
    /// バージョン（draft → trial → production の単調増加）
    pub version: i32,
    /// 表示名
    pub name: &'static str,
    /// 業種タグ
    pub industry: Option<&'static str>,
    /// `draft` / `trial` / `production` / `archived`
    pub status: &'static str,
    /// `nodes`/`edges` を含む `ReactFlow` JSON 文字列
    pub body_json: &'static str,
}

/// デモフロー一覧
pub const FLOWS: &[DemoFlow] = &[
    DemoFlow {
        id: "FL-ASSY-A",
        version: 1,
        name: "ベアリング組立フロー",
        industry: Some("機械加工"),
        status: "production",
        // ReactFlow 互換の最小ノード／エッジ定義（組立 → 検査 → 梱包）
        body_json: r#"{
            "nodes": [
                {"id":"step01","type":"step","position":{"x":50,"y":80},"data":{"label":"部品組立"}},
                {"id":"step02","type":"step","position":{"x":250,"y":80},"data":{"label":"外観検査"}},
                {"id":"step03","type":"step","position":{"x":450,"y":80},"data":{"label":"梱包"}}
            ],
            "edges": [
                {"id":"e1","source":"step01","target":"step02"},
                {"id":"e2","source":"step02","target":"step03"}
            ]
        }"#,
    },
];

/// デモタスク（Aggregate ルートに永続化する値の最小セット）
pub struct DemoTask {
    /// タスク ID
    pub id: &'static str,
    /// `TaskState::from_label` が解釈できる文字列
    pub state_label: &'static str,
    /// 主体端末 ID
    pub device_id: &'static str,
    /// Lamport クロック（保存時 i64 へキャスト）
    pub lamport: u64,
    /// `manual` / `photo`
    pub completion_criteria: &'static str,
    /// 班長監視ビューに表示するタイトル
    pub title: &'static str,
    /// 紐付くフロー ID
    pub flow_id: &'static str,
    /// 担当者 `user_id`
    pub responsible_user: &'static str,
    /// 現在ステップ ID（`None` なら未着手）
    pub current_step_id: Option<&'static str>,
}

/// デモタスク一覧（4 件で Idle/Ready/Running/Completed を網羅）
pub const TASKS: &[DemoTask] = &[
    DemoTask {
        id: "T-DEMO-001",
        state_label: "Running",
        device_id: "demo-terminal-01",
        lamport: 5,
        completion_criteria: "manual",
        title: "ベアリングユニット A1 組立 #001",
        flow_id: "FL-ASSY-A",
        responsible_user: "alice",
        current_step_id: Some("step02"),
    },
    DemoTask {
        id: "T-DEMO-002",
        state_label: "Ready",
        device_id: "demo-terminal-01",
        lamport: 2,
        completion_criteria: "photo",
        title: "ベアリングユニット A2 組立 #002",
        flow_id: "FL-ASSY-A",
        responsible_user: "bob",
        current_step_id: None,
    },
    DemoTask {
        id: "T-DEMO-003",
        state_label: "Completed",
        device_id: "demo-terminal-01",
        lamport: 9,
        completion_criteria: "manual",
        title: "ベアリングユニット A1 組立 #000",
        flow_id: "FL-ASSY-A",
        responsible_user: "alice",
        current_step_id: Some("step03"),
    },
    DemoTask {
        id: "T-DEMO-004",
        state_label: "Idle",
        device_id: "demo-terminal-02",
        lamport: 0,
        completion_criteria: "manual",
        title: "センサーモジュール B100 組立 #001",
        flow_id: "FL-ASSY-A",
        responsible_user: "charlie",
        current_step_id: None,
    },
];

/// 各タスクに割り当てるステップ定義（共通）
pub struct DemoStep {
    /// ステップ ID（フローの node.id に一致）
    pub id: &'static str,
    /// 並び順（1 始まり）
    pub sequence: i32,
    /// 表示ラベル
    pub label: &'static str,
    /// `manual` / `photo`
    pub completion_criteria: &'static str,
    /// 標準作業時間（秒）
    pub standard_time_seconds: i32,
}

/// デモステップ一覧（組立 → 検査 → 梱包）
pub const STEPS: &[DemoStep] = &[
    DemoStep { id: "step01", sequence: 1, label: "部品組立", completion_criteria: "manual", standard_time_seconds: 180 },
    DemoStep { id: "step02", sequence: 2, label: "外観検査", completion_criteria: "photo",  standard_time_seconds: 120 },
    DemoStep { id: "step03", sequence: 3, label: "梱包",     completion_criteria: "manual", standard_time_seconds: 60  },
];
