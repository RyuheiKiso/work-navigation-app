import type React from 'react';
import { useCallback, useState } from 'react';
import { Box, Typography, Button } from '@mui/material';
import CloudUploadIcon from '@mui/icons-material/CloudUpload';

// CSV/Excel 等のファイル取り込み UI（FR-MA-006 SOP インポート用）。
// ファイル種別判定は拡張子で行い、実体の解析は呼び出し側で papaparse/xlsx により行う。
export function FileDropZone({
  accept = '.csv,.xlsx',
  onFile,
}: {
  accept?: string;
  onFile: (file: File) => void;
}): React.ReactElement {
  const [drag, setDrag] = useState(false);

  const handleDrop = useCallback(
    (e: React.DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      setDrag(false);
      const file = e.dataTransfer.files.item(0);
      if (file) onFile(file);
    },
    [onFile],
  );

  return (
    <Box
      onDragOver={(e) => {
        e.preventDefault();
        setDrag(true);
      }}
      onDragLeave={() => setDrag(false)}
      onDrop={handleDrop}
      sx={{
        border: '2px dashed',
        borderColor: drag ? 'primary.main' : 'divider',
        backgroundColor: drag ? 'action.hover' : 'background.default',
        borderRadius: 2,
        p: 4,
        textAlign: 'center',
        cursor: 'pointer',
        transition: 'all 0.2s',
      }}
      role="region"
      aria-label="ファイル取り込みドロップゾーン"
    >
      <CloudUploadIcon sx={{ fontSize: 48, color: 'text.secondary' }} />
      <Typography variant="body1" gutterBottom>
        ファイルをここにドロップするか、ボタンから選択してください
      </Typography>
      <Typography variant="caption" color="text.secondary">
        対応形式: {accept}
      </Typography>
      <Box mt={2}>
        <Button variant="contained" component="label" startIcon={<CloudUploadIcon />}>
          ファイルを選択
          <input
            type="file"
            hidden
            accept={accept}
            onChange={(e) => {
              const file = e.target.files?.item(0);
              if (file) onFile(file);
            }}
            aria-label="ファイルを選択"
          />
        </Button>
      </Box>
    </Box>
  );
}
