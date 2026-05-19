-- V20260517120027__create_evidence_files_table.sql
-- TBL-009 evidence_files: 証拠ファイルメタデータ（Append-only）。バイナリは NAS 保存。

-- EN-013 EvidenceFile — 証拠ファイルメタデータ（Append-only）。バイナリは NAS 保存。
CREATE TABLE IF NOT EXISTS evidence_files (
    -- 証拠ファイル識別子。UUID v4。gen_random_uuid() で自動生成。
    evidence_id       UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 紐付く作業イベントの識別子。work_events への DEFERRABLE 外部キー。
    event_id          UUID        NOT NULL,
    -- ファイル種別。PHOTO / AUDIO / DOCUMENT / VIDEO の 4 種のみ許可。
    file_type         VARCHAR(16) NOT NULL,
    -- NAS 上の相対パス。形式: {year}/{month}/{uuid}.{ext}。
    file_path         TEXT        NOT NULL,
    -- SHA-256 ハッシュ（64 文字 hex）。改ざん検知および重複排除に使用する。ALCOA+ Original 要件。
    file_hash         CHAR(64)    NOT NULL,
    -- ファイルサイズ（バイト）。0 より大きい値のみ許可（CHECK 制約）。
    file_size_bytes   INTEGER     NOT NULL,
    -- MIME タイプ。例: image/jpeg, audio/wav, application/pdf, video/mp4。
    mime_type         VARCHAR(64) NOT NULL,
    -- クライアント側の撮影・作成時刻（ALCOA+ Contemporaneous 補足情報）。
    captured_at       TIMESTAMPTZ NOT NULL,
    -- TRUE = Exif（位置情報・機器情報）削除済み。プライバシー保護のため撮影直後に除去する。
    exif_stripped     BOOLEAN     NOT NULL DEFAULT TRUE,
    -- サーバー受信時刻。uploaded_at に相当する。IDX-019 のインデックス対象列。
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 主キー
    CONSTRAINT pk_evidence_files PRIMARY KEY (evidence_id),
    -- work_events への外部キー。パーティション化されたテーブルへの参照のため DEFERRABLE INITIALLY DEFERRED。
    CONSTRAINT fk_evidence_files_event FOREIGN KEY (event_id, captured_at)
        REFERENCES work_events (event_id, timestamp_server) ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    -- file_type は 4 種の列挙値のみ許可する。
    CONSTRAINT ck_evidence_files_type CHECK (
        file_type IN ('PHOTO', 'AUDIO', 'DOCUMENT', 'VIDEO')
    ),
    -- ファイルサイズは 0 より大きい値のみ許可する。
    CONSTRAINT ck_evidence_files_size_positive CHECK (file_size_bytes > 0),
    -- file_hash は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_evidence_files_hash_length CHECK (length(file_hash) = 64)
);

COMMENT ON TABLE  evidence_files IS 'EN-013 EvidenceFile — 証拠ファイルメタデータ。Append-only。バイナリは NAS /evidence/ 配下に UUID 命名で保存。7年以上保存。';
COMMENT ON COLUMN evidence_files.file_path    IS 'NAS 上の相対パス。形式: {year}/{month}/{uuid}.{ext}。バイナリを DB に保存しない設計（NFR-PRF-015 対応）。';
COMMENT ON COLUMN evidence_files.file_hash    IS 'SHA-256 ハッシュ（64 文字 hex）。改ざん検知および重複排除に使用する。ALCOA+ Original 要件。';
COMMENT ON COLUMN evidence_files.exif_stripped IS 'TRUE = Exif（位置情報・機器情報）削除済み。プライバシー保護のため撮影直後に除去する（アプリ層で処理）。';
COMMENT ON COLUMN evidence_files.created_at   IS 'サーバー受信時刻。captured_at がクライアント側の撮影時刻であるのに対し、こちらはサーバーへのアップロード受信時刻を示す。IDX-019 のインデックス対象列（06_インデックス §1 準拠）。';

-- Append-only 強制: app_event_writer ロールから UPDATE/DELETE を剥奪する
REVOKE UPDATE, DELETE ON evidence_files FROM PUBLIC;
REVOKE UPDATE, DELETE ON evidence_files FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON evidence_files TO app_event_writer;
