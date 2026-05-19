export type SeverityState = 'NORMAL' | 'TIGHTENED' | 'REDUCED';
export type InspectionLevel = 'I' | 'II' | 'III';

// ANSI/ASQ Z1.4 (= JIS Z 9015-1) 検査水準別のロットサイズ → サンプルサイズ記号
const SAMPLE_SIZE_CODE_TABLE: ReadonlyArray<{
  min: number;
  max: number;
  levels: Record<InspectionLevel, string>;
}> = [
  { min: 2, max: 8, levels: { I: 'A', II: 'A', III: 'B' } },
  { min: 9, max: 15, levels: { I: 'A', II: 'B', III: 'C' } },
  { min: 16, max: 25, levels: { I: 'B', II: 'C', III: 'D' } },
  { min: 26, max: 50, levels: { I: 'C', II: 'D', III: 'E' } },
  { min: 51, max: 90, levels: { I: 'C', II: 'E', III: 'F' } },
  { min: 91, max: 150, levels: { I: 'D', II: 'F', III: 'G' } },
  { min: 151, max: 280, levels: { I: 'E', II: 'G', III: 'H' } },
  { min: 281, max: 500, levels: { I: 'F', II: 'H', III: 'J' } },
  { min: 501, max: 1200, levels: { I: 'G', II: 'J', III: 'K' } },
  { min: 1201, max: 3200, levels: { I: 'H', II: 'K', III: 'L' } },
  { min: 3201, max: 10000, levels: { I: 'J', II: 'L', III: 'M' } },
  { min: 10001, max: 35000, levels: { I: 'K', II: 'M', III: 'N' } },
  { min: 35001, max: 150000, levels: { I: 'L', II: 'N', III: 'P' } },
  { min: 150001, max: 500000, levels: { I: 'M', II: 'P', III: 'Q' } },
  { min: 500001, max: Number.MAX_SAFE_INTEGER, levels: { I: 'N', II: 'Q', III: 'R' } },
];

// サンプルサイズ記号 → サンプル数 n（なみ検査）
export const SAMPLE_SIZE_BY_CODE: Readonly<Record<string, number>> = {
  A: 2,
  B: 3,
  C: 5,
  D: 8,
  E: 13,
  F: 20,
  G: 32,
  H: 50,
  J: 80,
  K: 125,
  L: 200,
  M: 315,
  N: 500,
  P: 800,
  Q: 1250,
  R: 2000,
};

// なみ検査の単一サンプリング Ac/Re 表（ANSI/ASQ Z1.4 Table II-A: AQL → サンプルサイズ記号 → [Ac, Re]）
const ACCEPT_REJECT_TABLE: Readonly<Record<string, Readonly<Record<string, readonly [number, number]>>>> = {
  '0.10': {
    A: [0, 1], B: [0, 1], C: [0, 1], D: [0, 1], E: [0, 1], F: [0, 1], G: [0, 1], H: [0, 1],
    J: [0, 1], K: [0, 1], L: [0, 1], M: [1, 2], N: [2, 3], P: [3, 4], Q: [5, 6], R: [7, 8],
  },
  '0.25': {
    A: [0, 1], B: [0, 1], C: [0, 1], D: [0, 1], E: [0, 1], F: [0, 1], G: [0, 1], H: [0, 1],
    J: [0, 1], K: [0, 1], L: [1, 2], M: [2, 3], N: [3, 4], P: [5, 6], Q: [7, 8], R: [10, 11],
  },
  '0.40': {
    A: [0, 1], B: [0, 1], C: [0, 1], D: [0, 1], E: [0, 1], F: [0, 1], G: [0, 1], H: [0, 1],
    J: [0, 1], K: [1, 2], L: [2, 3], M: [3, 4], N: [5, 6], P: [7, 8], Q: [10, 11], R: [14, 15],
  },
  '0.65': {
    A: [0, 1], B: [0, 1], C: [0, 1], D: [0, 1], E: [0, 1], F: [0, 1], G: [0, 1], H: [0, 1],
    J: [1, 2], K: [2, 3], L: [3, 4], M: [5, 6], N: [7, 8], P: [10, 11], Q: [14, 15], R: [21, 22],
  },
  '1.0': {
    A: [0, 1], B: [0, 1], C: [0, 1], D: [0, 1], E: [0, 1], F: [0, 1], G: [0, 1], H: [1, 2],
    J: [2, 3], K: [3, 4], L: [5, 6], M: [7, 8], N: [10, 11], P: [14, 15], Q: [21, 22], R: [21, 22],
  },
  '1.5': {
    A: [0, 1], B: [0, 1], C: [0, 1], D: [0, 1], E: [0, 1], F: [0, 1], G: [1, 2], H: [2, 3],
    J: [3, 4], K: [5, 6], L: [7, 8], M: [10, 11], N: [14, 15], P: [21, 22], Q: [21, 22], R: [21, 22],
  },
  '2.5': {
    A: [0, 1], B: [0, 1], C: [0, 1], D: [0, 1], E: [0, 1], F: [1, 2], G: [2, 3], H: [3, 4],
    J: [5, 6], K: [7, 8], L: [10, 11], M: [14, 15], N: [21, 22], P: [21, 22], Q: [21, 22], R: [21, 22],
  },
  '4.0': {
    A: [0, 1], B: [0, 1], C: [0, 1], D: [0, 1], E: [1, 2], F: [2, 3], G: [3, 4], H: [5, 6],
    J: [7, 8], K: [10, 11], L: [14, 15], M: [21, 22], N: [21, 22], P: [21, 22], Q: [21, 22], R: [21, 22],
  },
  '6.5': {
    A: [0, 1], B: [0, 1], C: [0, 1], D: [1, 2], E: [2, 3], F: [3, 4], G: [5, 6], H: [7, 8],
    J: [10, 11], K: [14, 15], L: [21, 22], M: [21, 22], N: [21, 22], P: [21, 22], Q: [21, 22], R: [21, 22],
  },
  '10.0': {
    A: [0, 1], B: [0, 1], C: [1, 2], D: [2, 3], E: [3, 4], F: [5, 6], G: [7, 8], H: [10, 11],
    J: [14, 15], K: [21, 22], L: [21, 22], M: [21, 22], N: [21, 22], P: [21, 22], Q: [21, 22], R: [21, 22],
  },
};

export interface SamplingPlanResult {
  sampleSizeCode: string;
  sampleSizeN: number;
  acceptNumberAc: number;
  rejectNumberRe: number;
  inspectionLevel: InspectionLevel;
  severityState: SeverityState;
  aqlValue: number;
}

// ロットサイズと検査水準からサンプルサイズ記号を導出する（Z1.4 表 I 互換）
export function resolveSampleSizeCode(lotSize: number, level: InspectionLevel): string {
  if (lotSize < 2) throw new Error('lotSize must be >= 2');
  for (const row of SAMPLE_SIZE_CODE_TABLE) {
    if (lotSize >= row.min && lotSize <= row.max) {
      return row.levels[level];
    }
  }
  return SAMPLE_SIZE_CODE_TABLE[SAMPLE_SIZE_CODE_TABLE.length - 1]!.levels[level];
}

// AQL とロットサイズから単一サンプリング計画（n, Ac, Re）を解決する
export function resolveSamplingPlan(
  lotSize: number,
  aqlValue: number,
  level: InspectionLevel = 'II',
  severityState: SeverityState = 'NORMAL',
): SamplingPlanResult {
  const code = resolveSampleSizeCode(lotSize, level);
  const aqlKey = aqlValue.toFixed(aqlValue < 1 ? 2 : 1);
  const aqlRow = ACCEPT_REJECT_TABLE[aqlKey];
  if (!aqlRow) {
    throw new Error(`AQL ${aqlValue} is not supported`);
  }
  const codeRow = aqlRow[code];
  if (!codeRow) {
    throw new Error(`Code ${code} not found for AQL ${aqlValue}`);
  }
  const sampleSizeN = SAMPLE_SIZE_BY_CODE[code];
  if (sampleSizeN === undefined) {
    throw new Error(`Sample size for code ${code} not defined`);
  }
  return {
    sampleSizeCode: code,
    sampleSizeN,
    acceptNumberAc: codeRow[0],
    rejectNumberRe: codeRow[1],
    inspectionLevel: level,
    severityState,
    aqlValue,
  };
}

export type AqlVerdict = 'PASSED' | 'REJECTED' | 'INSPECTING';

// 不良数と Ac/Re から AQL 合否判定を確定する（境界値: Ac 以下 PASSED / Re 以上 REJECTED）
export function judgeAql(defectCount: number, acceptNumberAc: number, rejectNumberRe: number): AqlVerdict {
  if (defectCount < 0) throw new Error('defectCount must be >= 0');
  if (defectCount <= acceptNumberAc) return 'PASSED';
  if (defectCount >= rejectNumberRe) return 'REJECTED';
  return 'INSPECTING';
}
