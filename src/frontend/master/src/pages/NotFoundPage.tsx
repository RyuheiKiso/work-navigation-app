import type React from 'react';
import { Box, Typography, Button } from '@mui/material';
import { useNavigate } from 'react-router-dom';

export function NotFoundPage(): React.ReactElement {
  const navigate = useNavigate();
  return (
    <Box textAlign="center" mt={8}>
      <Typography variant="h1" component="h1" gutterBottom>
        404 ページが見つかりません
      </Typography>
      <Button variant="contained" onClick={() => navigate('/')} aria-label="ホームに戻る">
        ホームに戻る
      </Button>
    </Box>
  );
}
