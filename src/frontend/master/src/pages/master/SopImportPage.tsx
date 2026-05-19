import type React from 'react';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Box,
  Button,
  Stack,
  Alert,
  Typography,
  Paper,
  Chip,
} from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import Papa from 'papaparse';
import * as XLSX from 'xlsx';
import { useMutation } from '@tanstack/react-query';
import { api } from '@/api/client';
import { PageHeader } from '@/components/PageHeader';
import { FileDropZone } from '@/components/FileDropZone';

interface ParsedStepRow {
  rowId: number;
  stepNumber: number;
  title: string;
  instruction: string;
  stepType: string;
  validation: 'ok' | 'error';
  validationMessage?: string;
}

// SCR-MA-005 SOP インポート。CSV/Excel → プレビュー → 取り込み（FR-MA-006）。
// papaparse / xlsx の解析はブラウザで完結し、サーバ送信前に検証する。
export function SopImportPage(): React.ReactElement {
  const navigate = useNavigate();
  const [rows, setRows] = useState<ParsedStepRow[]>([]);
  const [error, setError] = useState<string | null>(null);

  const importMutation = useMutation({
    mutationFn: async () => {
      const errors = rows.filter((r) => r.validation === 'error');
      if (errors.length > 0) throw new Error(`${errors.length} 行に検証エラーがあります`);
      const result = await api.post<{ id: string }>('/master/sops', {
        sopCode: `SOP-${Date.now()}`,
        nameJson: { ja: 'インポート SOP', en: '', zh: '' },
        descriptionJson: { ja: '', en: '', zh: '' },
        sopType: 'STANDARD',
        processId: '',
        operationId: '',
      });
      return result.data;
    },
    onSuccess: (created) => navigate(`/master/sops/${created.id}/edit`),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : 'インポートに失敗しました'),
  });

  const handleFile = async (file: File): Promise<void> => {
    setError(null);
    const ext = file.name.split('.').pop()?.toLowerCase();
    try {
      let parsed: Record<string, unknown>[] = [];
      if (ext === 'csv') {
        const text = await file.text();
        const result = Papa.parse<Record<string, unknown>>(text, { header: true, skipEmptyLines: true });
        parsed = result.data;
      } else if (ext === 'xlsx' || ext === 'xls') {
        const buffer = await file.arrayBuffer();
        const wb = XLSX.read(buffer, { type: 'array' });
        const firstSheet = wb.Sheets[wb.SheetNames[0] ?? ''];
        parsed = firstSheet ? (XLSX.utils.sheet_to_json(firstSheet) as Record<string, unknown>[]) : [];
      } else {
        throw new Error('対応形式は .csv / .xlsx のみです');
      }
      setRows(
        parsed.map((r, idx): ParsedStepRow => {
          const stepNumber = Number(r['stepNumber'] ?? r['step_number'] ?? idx + 1);
          const title = String(r['title'] ?? r['指示文'] ?? '');
          const instruction = String(r['instruction'] ?? r['作業指示'] ?? '');
          const stepType = String(r['stepType'] ?? r['type'] ?? 'standard');
          const validation: 'ok' | 'error' = title && instruction ? 'ok' : 'error';
          return {
            rowId: idx + 1,
            stepNumber,
            title,
            instruction,
            stepType,
            validation,
            ...(validation === 'error' ? { validationMessage: '指示文またはタイトルが未入力' } : {}),
          };
        }),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : 'ファイル解析に失敗しました');
      setRows([]);
    }
  };

  const columns: GridColDef<ParsedStepRow>[] = [
    { field: 'stepNumber', headerName: 'Step 番号', width: 100 },
    { field: 'title', headerName: 'タイトル', flex: 1 },
    { field: 'instruction', headerName: '作業指示', flex: 2 },
    { field: 'stepType', headerName: 'タイプ', width: 120 },
    {
      field: 'validation',
      headerName: '検証',
      width: 140,
      renderCell: ({ row }) =>
        row.validation === 'ok' ? (
          <Chip label="OK" color="success" size="small" />
        ) : (
          <Chip label={row.validationMessage ?? 'NG'} color="error" size="small" />
        ),
    },
  ];

  return (
    <Box>
      <PageHeader title="SOP インポート" subtitle="CSV/Excel ファイルから Step を一括取り込み" />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Stack spacing={3}>
        <FileDropZone accept=".csv,.xlsx" onFile={(f) => void handleFile(f)} />
        {rows.length > 0 && (
          <Paper sx={{ p: 2 }} elevation={1}>
            <Typography variant="h3" gutterBottom>
              プレビュー（{rows.length} 行）
            </Typography>
            <DataGrid
              rows={rows}
              columns={columns}
              getRowId={(r) => r.rowId}
              autoHeight
              pageSizeOptions={[10, 25]}
              initialState={{ pagination: { paginationModel: { pageSize: 10, page: 0 } } }}
              aria-label="インポートプレビュー"
            />
            <Stack direction="row" spacing={1} mt={2}>
              <Button
                variant="contained"
                disabled={importMutation.isPending}
                onClick={() => importMutation.mutate()}
                aria-label="インポートを実行"
              >
                インポート
              </Button>
              <Button onClick={() => setRows([])} aria-label="ファイル取込をリセット">
                リセット
              </Button>
            </Stack>
          </Paper>
        )}
      </Stack>
    </Box>
  );
}
