// SCR-HA-003 QR スキャン。スキャン結果で作業特定 → home または step へ
import React from 'react';
import { useRouter } from 'expo-router';
import { QrScannerOverlay } from '../../ui/QrScannerOverlay';

export default function QrScanScreen(): JSX.Element {
  const router = useRouter();

  const handleScanned = (value: string): void => {
    // QR ペイロードが caseId 形式なら直接 step 画面へ。それ以外は home に戻す
    if (value.startsWith('case:')) {
      const caseId = value.slice('case:'.length);
      router.replace(`/(main)/step/${encodeURIComponent(caseId)}`);
    } else {
      router.replace('/(main)/home');
    }
  };

  return (
    <QrScannerOverlay
      onScanned={handleScanned}
      onCancel={() => router.back()}
    />
  );
}
