// expo-camera のカメラビュー。撮影 → SHA-256 ハッシュ計算は EvidenceService 側で実施
import React, { useRef, useState } from 'react';
import { ActivityIndicator, StyleSheet, Text, View } from 'react-native';
import { CameraView, type CameraType, useCameraPermissions } from 'expo-camera';
import { WNavButton } from './WNavButton';

export interface CapturedPhoto {
  uri: string;
  width: number;
  height: number;
  base64?: string;
}

export interface PhotoCaptureViewProps {
  onCaptured: (photo: CapturedPhoto) => void;
  onCancel: () => void;
}

export function PhotoCaptureView(props: PhotoCaptureViewProps): JSX.Element {
  const [permission, requestPermission] = useCameraPermissions();
  const cameraRef = useRef<CameraView | null>(null);
  const [busy, setBusy] = useState(false);
  const facing: CameraType = 'back';

  if (permission === null) {
    return (
      <View style={styles.center}>
        <ActivityIndicator />
      </View>
    );
  }

  if (!permission.granted) {
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

  const handleCapture = async (): Promise<void> => {
    if (cameraRef.current === null || busy) return;
    setBusy(true);
    try {
      const photo = await cameraRef.current.takePictureAsync({ quality: 0.7, base64: true });
      if (photo !== undefined) {
        props.onCaptured({
          uri: photo.uri,
          width: photo.width ?? 0,
          height: photo.height ?? 0,
          base64: photo.base64,
        });
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <View style={styles.container}>
      <CameraView ref={cameraRef} style={styles.camera} facing={facing} />
      <View style={styles.controls}>
        <WNavButton
          label="キャンセル"
          accessibilityLabel="撮影をキャンセル"
          variant="secondary"
          onPress={props.onCancel}
          style={styles.button}
        />
        <WNavButton
          label="撮影"
          accessibilityLabel="写真を撮影"
          variant="primary"
          onPress={() => {
            void handleCapture();
          }}
          loading={busy}
          style={styles.button}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#000000' },
  camera: { flex: 1 },
  controls: { flexDirection: 'row', padding: 16, gap: 12 },
  button: { flex: 1 },
  center: { flex: 1, justifyContent: 'center', alignItems: 'center', padding: 16 },
  message: { fontSize: 18, marginBottom: 16, color: '#FFFFFF' },
});
