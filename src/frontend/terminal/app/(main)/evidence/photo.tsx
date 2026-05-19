// SCR-HA-008 写真証跡。PhotoCaptureView + EvidenceService.capturePhoto()
import React from 'react';
import { useRouter } from 'expo-router';
import * as FileSystem from 'expo-file-system';
import { PhotoCaptureView, type CapturedPhoto } from '../../../ui/PhotoCaptureView';
import { EvidenceService } from '../../../domain/evidence/EvidenceService';
import { useAuth } from '../../../contexts/AuthContext';
import { useWorkExecution } from '../../../contexts/WorkExecutionContext';

export default function EvidencePhotoScreen(): JSX.Element {
  const router = useRouter();
  const { state: auth } = useAuth();
  const { state: exec } = useWorkExecution();

  const handleCaptured = async (photo: CapturedPhoto): Promise<void> => {
    const evidence = new EvidenceService();
    // base64 → Uint8Array に変換してハッシュ計算へ渡す
    let bytes = new Uint8Array(0);
    if (photo.base64 !== undefined) {
      const binary = globalThis.atob(photo.base64);
      bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    } else {
      const fileInfo = await FileSystem.readAsStringAsync(photo.uri, { encoding: 'base64' });
      const binary = globalThis.atob(fileInfo);
      bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    }
    await evidence.capturePhoto({
      workExecutionId: exec.workExecutionId ?? 'unknown',
      stepId: exec.caseId ?? 'unknown',
      filePath: photo.uri,
      fileBytes: bytes,
      width: photo.width,
      height: photo.height,
      description: '作業証跡',
      uploadedBy: auth.user?.userId ?? 'unknown',
    });
    router.back();
  };

  return <PhotoCaptureView onCaptured={(p) => void handleCaptured(p)} onCancel={() => router.back()} />;
}
