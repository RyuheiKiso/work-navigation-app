// react-native-signature-canvas ラッパ。署名データを Base64 で返す
import React, { useRef } from 'react';
import { StyleSheet, View } from 'react-native';
import SignatureCanvas, { type SignatureViewRef } from 'react-native-signature-canvas';
import { WNavButton } from './WNavButton';

export interface SignaturePadProps {
  onConfirm: (base64Signature: string) => void;
  onCancel: () => void;
}

export function SignaturePad(props: SignaturePadProps): JSX.Element {
  const ref = useRef<SignatureViewRef>(null);

  const handleSign = (signature: string): void => {
    props.onConfirm(signature);
  };

  const handleSubmit = (): void => {
    ref.current?.readSignature();
  };

  const handleClear = (): void => {
    ref.current?.clearSignature();
  };

  return (
    <View style={styles.container}>
      <SignatureCanvas
        ref={ref}
        onOK={handleSign}
        webStyle={WEB_STYLE}
        descriptionText="ここに署名してください"
        clearText="クリア"
        confirmText="確定"
      />
      <View style={styles.actions}>
        <WNavButton
          label="クリア"
          accessibilityLabel="署名をクリア"
          variant="secondary"
          onPress={handleClear}
          style={styles.button}
        />
        <WNavButton
          label="キャンセル"
          accessibilityLabel="署名をキャンセル"
          variant="secondary"
          onPress={props.onCancel}
          style={styles.button}
        />
        <WNavButton
          label="確定"
          accessibilityLabel="署名を確定"
          variant="primary"
          onPress={handleSubmit}
          style={styles.button}
        />
      </View>
    </View>
  );
}

const WEB_STYLE = `.m-signature-pad--footer{display:none;} body,html{width:100%;height:100%;}`;

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16 },
  actions: { flexDirection: 'row', justifyContent: 'space-between', marginTop: 16, gap: 8 },
  button: { flex: 1, marginHorizontal: 4 },
});
