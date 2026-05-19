// QR/バーコードスキャン用オーバーレイ。expo-camera の BarCodeScanner 互換 API を使用
import React, { useState } from 'react';
import { StyleSheet, Text, View } from 'react-native';
import { CameraView, useCameraPermissions, type BarcodeScanningResult } from 'expo-camera';
import { WNavButton } from './WNavButton';

export interface QrScannerOverlayProps {
  onScanned: (value: string, type: string) => void;
  onCancel: () => void;
}

export function QrScannerOverlay(props: QrScannerOverlayProps): JSX.Element {
  const [permission, requestPermission] = useCameraPermissions();
  const [scanned, setScanned] = useState(false);

  if (!permission?.granted) {
    return (
      <View style={styles.center}>
        <Text style={styles.message}>カメラ権限が必要です</Text>
        <WNavButton
          label="権限を許可"
          accessibilityLabel="カメラ権限を許可"
          onPress={() => {
            void requestPermission();
          }}
        />
      </View>
    );
  }

  const handleScanned = (result: BarcodeScanningResult): void => {
    if (scanned) return;
    setScanned(true);
    props.onScanned(result.data, result.type);
  };

  return (
    <View style={styles.container}>
      <CameraView
        style={styles.camera}
        barcodeScannerSettings={{ barcodeTypes: ['qr', 'code128', 'ean13', 'ean8', 'code39'] }}
        onBarcodeScanned={scanned ? undefined : handleScanned}
      />
      <View style={styles.controls}>
        <WNavButton
          label="キャンセル"
          accessibilityLabel="スキャンをキャンセル"
          variant="secondary"
          onPress={props.onCancel}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#000000' },
  camera: { flex: 1 },
  controls: { padding: 16 },
  center: { flex: 1, justifyContent: 'center', alignItems: 'center', padding: 16 },
  message: { fontSize: 18, marginBottom: 16, color: '#FFFFFF' },
});
