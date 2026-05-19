// 72dp タッチターゲットを強制する標準ボタン。手袋着用前提の現場 UX 要件
import React from 'react';
import { ActivityIndicator, StyleSheet, Text, TouchableOpacity, type ViewStyle } from 'react-native';
import { tokens } from '@wnav/shared';

export type WNavButtonVariant = 'primary' | 'secondary' | 'danger';

export interface WNavButtonProps {
  label: string;
  accessibilityLabel: string;
  onPress: () => void;
  variant?: WNavButtonVariant;
  disabled?: boolean;
  loading?: boolean;
  style?: ViewStyle;
}

export function WNavButton(props: WNavButtonProps): JSX.Element {
  const variant = props.variant ?? 'primary';
  const backgroundColor =
    variant === 'primary'
      ? tokens.brand.primary.light[500]
      : variant === 'danger'
      ? tokens.state.danger.light[500]
      : tokens.neutral[200];
  const color = variant === 'secondary' ? tokens.neutral[900] : '#FFFFFF';
  const isDisabled = props.disabled === true || props.loading === true;

  return (
    <TouchableOpacity
      accessible
      accessibilityLabel={props.accessibilityLabel}
      accessibilityRole="button"
      accessibilityState={{ disabled: isDisabled }}
      onPress={isDisabled ? undefined : props.onPress}
      disabled={isDisabled}
      style={[
        styles.button,
        { backgroundColor, opacity: isDisabled ? 0.4 : 1 },
        props.style,
      ]}
    >
      {props.loading === true ? (
        <ActivityIndicator color={color} />
      ) : (
        <Text style={[styles.label, { color }]} accessibilityElementsHidden>
          {props.label}
        </Text>
      )}
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  button: {
    minHeight: 72,
    minWidth: 72,
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    justifyContent: 'center',
  },
  label: {
    fontSize: 18,
    fontWeight: '700',
  },
});
