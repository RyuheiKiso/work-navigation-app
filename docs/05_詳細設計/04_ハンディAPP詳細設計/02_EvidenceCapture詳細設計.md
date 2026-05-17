# 02 EvidenceCapture 詳細設計

本章は MOD-FE-HA-004（EvidenceCapture）の詳細設計を確定する。写真撮影（FR-EV-002）・測定値入力（FR-EV-003）・QR/バーコードスキャン（FR-EV-004）・Exif 除去（BR-BUS-028）・SHA-256 ハッシュ計算（FR-EV-001）の全仕様を定める。

---

## 1. モジュールインターフェース

```typescript
// src/features/evidence/EvidenceCaptureModule.ts

export interface EvidenceCaptureModule {
  /** FNC-FE-004: 写真撮影 → Exif 除去 → SHA-256 計算 → 一時保存 */
  capturePhoto(stepId: string): Promise<CaptureResult>;

  /** FNC-FE-005: 測定値入力 → USL/LSL バリデーション → 記録 */
  recordMeasurement(
    stepId: string,
    value: number,
    unit: string,
  ): Promise<MeasurementResult>;

  /** FNC-FE-006: QR/バーコードスキャン → GS1 形式検証 → required_scans マルチターゲット照合 → 記録 */
  scanQrCode(
    stepId: string,
    requiredScans: RequiredScan[]
  ): Promise<MultiTargetScanResult>;
}

/** FR-EV-013 マルチターゲット照合結果 */
export interface MultiTargetScanResult {
  targetResults: ScanVerification[];
  allRequiredMet: boolean;  // false なら StepEngine は 'WRONG_TOOL_SCAN' を返す
  rejectedReason?: 'WRONG_TOOL' | 'WRONG_MATERIAL' | 'CALIBRATION_EXPIRED' | 'UNKNOWN_SCAN_CODE';
}

/** 写真撮影設定（CFG 値は LocalAppSettings から取得） */
export interface PhotoCaptureConfig {
  minResolutionWidth: number;   // 1280 px（CFG-PH-001）
  minResolutionHeight: number;  // 720 px（CFG-PH-002）
  quality: number;              // 0.85（JPEG 品質。CFG-PH-003）
  stripExif: boolean;           // 常に true（BR-BUS-028: 位置情報保護）
  maxFileSizeBytes: number;     // 5_242_880（5 MB。CFG-PH-004）
}

/** 写真撮影結果 */
export interface CaptureResult {
  evidenceId: string;      // UUID v7
  fileHash: string;        // SHA-256（Exif 除去後のバイナリ）
  filePath: string;        // 端末ローカル一時パス（uploads/evidence/{evidenceId}.jpg）
  mimeType: string;        // 'image/jpeg'
  capturedAt: string;      // ISO 8601 UTC
  widthPx: number;
  heightPx: number;
  fileSizeBytes: number;
}

/** 測定値入力結果 */
export interface MeasurementResult {
  evidenceId: string;
  value: number;
  unit: string;
  withinSpec: boolean;     // USL/LSL 範囲内か
  recordedAt: string;
}

/** QR/バーコードスキャン結果 */
export interface QrScanResult {
  evidenceId: string;
  rawValue: string;        // スキャン生値
  format: QrFormat;        // 'qr_code' | 'gs1_128' | 'ean_13' 等
  validGS1: boolean;       // GS1 形式チェック結果（BR-BUS-034）
  scannedAt: string;
}

export type QrFormat =
  | 'qr_code'
  | 'gs1_128'
  | 'ean_13'
  | 'ean_8'
  | 'code_128'
  | 'code_39'
  | 'pdf_417'
  | 'data_matrix'
  | 'unknown';
```

---

## 2. 写真撮影フロー（FNC-FE-004）

### 2-1. 処理フロー

```
1. react-native-vision-camera で Camera.takePhoto() を呼び出す
2. 解像度チェック（width >= 1280 AND height >= 720）
   └ 不足 → ERR-VAL-003: 解像度不足ダイアログ表示、再撮影
3. ExifRemover.strip(filePath) で Exif メタデータを除去（BR-BUS-028）
4. SHA-256(stripped binary) を計算 → fileHash
5. 端末ローカルに uploads/evidence/{evidenceId}.jpg として保存
6. LocalDbService.insertEvidence(evidenceRecord) で evidence_files テーブルに記録
7. CaptureResult を返す
```

```typescript
// src/features/evidence/PhotoCapture.ts
import { Camera, PhotoFile } from 'react-native-vision-camera';
import { createHash } from 'react-native-quick-crypto';
import { v7 as uuidv7 } from 'uuid';
import RNFS from 'react-native-fs';

import type { ClockService } from '../../shared/clock/ClockService';
import type { LocalDbService } from '../../shared/db/LocalDbService';
import type { PhotoCaptureConfig, CaptureResult } from './types';
import { DomainError } from '../../shared/errors/DomainError';

export class PhotoCaptureService {
  constructor(
    private readonly config: PhotoCaptureConfig,
    private readonly clock: ClockService,
    private readonly localDb: LocalDbService,
  ) {}

  async capture(stepId: string, photo: PhotoFile): Promise<CaptureResult> {
    // 解像度チェック
    if (
      photo.width < this.config.minResolutionWidth ||
      photo.height < this.config.minResolutionHeight
    ) {
      throw new DomainError(
        'ERR-VAL-003',
        `解像度不足: ${photo.width}x${photo.height}（最小: ${this.config.minResolutionWidth}x${this.config.minResolutionHeight}）`,
      );
    }

    // Exif 除去（BR-BUS-028）
    const strippedPath = await this.stripExif(photo.path);

    // SHA-256 計算（stripped バイナリ）
    const binaryData = await RNFS.readFile(strippedPath, 'base64');
    const fileHash = this.computeSha256Base64(binaryData);

    // 一時ファイルとして保存
    const evidenceId = uuidv7();
    const destPath = `${RNFS.DocumentDirectoryPath}/uploads/evidence/${evidenceId}.jpg`;
    await RNFS.moveFile(strippedPath, destPath);

    // ファイルサイズチェック
    const stat = await RNFS.stat(destPath);
    if (stat.size > this.config.maxFileSizeBytes) {
      throw new DomainError(
        'ERR-VAL-004',
        `ファイルサイズ超過: ${stat.size} bytes（上限: ${this.config.maxFileSizeBytes} bytes）`,
      );
    }

    const result: CaptureResult = {
      evidenceId,
      fileHash,
      filePath: destPath,
      mimeType: 'image/jpeg',
      capturedAt: this.clock.nowIso(),
      widthPx: photo.width,
      heightPx: photo.height,
      fileSizeBytes: stat.size,
    };

    await this.localDb.insertEvidenceFile({
      evidenceId,
      stepId,
      fileHash,
      filePath: destPath,
      mimeType: 'image/jpeg',
      capturedAt: result.capturedAt,
      synced: false,
    });

    return result;
  }

  /** Exif ストリップ（react-native-image-exif または純 Node ストリッパー）*/
  private async stripExif(filePath: string): Promise<string> {
    // JPEG JFIF/Exif セグメント（0xFFE0, 0xFFE1）をゼロ埋めして除去
    // 実装: react-native-exif-remover ネイティブモジュール呼び出し
    const strippedPath = filePath.replace(/\.jpg$/, '_stripped.jpg');
    // ネイティブモジュール: ExifRemoverModule.strip(filePath, strippedPath)
    await (global as Record<string, unknown>)['ExifRemoverModule']?.strip(filePath, strippedPath);
    return strippedPath;
  }

  private computeSha256Base64(base64Data: string): string {
    const hash = createHash('sha256');
    hash.update(Buffer.from(base64Data, 'base64'));
    return hash.digest('hex');
  }
}
```

---

## 3. 測定値入力フロー（FNC-FE-005）

### 3-1. バリデーション仕様

```typescript
// src/features/evidence/MeasurementInput.ts

export interface MeasurementValidationResult {
  valid: boolean;
  withinSpec: boolean;
  errorCode?: 'ERR-VAL-001';
  message?: string;
}

export function validateMeasurement(
  value: number,
  usl: number | null,
  lsl: number | null,
  unit: string,
): MeasurementValidationResult {
  if (usl !== null && value > usl) {
    return {
      valid: false,
      withinSpec: false,
      errorCode: 'ERR-VAL-001',
      message: `測定値 ${value} ${unit} は上限値 ${usl} ${unit} を超過しています`,
    };
  }
  if (lsl !== null && value < lsl) {
    return {
      valid: false,
      withinSpec: false,
      errorCode: 'ERR-VAL-001',
      message: `測定値 ${value} ${unit} は下限値 ${lsl} ${unit} を下回っています`,
    };
  }
  return { valid: true, withinSpec: true };
}
```

### 3-2. Bluetooth 計測器連携（FR-EV-006）

```typescript
// src/features/evidence/BluetoothMeasurementBridge.ts

export interface BluetoothMeasurementBridge {
  /** Bluetooth 計測器をスキャンして接続 */
  connectDevice(deviceId: string): Promise<void>;

  /** 測定値を受信（ストリーミング）*/
  subscribe(onValue: (value: number, unit: string) => void): () => void;

  /** 接続解除 */
  disconnect(): Promise<void>;
}
```

---

## 4. QR/バーコードスキャンフロー（FNC-FE-006）

### 4-1. GS1 形式バリデーション（BR-BUS-034）

```typescript
// src/features/evidence/QrScanService.ts
import { Camera, useCodeScanner } from 'react-native-vision-camera';
import { v7 as uuidv7 } from 'uuid';

import type { ClockService } from '../../shared/clock/ClockService';
import type { QrScanResult, QrFormat } from './types';
import { DomainError } from '../../shared/errors/DomainError';

/** GS1-128 / GS1 QR Code の AI (Application Identifier) 検証 */
function validateGS1(rawValue: string): boolean {
  // GS1-128 は先頭 ']C1' または FNC1 (ASCII 0x1D) で始まる
  // GS1 QR は先頭 ']Q3' で始まる
  // 最低限の AI フォーマット（01 + 14 桁 GTIN）を検証
  const gs1Pattern = /^(\]C1|\]Q3|\x1D)(01\d{14}|410\d{13})/;
  return gs1Pattern.test(rawValue);
}

export class QrScanService {
  constructor(private readonly clock: ClockService) {}

  processScannedCode(
    rawValue: string,
    format: string,
    stepId: string,
  ): QrScanResult {
    const mappedFormat = this.mapFormat(format);
    const validGS1 =
      mappedFormat === 'gs1_128' ? validateGS1(rawValue) : false;

    if (mappedFormat === 'unknown') {
      throw new DomainError(
        'ERR-VAL-002',
        `未対応のバーコード形式: ${format}`,
      );
    }

    return {
      evidenceId: uuidv7(),
      rawValue,
      format: mappedFormat,
      validGS1,
      scannedAt: this.clock.nowIso(),
    };
  }

  private mapFormat(cameraFormat: string): QrFormat {
    const formatMap: Record<string, QrFormat> = {
      qr: 'qr_code',
      code128: 'code_128',
      code39: 'code_39',
      ean13: 'ean_13',
      ean8: 'ean_8',
      pdf417: 'pdf_417',
      datamatrix: 'data_matrix',
    };
    return formatMap[cameraFormat.toLowerCase()] ?? 'unknown';
  }
}
```

---

### §4-2 マルチターゲット照合（FNC-FE-006 拡張）

1. スキャン受信後、GS1 AI を解析して target を判定する:
   - AI '01' / '21' / '00' → target: 'material'（lot_id / serial_number / product_code）
   - AI '8004' / 社内識別 AI → target: 'tool' または 'instrument'
2. target が 'material' の場合: 既存の lots テーブル照合ロジック（FR-EV-004）を適用する
3. target が 'tool' の場合:
   - equipments テーブルを scan_code でルックアップする
   - requiredScans で指定された ref_id または ref_scan_code と一致することを検証する
   - 不一致の場合: MultiTargetScanResult.rejectedReason = 'WRONG_TOOL' として返す
4. target が 'instrument' の場合:
   - instruments テーブルを instrument_code でルックアップする
   - 一致確認後、calibration_due_date >= today を AND 評価する（BR-BUS-007 準拠）
   - calibration_due_date 期限切れの場合: rejectedReason = 'CALIBRATION_EXPIRED'
5. 全 required: true エントリが verified: true の場合のみ allRequiredMet = true を返す

---

## 5. エラーコード対応表

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-VAL-003 | 解像度不足（< 1280×720）| 「解像度が不足しています。カメラ設定を確認してください」ダイアログ |
| ERR-VAL-004 | ファイルサイズ超過（> 5 MB）| 「写真サイズが上限を超えました。再撮影してください」ダイアログ |
| ERR-VAL-001 | 測定値が USL/LSL 外 | 赤ボーダー + インライン警告メッセージ（入力継続は可能、StepEngine 側でゲート）|
| ERR-VAL-002 | 未対応バーコード形式 | 「このバーコード形式には対応していません」トースト + 再スキャン |
| ERR-VAL-005 | GS1 形式不正（BR-BUS-034）| 「GS1 形式が正しくありません」トースト + 再スキャン |
| ERR-VAL-006 | required_scans 不一致 / 未登録 scan_code | 赤バナー + 期待[ref] / 読取[value] 併記 + 不適合起票 CTA |
| ERR-SYS-002 | カメラ権限なし | システム設定画面への誘導ダイアログ |

---

**本節で確定した方針**
- **Exif 除去（BR-BUS-028）は PhotoCaptureService.stripExif にて必ず実施し、SHA-256 は Exif 除去後のバイナリに対して計算することで証拠ファイルの完全性とプライバシー保護を両立した。**
- **測定値バリデーション（USL/LSL）はフロントエンドで即時フィードバックし、StepEngine.canAdvanceToStep でも証拠ゲートとして二重検証することで BR-BUS-002 を確実に執行する。**
- **QR/バーコードスキャンの GS1 形式検証（BR-BUS-034）は QrScanService 内で実施し、形式不正時は StepEngine への進行要求を行わずユーザーに再スキャンを促す。**
- **EvidenceCaptureModule.scanQrCode を required_scans 配列駆動のマルチターゲット照合 API に拡張し、誤工具・誤材料・校正期限切れの 3 種類の拒否理由を明示的に区別することを確定した（FR-EV-013）。**

---

## 参照業界分析

### 必須
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)

### 関連
- [`90_業界分析/12_認知工学と状況認識.md`](../../90_業界分析/12_認知工学と状況認識.md)
