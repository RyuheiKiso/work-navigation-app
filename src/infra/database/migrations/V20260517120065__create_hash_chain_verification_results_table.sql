-- V20260517120065__create_hash_chain_verification_results_table.sql
-- hash_chain_verification_results: TBL-031 補助テーブル。週次ハッシュチェーン検証結果の記録。
-- 権威: docs/05_詳細設計/07_アルゴリズム詳細設計/03_ハッシュチェーンアルゴリズム詳細.md §6
--       docs/05_詳細設計/07_アルゴリズム詳細設計/06_バッチジョブ処理詳細.md §2 BAT-001
-- BAT-001（毎週月曜 03:00, wnav_master_api 内 tokio task）が INSERT する。
-- 同一週の複数実行は最初の 1 回のみ有効（週次 UNIQUE インデックスで冪等性を保証する）。

-- EN-025 HashChainBlock 補助 — ハッシュチェーン週次検証結果
CREATE TABLE IF NOT EXISTS hash_chain_verification_results (
    -- 検証結果識別子。UUID v4。gen_random_uuid() で自動生成。
    id                   UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- 検証実行日時。
    verified_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- 検証対象のイベント件数。
    verified_count       BIGINT       NOT NULL,
    -- 検証ステータス。PASSED / FAILED / CORRECTED の 3 値のみ許可する。
    status               TEXT         NOT NULL,
    -- チェーン破断を最初に検知した hash_chain_blocks レコードの識別子。PASSED 時は NULL。
    broken_at_block_id   UUID         NULL,
    -- 補正ブロック（is_correction=TRUE）の event_id 配列。status=CORRECTED 時に設定する（ALG-025）。
    correction_block_ids UUID[]       NULL,
    -- エラーコード。FAILED 時に ERR-DB-003 等を格納する。
    error_code           TEXT         NULL,
    -- レコード作成日時。
    created_at           TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_hash_chain_verification_results PRIMARY KEY (id),
    -- hash_chain_blocks テーブルへの外部キー。破断ブロックの識別子。
    CONSTRAINT fk_hcvr_broken_block FOREIGN KEY (broken_at_block_id)
        REFERENCES hash_chain_blocks (block_id) ON DELETE RESTRICT,
    -- status は 3 値のみ許可する。
    CONSTRAINT ck_hcvr_status CHECK (status IN ('PASSED', 'FAILED', 'CORRECTED')),
    -- FAILED 時は broken_at_block_id が必須（どのブロックで破断したかを記録する）。
    CONSTRAINT ck_hcvr_failed_requires_broken_block CHECK (
        status <> 'FAILED' OR broken_at_block_id IS NOT NULL
    ),
    -- CORRECTED 時は correction_block_ids が必須。
    CONSTRAINT ck_hcvr_corrected_requires_ids CHECK (
        status <> 'CORRECTED' OR correction_block_ids IS NOT NULL
    )
);

COMMENT ON TABLE hash_chain_verification_results IS
    'TBL-031 補助テーブル — 週次ハッシュチェーン検証結果（BAT-001）。Append-only。同一週の 1 回のみ有効（週次 UNIQUE インデックス）。';
COMMENT ON COLUMN hash_chain_verification_results.id                   IS 'UUID v4（gen_random_uuid()）。';
COMMENT ON COLUMN hash_chain_verification_results.verified_at          IS '検証実行日時。週次 UNIQUE インデックスの基準列。';
COMMENT ON COLUMN hash_chain_verification_results.verified_count       IS '検証対象のイベント件数。';
COMMENT ON COLUMN hash_chain_verification_results.status               IS 'PASSED（整合）/ FAILED（破断検知）/ CORRECTED（補正済）の 3 値。';
COMMENT ON COLUMN hash_chain_verification_results.broken_at_block_id   IS 'FAILED 時: チェーン破断を最初に検知した hash_chain_blocks.block_id。';
COMMENT ON COLUMN hash_chain_verification_results.correction_block_ids IS 'CORRECTED 時: 補正ブロック（is_correction=TRUE）の event_id 配列（ALG-025）。';
COMMENT ON COLUMN hash_chain_verification_results.error_code           IS 'FAILED 時の内部エラーコード（例: ERR-DB-003）。';

-- 週次 UNIQUE インデックス: 同一週の複数実行を禁止する（BAT-001 冪等性保証）
-- DATE_TRUNC('week', verified_at) で週の開始日（月曜 00:00 UTC）に正規化する。
CREATE UNIQUE INDEX idx_hcvr_verified_week
    ON hash_chain_verification_results (DATE_TRUNC('week', verified_at));

COMMENT ON INDEX idx_hcvr_verified_week IS
    'BAT-001 冪等性保証用の週次 UNIQUE インデックス。同一週の複数実行は最初の 1 回のみ有効。';

-- Append-only 保証: UPDATE/DELETE を全ロールから REVOKE する（検証結果は不変）
REVOKE UPDATE, DELETE ON hash_chain_verification_results FROM PUBLIC;
REVOKE UPDATE, DELETE ON hash_chain_verification_results FROM app_event_writer;
REVOKE UPDATE, DELETE ON hash_chain_verification_results FROM app_read_write;

-- BAT-001 が書き込むため app_event_writer に INSERT/SELECT 権限を付与する
GRANT INSERT, SELECT ON hash_chain_verification_results TO app_event_writer;
