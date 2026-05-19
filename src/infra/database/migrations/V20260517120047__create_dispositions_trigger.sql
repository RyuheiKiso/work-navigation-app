-- V20260517120047__create_dispositions_trigger.sql
-- dispositions テーブルの Two-Person Integrity を保証する PL/pgSQL 関数 + BEFORE INSERT トリガ。

-- NFR-SEC-048: Two-Person Integrity（TPI）— 同一署名者による quality_admin と supervisor の兼任を禁止する。
-- 電子サイン ID レベルのチェック（ck_disp_distinct_signs）に加え、
-- 署名者（signer_id）レベルの検証をトリガで実施する。
CREATE OR REPLACE FUNCTION check_disposition_distinct_signers()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    -- 品質管理者署名の signer_id（users.user_id）を格納する変数
    qa_signer_id  UUID;
    -- 監督者署名の signer_id（users.user_id）を格納する変数
    sup_signer_id UUID;
BEGIN
    -- quality_admin 署名者の user_id を取得する
    SELECT signer_id INTO qa_signer_id
    FROM electronic_signs
    WHERE sign_id = NEW.quality_admin_sign_id;

    -- supervisor 署名者の user_id を取得する
    SELECT signer_id INTO sup_signer_id
    FROM electronic_signs
    WHERE sign_id = NEW.supervisor_sign_id;

    -- 同一署名者による Two-Person Integrity 違反を検出する（NFR-SEC-048）
    IF qa_signer_id = sup_signer_id THEN
        RAISE EXCEPTION 'ERR-BIZ-021: disposition requires two distinct signers (quality_admin and supervisor must be different persons)';
    END IF;

    RETURN NEW;
END;
$$;

COMMENT ON FUNCTION check_disposition_distinct_signers() IS 'NFR-SEC-048 Two-Person Integrity 検証トリガ関数。dispositions INSERT 時に quality_admin と supervisor の署名者（signer_id）が異なることを保証する。違反時は ERR-BIZ-021 を RAISE する。';

-- dispositions テーブルへの INSERT 前に Two-Person Integrity を検証するトリガを作成する
CREATE TRIGGER trg_disposition_distinct_signers
    BEFORE INSERT ON dispositions
    FOR EACH ROW
    EXECUTE FUNCTION check_disposition_distinct_signers();

COMMENT ON TRIGGER trg_disposition_distinct_signers ON dispositions IS 'dispositions INSERT 前に check_disposition_distinct_signers() を呼び出す。Two-Person Integrity（NFR-SEC-048）を DB レベルで強制する。';
