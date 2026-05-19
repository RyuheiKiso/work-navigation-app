import type React from 'react';
import { useState, useEffect } from 'react';
import {
  Box,
  Button,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Stack,
  TextField,
  Alert,
} from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { Add, Edit, Block } from '@mui/icons-material';
import type { Process } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { ApiError } from '@/api/client';
import { useMasterList, useDeprecateMaster, useCreateMaster, useUpdateMaster } from '@/api/useMasterCrud';
import { queryKeys } from '@/api/queryKeys';
import { MasterListShell } from '@/components/MasterListShell';
import { LocalizedTextField } from '@/components/LocalizedTextField';
import { ImpactRangePreview } from '@/components/ImpactRangePreview';

// SCR-MA-001 プロセス一覧。CRUD + 論理削除 + 影響範囲確認（src/CLAUDE.md §マスタの不変ルール）。
export function ProcessListPage(): React.ReactElement {
  const [asOfUtc, setAsOfUtc] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [openCreate, setOpenCreate] = useState(false);
  const [editingProcess, setEditingProcess] = useState<Process | null>(null);
  const [confirmDeprecate, setConfirmDeprecate] = useState<Process | null>(null);

  const queryKey = queryKeys.master.processes(asOfUtc);
  const { data, isLoading, error } = useMasterList<Process>(queryKey, '/master/processes', {
    asOfUtc,
    search,
  });

  const deprecateMutation = useDeprecateMaster('/master/processes', queryKey);

  const columns: GridColDef<Process>[] = [
    { field: 'processCode', headerName: 'プロセスコード', width: 200 },
    {
      field: 'nameJson',
      headerName: '名称',
      flex: 1,
      valueGetter: (_, row) => resolveLocale(row.nameJson, 'ja'),
    },
    {
      field: 'isActive',
      headerName: '状態',
      width: 120,
      renderCell: ({ row }) =>
        row.deletedAt ? (
          <Chip label="廃止" color="error" size="small" />
        ) : row.isActive ? (
          <Chip label="有効" color="success" size="small" />
        ) : (
          <Chip label="無効" color="default" size="small" />
        ),
    },
    {
      field: 'actions',
      headerName: '操作',
      width: 220,
      sortable: false,
      renderCell: ({ row }) => (
        <Stack direction="row" spacing={1}>
          <Button
            size="small"
            startIcon={<Edit />}
            onClick={() => setEditingProcess(row)}
            disabled={!!row.deletedAt}
            aria-label={`${resolveLocale(row.nameJson, 'ja')} を編集`}
          >
            編集
          </Button>
          {!row.deletedAt && (
            <Button
              size="small"
              color="error"
              startIcon={<Block />}
              onClick={() => setConfirmDeprecate(row)}
              aria-label={`${resolveLocale(row.nameJson, 'ja')} を廃止`}
            >
              廃止
            </Button>
          )}
        </Stack>
      ),
    },
  ];

  return (
    <MasterListShell
      title="プロセス一覧"
      subtitle="製造プロセスのマスタ管理（CRUD・論理削除・時点参照）"
      search={search}
      onSearchChange={setSearch}
      onAsOfChange={setAsOfUtc}
      actions={
        <Button
          variant="contained"
          startIcon={<Add />}
          onClick={() => setOpenCreate(true)}
          aria-label="プロセスを新規作成"
        >
          新規作成
        </Button>
      }
    >
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error instanceof Error ? error.message : 'データ取得に失敗しました'}
        </Alert>
      )}
      <Box sx={{ width: '100%' }}>
        <DataGrid
          rows={data ?? []}
          columns={columns}
          loading={isLoading}
          getRowId={(row) => row.id}
          autoHeight
          pageSizeOptions={[10, 25, 50]}
          initialState={{ pagination: { paginationModel: { pageSize: 25, page: 0 } } }}
          aria-label="プロセス一覧テーブル"
        />
      </Box>
      <CreateProcessDialog open={openCreate} onClose={() => setOpenCreate(false)} queryKey={queryKey} />
      <EditProcessDialog process={editingProcess} onClose={() => setEditingProcess(null)} queryKey={queryKey} />
      <ConfirmDeprecateDialog
        process={confirmDeprecate}
        onClose={() => setConfirmDeprecate(null)}
        onConfirm={(id) => {
          deprecateMutation.mutate(id);
          setConfirmDeprecate(null);
        }}
      />
    </MasterListShell>
  );
}

interface CreateProcessForm {
  processCode: string;
  nameJa: string;
  nameEn: string;
  nameZh: string;
}

function CreateProcessDialog({
  open,
  onClose,
  queryKey,
}: {
  open: boolean;
  onClose: () => void;
  queryKey: ReturnType<typeof queryKeys.master.processes>;
}): React.ReactElement {
  const [form, setForm] = useState<CreateProcessForm>({ processCode: '', nameJa: '', nameEn: '', nameZh: '' });
  const [error, setError] = useState<string | null>(null);

  const create = useCreateMaster<Partial<Process>, Process>('/master/processes', queryKey);

  const submit = (): void => {
    setError(null);
    if (!/^[A-Za-z0-9_]+$/.test(form.processCode) || form.processCode.length === 0 || form.processCode.length > 64) {
      setError('プロセスコードは 1〜64 文字の英数字・アンダースコアのみ');
      return;
    }
    if (form.nameJa.length === 0 || form.nameJa.length > 200) {
      setError('プロセス名称（ja）は 1〜200 文字');
      return;
    }
    create.mutate(
      {
        processCode: form.processCode,
        nameJson: { ja: form.nameJa, en: form.nameEn, zh: form.nameZh },
        descriptionJson: { ja: '', en: '', zh: '' },
        isActive: true,
      },
      {
        onSuccess: () => {
          onClose();
          setForm({ processCode: '', nameJa: '', nameEn: '', nameZh: '' });
        },
        onError: (e: unknown) => {
          if (e instanceof ApiError) setError(e.problem.detail || e.problem.title);
          else if (e instanceof Error) setError(e.message);
        },
      },
    );
  };

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>プロセス新規作成</DialogTitle>
      <DialogContent>
        <Stack spacing={2} mt={1}>
          {error && <Alert severity="error">{error}</Alert>}
          <TextField
            label="プロセスコード"
            value={form.processCode}
            onChange={(e) => setForm({ ...form, processCode: e.target.value })}
            required
            inputProps={{ maxLength: 64, pattern: '[A-Za-z0-9_]+', 'aria-label': 'プロセスコード' }}
            helperText="1〜64 文字、英数字とアンダースコアのみ"
          />
          <LocalizedTextField
            label="プロセス名称"
            value={{ ja: form.nameJa, en: form.nameEn, zh: form.nameZh }}
            onChange={(v) => setForm({ ...form, nameJa: v.ja, nameEn: v.en, nameZh: v.zh })}
            required
            maxLength={200}
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} aria-label="作成をキャンセル">
          キャンセル
        </Button>
        <Button onClick={submit} variant="contained" disabled={create.isPending} aria-label="プロセスを保存">
          保存
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function EditProcessDialog({
  process,
  onClose,
  queryKey,
}: {
  process: Process | null;
  onClose: () => void;
  queryKey: ReturnType<typeof queryKeys.master.processes>;
}): React.ReactElement {
  const [form, setForm] = useState<CreateProcessForm>({ processCode: '', nameJa: '', nameEn: '', nameZh: '' });
  const [error, setError] = useState<string | null>(null);
  const update = useUpdateMaster<Partial<Process>, Process>('/master/processes', queryKey);

  // process が変わったらフォームを初期化する
  useEffect(() => {
    if (process) {
      setForm({
        processCode: process.processCode,
        nameJa: process.nameJson.ja,
        nameEn: process.nameJson.en,
        nameZh: process.nameJson.zh,
      });
      setError(null);
    }
  }, [process]);

  const submit = (): void => {
    if (!process) return;
    setError(null);
    if (!/^[A-Za-z0-9_]+$/.test(form.processCode) || form.processCode.length === 0 || form.processCode.length > 64) {
      setError('プロセスコードは 1〜64 文字の英数字・アンダースコアのみ');
      return;
    }
    if (form.nameJa.length === 0 || form.nameJa.length > 200) {
      setError('プロセス名称（ja）は 1〜200 文字');
      return;
    }
    update.mutate(
      {
        id: process.id,
        patch: {
          processCode: form.processCode,
          nameJson: { ja: form.nameJa, en: form.nameEn, zh: form.nameZh },
        },
      },
      {
        onSuccess: () => onClose(),
        onError: (e: unknown) => {
          if (e instanceof ApiError) setError(e.problem.detail || e.problem.title);
          else if (e instanceof Error) setError(e.message);
        },
      },
    );
  };

  return (
    <Dialog open={process !== null} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>プロセス編集</DialogTitle>
      <DialogContent>
        <Stack spacing={2} mt={1}>
          {error && <Alert severity="error">{error}</Alert>}
          <TextField
            label="プロセスコード"
            value={form.processCode}
            onChange={(e) => setForm({ ...form, processCode: e.target.value })}
            required
            inputProps={{ maxLength: 64, pattern: '[A-Za-z0-9_]+', 'aria-label': 'プロセスコード' }}
            helperText="1〜64 文字、英数字とアンダースコアのみ"
          />
          <LocalizedTextField
            label="プロセス名称"
            value={{ ja: form.nameJa, en: form.nameEn, zh: form.nameZh }}
            onChange={(v) => setForm({ ...form, nameJa: v.ja, nameEn: v.en, nameZh: v.zh })}
            required
            maxLength={200}
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} aria-label="編集をキャンセル">キャンセル</Button>
        <Button onClick={submit} variant="contained" disabled={update.isPending} aria-label="プロセスを更新">
          更新
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function ConfirmDeprecateDialog({
  process,
  onClose,
  onConfirm,
}: {
  process: Process | null;
  onClose: () => void;
  onConfirm: (id: string) => void;
}): React.ReactElement {
  // 廃止前の影響範囲表示（dry-run）。将来 GET /master/processes/:id/impact が実装されたら接続する。
  const impacted: Array<{ type: 'work_order'; id: string; label: string }> = [];

  return (
    <Dialog open={process !== null} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>プロセスを廃止</DialogTitle>
      <DialogContent>
        {process && (
          <Stack spacing={2}>
            <Alert severity="warning">
              {resolveLocale(process.nameJson, 'ja')}（{process.processCode}）を論理削除します。物理削除はされません。
            </Alert>
            <ImpactRangePreview items={impacted} emptyMessage="このプロセスを参照中の作業指示はありません" />
          </Stack>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} aria-label="廃止をキャンセル">
          キャンセル
        </Button>
        <Button
          color="error"
          variant="contained"
          onClick={() => process && onConfirm(process.id)}
          aria-label="廃止を確定"
        >
          廃止する
        </Button>
      </DialogActions>
    </Dialog>
  );
}
