import React, { Component, type ErrorInfo, type ReactNode } from 'react';
import { StyleSheet, Text, View } from 'react-native';

interface Props {
  children: ReactNode;
  fallback?: (error: Error, reset: () => void) => ReactNode;
}

interface State {
  error: Error | null;
}

// 画面単位で予期せぬ例外を捕捉して FullStop を防ぐためのエラー境界
export class ErrorBoundary extends Component<Props, State> {
  override state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  override componentDidCatch(error: Error, info: ErrorInfo): void {
    // 開発時の調査のため stderr へ詳細を残す（本番では監視基盤へ送信する想定）
    console.error('[ErrorBoundary]', error, info.componentStack);
  }

  reset = (): void => {
    this.setState({ error: null });
  };

  override render(): ReactNode {
    const { error } = this.state;
    const { fallback, children } = this.props;
    if (error !== null) {
      if (fallback !== undefined) return fallback(error, this.reset);
      return (
        <View style={styles.container} accessibilityRole="alert">
          <Text style={styles.title} accessibilityLabel="予期しないエラー">
            予期しないエラーが発生しました
          </Text>
          <Text style={styles.message}>{error.message}</Text>
        </View>
      );
    }
    return children;
  }
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 24,
    backgroundColor: '#1E3A5F',
  },
  title: {
    fontSize: 22,
    fontWeight: '700',
    color: '#FFFFFF',
    marginBottom: 12,
  },
  message: {
    fontSize: 16,
    color: '#FFFFFF',
    textAlign: 'center',
  },
});
