import { z } from 'zod';

// 共通基底スキーマ群（docs/05/06 §04_入力バリデーション仕様.md §1 のバリデーション種別マスタに対応）

export const uuidV7Schema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
    'UUID v7 形式が不正です',
  );

export const userIdSchema = z
  .string()
  .min(1, 'ユーザー ID を入力してください')
  .max(128, 'ユーザー ID は 128 文字以内で入力してください');

export const usernameSchema = z
  .string()
  .min(1, 'ユーザー名は必須です')
  .max(64, 'ユーザー名は 64 文字以内で入力してください')
  .regex(/^[A-Za-z0-9_-]+$/, 'ユーザー名は英数字・アンダースコア・ハイフンのみ使用できます');

export const passwordSchema = z
  .string()
  .min(8, 'パスワードは 8 文字以上で入力してください')
  .max(128, 'パスワードは 128 文字以内で入力してください');

// 電子サイン PIN は 4〜8 桁の数字のみ（docs/05/06 §04 §1 PIN ルール）
export const pinSchema = z
  .string()
  .regex(/^\d{4,8}$/, 'PIN は 4〜8 桁の数字で入力してください');

export const sopCodeSchema = z
  .string()
  .min(1)
  .max(64)
  .regex(/^[A-Za-z0-9_-]+$/, 'SOP コードは英数字・アンダースコア・ハイフンのみ使用できます');

export const processCodeSchema = z
  .string()
  .min(1)
  .max(64)
  .regex(/^[A-Za-z0-9_]+$/, 'プロセスコードは英数字・アンダースコアのみ使用できます');

export const operationCodeSchema = z
  .string()
  .min(1)
  .max(64)
  .regex(/^[A-Za-z0-9_]+$/, 'オペレーションコードは英数字・アンダースコアのみ使用できます');

export const productCodeSchema = z
  .string()
  .min(1)
  .max(64)
  .regex(/^[A-Za-z0-9_-]+$/, '製品コードの形式が不正です');

export const materialCodeSchema = z
  .string()
  .min(1)
  .max(64)
  .regex(/^[A-Za-z0-9_-]+$/, '材料コードの形式が不正です');

export const supplierCodeSchema = z
  .string()
  .min(1)
  .max(64)
  .regex(/^[A-Za-z0-9_-]+$/, '仕入先コードの形式が不正です');

// AQL 許容値（ANSI/ASQ Z1.4 標準値）
const ALLOWED_AQL_VALUES = [0.10, 0.25, 0.40, 0.65, 1.0, 1.5, 2.5, 4.0, 6.5, 10.0] as const;

export const aqlValueSchema = z
  .number()
  .refine((v) => (ALLOWED_AQL_VALUES as readonly number[]).includes(v), {
    message: 'AQL 値は 0.10, 0.25, 0.40, 0.65, 1.0, 1.5, 2.5, 4.0, 6.5, 10.0 のいずれかである必要があります',
  });

export const receivedQuantitySchema = z
  .number()
  .int('受入数量は整数で入力してください')
  .positive('受入数量は正の数で入力してください');

// 電子サインの非空検証は SVG ストロークなど任意文字列でも長さでチェックする
export const electronicSignContentSchema = z
  .string()
  .min(1, '電子サインは必須です');

export const measurementValueSchema = (lsl?: number, usl?: number): z.ZodType<number> => {
  let base: z.ZodType<number> = z.number().finite('測定値は有限数で入力してください');
  if (lsl !== undefined) {
    base = base.refine((v) => v >= lsl, {
      message: `測定値は LSL (${lsl}) 以上で入力してください`,
    });
  }
  if (usl !== undefined) {
    base = base.refine((v) => v <= usl, {
      message: `測定値は USL (${usl}) 以下で入力してください`,
    });
  }
  return base;
};

// GS1 AI (01) で始まる 14 桁 GTIN を検証する
export const gs1Schema = z
  .string()
  .regex(/^\(01\)\d{14}/, 'GS1 形式が不正です');

// UCUM コード形式（英数字 + / [ ] { } .）
export const ucumCodeSchema = z
  .string()
  .min(1)
  .max(64)
  .regex(/^[A-Za-z0-9/\[\]{}.]+$/, 'UCUM コードの形式が不正です');

// 多言語テキスト（ja は必須、en/zh は空文字許容）
export const localizedTextSchema = z.object({
  ja: z.string().min(1, '日本語は必須です'),
  en: z.string(),
  zh: z.string(),
});

export const localeSchema = z.enum(['ja', 'en', 'zh']);

export const userRoleSchema = z.enum([
  'operator',
  'supervisor',
  'quality_admin',
  'master_admin',
  'system_admin',
  'executive',
]);

export const sopStatusSchema = z.enum(['draft', 'in_review', 'published', 'deprecated']);

export const qcStatusSchema = z.enum([
  'PENDING',
  'SAMPLING',
  'INSPECTING',
  'PASSED',
  'FAILED',
  'REJECTED',
  'CONDITIONAL_PASS',
  'SCREENING_REQUIRED',
  'SCRAPPED',
  'RETURNED_TO_VENDOR',
]);

// JSON Logic DSL のホワイトリスト演算子（docs/05/06 §04 §2-2）
const JSON_LOGIC_ALLOWED_OPERATORS = new Set([
  '==', '===', '!=', '!==',
  '<', '<=', '>', '>=',
  '!', '!!',
  'and', 'or',
  'if',
  'var',
  'in',
  'cat',
  'substr',
  '+', '-', '*', '/', '%',
  'min', 'max',
  'merge',
  'all', 'none', 'some', 'filter', 'map', 'reduce',
  'missing', 'missing_some',
]);

export const JSON_LOGIC_MAX_NEST = 5;

// JSON Logic DSL のホワイトリスト演算子と最大ネスト深度を再帰検証する
export function validateJsonLogicDsl(rule: unknown, depth = 0): boolean {
  if (depth > JSON_LOGIC_MAX_NEST) return false;
  if (typeof rule !== 'object' || rule === null) return true;
  if (Array.isArray(rule)) {
    return rule.every((item) => validateJsonLogicDsl(item, depth + 1));
  }
  const keys = Object.keys(rule as Record<string, unknown>);
  if (keys.length !== 1) return false;
  const operator = keys[0]!;
  if (!JSON_LOGIC_ALLOWED_OPERATORS.has(operator)) return false;
  const operands = (rule as Record<string, unknown>)[operator];
  return validateJsonLogicDsl(operands, depth + 1);
}

export const jsonLogicRuleSchema = z
  .record(z.unknown())
  .refine((v) => validateJsonLogicDsl(v), {
    message: 'JSON Logic DSL の演算子またはネスト深度が許容外です',
  });

// 認証ログイン用スキーマ
export const authLoginSchema = z.object({
  loginId: userIdSchema,
  password: passwordSchema,
  deviceId: uuidV7Schema,
  factoryId: uuidV7Schema,
});

// 作業実行作成用スキーマ
export const createWorkExecutionSchema = z.object({
  workOrderId: uuidV7Schema,
  operatorId: uuidV7Schema,
  deviceId: uuidV7Schema,
  startTimestampClient: z.string().datetime(),
});

// 受入検査作成用スキーマ
export const createIncomingInspectionSchema = z.object({
  lotId: uuidV7Schema,
  supplierId: uuidV7Schema,
  materialId: uuidV7Schema,
  receivedQty: receivedQuantitySchema,
});

// 電子サイン作成用スキーマ
export const createElectronicSignSchema = z.object({
  signerId: uuidV7Schema,
  signedContentHash: z.string().regex(/^sha256:[0-9a-f]{64}$/, 'SHA-256 ハッシュ形式が不正です'),
  pinHash: z.string().regex(/^\$2[ab]\$\d{2}\$.+$/, 'bcrypt ハッシュ形式が不正です'),
  contextType: z.enum(['step_sign', 'work_complete_sign', 'approval_sign', 'quality_check_sign']),
  contextId: uuidV7Schema,
  stepId: uuidV7Schema.optional(),
  timestampClient: z.string().datetime(),
  deviceSignature: z.string().min(1),
});
