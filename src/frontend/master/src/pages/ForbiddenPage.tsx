import type React from 'react';
import { Box, Paper, Typography, Button } from '@mui/material';
import { useNavigate } from 'react-router-dom';

export function ForbiddenPage(): React.ReactElement {
  const navigate = useNavigate();
  return (
    <Box display="flex" alignItems="center" justifyContent="center" minHeight="100vh">
      <Paper sx={{ p: 4, maxWidth: 480 }}>
        <Typography variant="h1" component="h1" gutterBottom>
          403 アクセス権がありません
        </Typography>
        <Typography variant="body1" color="text.secondary" gutterBottom>
          この画面にアクセスする権限がロールに付与されていません。管理者にお問い合わせください。
        </Typography>
        <Button variant="contained" onClick={() => navigate(-1)} aria-label="前のページへ戻る">
          戻る
        </Button>
      </Paper>
    </Box>
  );
}
