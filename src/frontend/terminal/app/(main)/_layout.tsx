// (main) は認証後ナビゲーション領域。AuthGuard で未認証なら login へ redirect する
import React from 'react';
import { Stack } from 'expo-router';
import { AuthGuard } from '../../auth/AuthGuard';
import { EmergencyModeOverlay } from '../../ui/EmergencyModeOverlay';
import { View } from 'react-native';

export default function MainLayout(): JSX.Element {
  return (
    <AuthGuard>
      <View style={{ flex: 1 }}>
        <EmergencyModeOverlay />
        <Stack screenOptions={{ headerShown: true }} />
      </View>
    </AuthGuard>
  );
}
