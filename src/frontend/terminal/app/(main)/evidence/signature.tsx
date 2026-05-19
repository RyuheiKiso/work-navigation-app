// SCR-HA-010 電子サイン。SignaturePad で署名取得 → Ed25519 署名 → 永続化
import React from 'react';
import { Alert } from 'react-native';
import { useRouter } from 'expo-router';
import { generateId } from '@wnav/shared/domain/id';
import { SignaturePad } from '../../../ui/SignaturePad';
import { sha256Hex } from '../../../crypto/sha256';
import { sign } from '../../../crypto/ed25519';
import { KeystoreAdapter, KEY_PRIVATE } from '../../../crypto/KeystoreAdapter';
import { getDataSource } from '../../../db/data-source';
import { LocalElectronicSign } from '../../../db/entities/LocalElectronicSign';
import { useAuth } from '../../../contexts/AuthContext';
import { useWorkExecution } from '../../../contexts/WorkExecutionContext';

export default function SignatureScreen(): JSX.Element {
  const router = useRouter();
  const { state: auth } = useAuth();
  const { state: exec } = useWorkExecution();

  const handleConfirm = async (base64: string): Promise<void> => {
    try {
      const keystore = new KeystoreAdapter();
      let privateKey = await keystore.getItem(KEY_PRIVATE);
      if (privateKey === null) {
        Alert.alert('鍵未登録', '初回サインのため、設定からキー登録を実施してください');
        return;
      }
      const contentHash = sha256Hex(base64);
      const messageBytes = new TextEncoder().encode(contentHash);
      const signatureHex = sign(messageBytes, privateKey);

      const entity: LocalElectronicSign = {
        id: generateId(),
        signerId: auth.user?.userId ?? 'unknown',
        signedContentHash: contentHash,
        contextType: 'step_sign',
        contextId: exec.caseId ?? 'unknown',
        stepId: exec.currentStepId ?? null,
        signedAt: new Date().toISOString(),
        hashChainBlockId: generateId(),
        hashChainValue: signatureHex,
        hashChainPrev: '0'.repeat(64),
        deviceId: 'terminal-1',
        synced: false,
      };
      await getDataSource().getRepository(LocalElectronicSign).save(entity);
      router.back();
    } catch (err) {
      const message = err instanceof Error ? err.message : '署名失敗';
      Alert.alert('署名エラー', message);
    }
  };

  return <SignaturePad onConfirm={(s) => void handleConfirm(s)} onCancel={() => router.back()} />;
}
