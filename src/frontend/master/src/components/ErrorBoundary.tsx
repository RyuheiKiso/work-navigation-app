import React from 'react';
import { Box, Typography, Button, Paper } from '@mui/material';

interface State {
  hasError: boolean;
  error: Error | null;
}

// 画面単位 ErrorBoundary（src/frontend/CLAUDE.md §共通コーディング規約）。
// アプリ全体の最上位とページ単位の二段構えで利用する。
export class ErrorBoundary extends React.Component<
  { children: React.ReactNode; fallback?: React.ReactNode },
  State
> {
  state: State = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  override componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    // 開発時のみ console に詳細を出力。本番では集約ログに送る想定
    console.error('[ErrorBoundary]', error, errorInfo);
  }

  override render(): React.ReactNode {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;
      return (
        <Box display="flex" alignItems="center" justifyContent="center" minHeight="60vh" p={3}>
          <Paper sx={{ p: 4, maxWidth: 560 }} role="alert" aria-live="assertive">
            <Typography variant="h2" component="h1" gutterBottom>
              画面の表示中にエラーが発生しました
            </Typography>
            <Typography variant="body1" color="text.secondary" gutterBottom>
              {this.state.error?.message ?? '不明なエラー'}
            </Typography>
            <Button
              variant="contained"
              onClick={() => this.setState({ hasError: false, error: null })}
              aria-label="再試行"
            >
              再試行
            </Button>
          </Paper>
        </Box>
      );
    }
    return this.props.children;
  }
}
