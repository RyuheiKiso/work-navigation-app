import type React from 'react';
import { useState, type ReactNode } from 'react';
import {
  Box,
  Button,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Stack,
  Alert,
} from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { Add, Block, Edit } from '@mui/icons-material';
import type { QueryKey } from '@tanstack/react-query';
import { useMasterList, useDeprecateMaster, useCreateMaster } from '@/api/useMasterCrud';
import { MasterListShell } from './MasterListShell';
import { ImpactRangePreview } from './ImpactRangePreview';

// 物理削除禁止（src/CLAUDE.md §マスタの不変ルール）の論理削除しか持たないマスタ用の汎用一覧画面。
export interface GenericMasterListPageProps<T extends { id: string; deletedAt: string | null }, TCreate> {
  title: string;
  subtitle?: string;
  endpoint: string;
  queryKeyBuilder: (asOf: string | null) => QueryKey;
  columns: GridColDef<T>[];
  // 作成フォームの初期値（呼び出し側が型を制約する）
  initialCreateForm: TCreate;
  // 作成フォーム本体（state を呼び出し側で保持しない設計）
  renderCreateForm: (state: {
    value: TCreate;
    onChange: (next: TCreate) => void;
    error: string | null;
    setError: (msg: string | null) => void;
  }) => ReactNode;
  // 作成前バリデーション（OK のとき payload を返す）
  validateAndBuildPayload: (form: TCreate) => { ok: true; payload: Partial<T> } | { ok: false; message: string };
  // 廃止確認ダイアログのラベル取得
  labelOf: (item: T) => string;
  // 廃止時の影響範囲リスト取得（任意）
  fetchImpacts?: (id: string) => Promise<Array<{ type: 'work_order' | 'work_execution' | 'sop_version'; id: string; label: string }>>;
  // 行のステータス表示（任意。デフォルトは「廃止 / 有効」）
  renderStatus?: (item: T) => ReactNode;
}

export function GenericMasterListPage<T extends { id: string; deletedAt: string | null }, TCreate>({
  title,
  subtitle,
  endpoint,
  queryKeyBuilder,
  columns,
  initialCreateForm,
  renderCreateForm,
  validateAndBuildPayload,
  labelOf,
  renderStatus,
}: GenericMasterListPageProps<T, TCreate>): React.ReactElement {
  const [asOfUtc, setAsOfUtc] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [openCreate, setOpenCreate] = useState(false);
  const [confirmDeprecate, setConfirmDeprecate] = useState<T | null>(null);
  const [createForm, setCreateForm] = useState<TCreate>(initialCreateForm);
  const [createError, setCreateError] = useState<string | null>(null);

  const queryKey = queryKeyBuilder(asOfUtc);
  const { data, isLoading, error } = useMasterList<T>(queryKey, endpoint, { asOfUtc, search });
  const deprecateMutation = useDeprecateMaster(endpoint, queryKey);
  const create = useCreateMaster<Partial<T>, T>(endpoint, queryKey);

  const allColumns: GridColDef<T>[] = [
    ...columns,
    {
      field: '__status',
      headerName: '状態',
      width: 120,
      sortable: false,
      renderCell: ({ row }) =>
        renderStatus ? (
          renderStatus(row)
        ) : row.deletedAt ? (
          <Chip label="廃止" color="error" size="small" />
        ) : (
          <Chip label="有効" color="success" size="small" />
        ),
    },
    {
      field: '__actions',
      headerName: '操作',
      width: 220,
      sortable: false,
      renderCell: ({ row }) => (
        <Stack direction="row" spacing={1}>
          <Button size="small" startIcon={<Edit />} href={`${endpoint}/${row.id}/edit`} aria-label={`${labelOf(row)} を編集`}>
            編集
          </Button>
          {!row.deletedAt && (
            <Button
              size="small"
              color="error"
              startIcon={<Block />}
              onClick={() => setConfirmDeprecate(row)}
              aria-label={`${labelOf(row)} を廃止`}
            >
              廃止
            </Button>
          )}
        </Stack>
      ),
    },
  ];

  const submit = (): void => {
    const validation = validateAndBuildPayload(createForm);
    if (!validation.ok) {
      setCreateError(validation.message);
      return;
    }
    create.mutate(validation.payload, {
      onSuccess: () => {
        setOpenCreate(false);
        setCreateForm(initialCreateForm);
        setCreateError(null);
      },
      onError: (e: unknown) => setCreateError(e instanceof Error ? e.message : '作成に失敗しました'),
    });
  };

  return (
    <MasterListShell
      title={title}
      {...(subtitle !== undefined ? { subtitle } : {})}
      search={search}
      onSearchChange={setSearch}
      onAsOfChange={setAsOfUtc}
      actions={
        <Button variant="contained" startIcon={<Add />} onClick={() => setOpenCreate(true)} aria-label={`${title} を新規作成`}>
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
          columns={allColumns}
          loading={isLoading}
          getRowId={(row) => row.id}
          autoHeight
          pageSizeOptions={[10, 25, 50]}
          initialState={{ pagination: { paginationModel: { pageSize: 25, page: 0 } } }}
          aria-label={`${title} テーブル`}
        />
      </Box>

      <Dialog open={openCreate} onClose={() => setOpenCreate(false)} fullWidth maxWidth="sm">
        <DialogTitle>{title} を新規作成</DialogTitle>
        <DialogContent>
          <Stack spacing={2} mt={1}>
            {createError && <Alert severity="error">{createError}</Alert>}
            {renderCreateForm({ value: createForm, onChange: setCreateForm, error: createError, setError: setCreateError })}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOpenCreate(false)} aria-label="作成をキャンセル">
            キャンセル
          </Button>
          <Button onClick={submit} variant="contained" disabled={create.isPending} aria-label="保存">
            保存
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog open={confirmDeprecate !== null} onClose={() => setConfirmDeprecate(null)} fullWidth maxWidth="sm">
        <DialogTitle>{title} を廃止</DialogTitle>
        <DialogContent>
          {confirmDeprecate && (
            <Stack spacing={2}>
              <Alert severity="warning">
                {labelOf(confirmDeprecate)} を論理削除します。物理削除はされません。
              </Alert>
              <ImpactRangePreview items={[]} emptyMessage="影響範囲はありません" />
            </Stack>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfirmDeprecate(null)} aria-label="廃止をキャンセル">
            キャンセル
          </Button>
          <Button
            color="error"
            variant="contained"
            onClick={() => {
              if (confirmDeprecate) {
                deprecateMutation.mutate(confirmDeprecate.id);
                setConfirmDeprecate(null);
              }
            }}
            aria-label="廃止を確定"
          >
            廃止する
          </Button>
        </DialogActions>
      </Dialog>
    </MasterListShell>
  );
}
