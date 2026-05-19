import type React from 'react';
import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  Box,
  Button,
  Stack,
  Tab,
  Tabs,
  TextField,
  Paper,
  Typography,
  Alert,
  Divider,
} from '@mui/material';
import { Undo, Redo, Visibility, Send } from '@mui/icons-material';
import type { Sop, Step } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { api, ApiError } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { useSopEditorStore } from '@/stores/sopEditorStore';
import { PageHeader } from '@/components/PageHeader';
import { LocalizedTextField } from '@/components/LocalizedTextField';
import { AutoSaveIndicator } from '@/components/AutoSaveIndicator';
import { SopFlowEditor } from '@/components/SopFlowEditor';
import { DslConditionBuilder } from '@/components/DslConditionBuilder';

// SCR-MA-004 SOP 編集（docs/05/05_WebAPP詳細設計/01_SopEditor詳細設計.md）。
// Auto-Save debounce 1 秒 + Undo/Redo（最大50）+ センターペイン（step-form / dag-flow）+ 即時公開禁止。
type CenterMode = 'step-form' | 'dag-flow';

export function SopEditPage(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const isNew = !id;

  const [centerMode, setCenterMode] = useState<CenterMode>('step-form');
  const [editingStepId, setEditingStepId] = useState<string | null>(null);
  const [autoSaveState, setAutoSaveState] = useState<'idle' | 'editing' | 'saving' | 'saved' | 'error'>('idle');
  const [error, setError] = useState<string | null>(null);

  const { steps, flow, undoStack, redoStack, isDirty, lastSavedAt, push, setFlow, undo, redo, markSaved, clear } =
    useSopEditorStore();

  const sopQuery = useQuery({
    queryKey: queryKeys.master.sop(id ?? 'new'),
    queryFn: async (): Promise<Sop | null> => {
      if (isNew) return null;
      const result = await api.get<Sop>(`/master/sops/${id}`);
      return result.data;
    },
    enabled: !isNew,
  });

  // 別 SOP に切り替わったらストアをクリアしてからサーバーデータで初期化する
  useEffect(() => {
    clear();
  }, [id, clear]);

  // サーバーから取得した steps でストアを初期化する（ロード完了後に一度だけ実行）
  useEffect(() => {
    if (!sopQuery.data) return;
    // steps は SOP と別エンドポイントで管理されるため、ここでは nameJson 等のメタ情報のみが存在する。
    // steps は GET /master/sops/{id}/steps から取得する必要があるが、現状は空配列で開始する（初版と同等）。
    // flow は GET /master/sops/{id}/flow から取得するが、現状未定義の場合は空で開始する。
  }, [sopQuery.data]);

  // Auto-Save: 入力停止 3 秒後のデバウンスで発動する（EVT-104-002 / docs/05_詳細設計/06_画面詳細設計/05_イベントとアクション定義.md §SCR-MA-004）
  useEffect(() => {
    if (!isDirty || isNew) return;
    setAutoSaveState('editing');
    const timer = setTimeout(async () => {
      try {
        setAutoSaveState('saving');
        // steps と flow（DAG ノード・エッジ）を同時に保存する
        await api.patch(`/master/sops/${id}`, { steps, flow: JSON.stringify(flow) });
        const now = new Date().toISOString();
        markSaved(now);
        setAutoSaveState('saved');
      } catch (e) {
        setAutoSaveState('error');
        setError(e instanceof Error ? e.message : 'Auto-Save 失敗');
      }
    }, 3000);
    return () => clearTimeout(timer);
  }, [isDirty, isNew, id, steps, flow, markSaved]);

  const saveSopMutation = useMutation({
    mutationFn: async (payload: Partial<Sop>): Promise<Sop> => {
      if (isNew) {
        const r = await api.post<Sop>('/master/sops', payload);
        return r.data;
      }
      const r = await api.patch<Sop>(`/master/sops/${id}`, payload);
      return r.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: ['master', 'sops'] }),
    onError: (e: unknown) => {
      if (e instanceof ApiError) setError(e.problem.detail);
      else setError(e instanceof Error ? e.message : '保存に失敗しました');
    },
  });

  const requestReview = useMutation({
    mutationFn: async () => {
      if (!id) throw new Error('SOP が未保存です');
      // OpenAPI operationId: submitSopForReview → /submit エンドポイントを使用する
      await api.post(`/master/sops/${id}/submit`, {});
    },
    onSuccess: () => navigate(`/master/sops/${id}/review`),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : 'レビュー依頼に失敗しました'),
  });

  const handleAddStep = (): void => {
    const nextNumber = (steps[steps.length - 1]?.stepNumber ?? 0) + 1;
    const draft: Step = {
      id: `new-${crypto.randomUUID()}`,
      sopVersionId: '',
      stepNumber: nextNumber,
      stepType: 'standard',
      titleJson: { ja: `Step ${nextNumber}`, en: '', zh: '' },
      instructionJson: { ja: '', en: '', zh: '' },
      payload: '{}',
      isMandatory: true,
      requiresEvidence: false,
      requiresSign: false,
      skillLevelRequired: 1,
      estimatedSeconds: 60,
      fallbackType: 'manual',
      flowRules: { onComplete: 'next', onSkip: 'next' },
      deletedAt: null,
    };
    push([...steps, draft]);
    setEditingStepId(draft.id);
  };

  const handleUpdateStep = (next: Step): void => {
    push(steps.map((s) => (s.id === next.id ? next : s)));
  };

  const handleDeleteStep = (stepId: string): void => {
    push(steps.filter((s) => s.id !== stepId));
    if (editingStepId === stepId) setEditingStepId(null);
  };

  const selectedStep = steps.find((s) => s.id === editingStepId) ?? null;

  return (
    <Box>
      <PageHeader
        title={isNew ? 'SOP 新規作成' : `SOP 編集 (${resolveLocale(sopQuery.data?.nameJson, 'ja')})`}
        subtitle="Auto-Save: 1 秒・Undo/Redo: 最大 50 件・即時公開禁止"
        actions={
          <Stack direction="row" spacing={1}>
            <AutoSaveIndicator state={autoSaveState} lastSavedAt={lastSavedAt} />
            <Button onClick={undo} disabled={undoStack.length === 0} startIcon={<Undo />} aria-label="元に戻す">
              元に戻す
            </Button>
            <Button onClick={redo} disabled={redoStack.length === 0} startIcon={<Redo />} aria-label="やり直し">
              やり直し
            </Button>
            <Button
              variant="outlined"
              startIcon={<Visibility />}
              disabled={isNew}
              onClick={() => id && navigate(`/master/sops/${id}/preview`)}
              aria-label="プレビューを開く"
            >
              プレビュー
            </Button>
            <Button
              variant="contained"
              startIcon={<Send />}
              disabled={isNew || requestReview.isPending}
              onClick={() => requestReview.mutate()}
              aria-label="レビュー依頼"
            >
              レビュー依頼
            </Button>
          </Stack>
        }
      />

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <Stack direction="row" spacing={2}>
        {/* 左ペイン: Step ツリー */}
        <Paper sx={{ p: 2, width: 280 }} elevation={1}>
          <Typography variant="h3" gutterBottom>
            Steps ({steps.length})
          </Typography>
          <Button fullWidth variant="outlined" onClick={handleAddStep} sx={{ mb: 2 }} aria-label="Step を追加">
            Step を追加
          </Button>
          <Stack spacing={0.5} role="list" aria-label="Step 一覧">
            {steps.map((s) => (
              <Paper
                key={s.id}
                sx={{
                  p: 1,
                  cursor: 'pointer',
                  backgroundColor: editingStepId === s.id ? 'action.selected' : undefined,
                }}
                role="listitem"
                onClick={() => setEditingStepId(s.id)}
                onKeyDown={(e) => e.key === 'Enter' && setEditingStepId(s.id)}
                tabIndex={0}
                aria-label={`Step ${s.stepNumber} ${resolveLocale(s.titleJson, 'ja')}`}
              >
                <Typography variant="body2">
                  {s.stepNumber}. {resolveLocale(s.titleJson, 'ja') || '(無題)'}
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  {s.stepType}
                </Typography>
              </Paper>
            ))}
          </Stack>
        </Paper>

        {/* 中央ペイン: Step 編集 or DAG */}
        <Paper sx={{ p: 2, flex: 1 }} elevation={1}>
          <Tabs
            value={centerMode}
            onChange={(_, v: CenterMode) => setCenterMode(v)}
            aria-label="編集モード切替"
          >
            <Tab value="step-form" label="Step 編集" />
            <Tab value="dag-flow" label="DAG フロー" />
          </Tabs>
          <Divider />
          <Box mt={2}>
            {centerMode === 'step-form' ? (
              selectedStep ? (
                <StepEditor
                  step={selectedStep}
                  onChange={handleUpdateStep}
                  onDelete={() => handleDeleteStep(selectedStep.id)}
                />
              ) : (
                <Alert severity="info">左ペインから Step を選択してください</Alert>
              )
            ) : (
              <SopFlowEditor
                steps={steps}
                flow={flow}
                onFlowChange={setFlow}
              />
            )}
          </Box>
        </Paper>
      </Stack>

      {isNew && (
        <Stack direction="row" spacing={1} mt={2}>
          <Button
            variant="contained"
            onClick={() =>
              saveSopMutation.mutate({
                sopCode: `SOP-${Date.now()}`,
                nameJson: { ja: '新規 SOP', en: '', zh: '' },
                descriptionJson: { ja: '', en: '', zh: '' },
                sopType: 'STANDARD',
                processId: '',
                operationId: '',
              })
            }
            disabled={saveSopMutation.isPending}
            aria-label="SOP を保存"
          >
            SOP を保存
          </Button>
          <Typography variant="caption" color="text.secondary">
            保存後にレビュー依頼が可能になります（即時公開禁止）
          </Typography>
        </Stack>
      )}
    </Box>
  );
}

// Step 単体エディタ（中央ペイン step-form モード）
function StepEditor({
  step,
  onChange,
  onDelete,
}: {
  step: Step;
  onChange: (next: Step) => void;
  onDelete: () => void;
}): React.ReactElement {
  return (
    <Stack spacing={2}>
      <Typography variant="h3">Step {step.stepNumber}</Typography>
      <LocalizedTextField
        label="Step タイトル"
        value={step.titleJson}
        onChange={(v) => onChange({ ...step, titleJson: v })}
        required
        maxLength={200}
      />
      <LocalizedTextField
        label="作業指示"
        value={step.instructionJson}
        onChange={(v) => onChange({ ...step, instructionJson: v })}
        multiline
        rows={4}
        required
        maxLength={2000}
      />
      <TextField
        label="所要時間（秒）"
        type="number"
        value={step.estimatedSeconds}
        onChange={(e) => onChange({ ...step, estimatedSeconds: Number(e.target.value) })}
        inputProps={{ min: 1, max: 86400, 'aria-label': '所要時間秒' }}
      />
      {step.stepType === 'branching' && (
        <Box>
          <Typography variant="caption" color="text.secondary">
            分岐条件（JSON Logic）
          </Typography>
          <DslConditionBuilder
            value={step.payload}
            onChange={(next) => onChange({ ...step, payload: next })}
          />
        </Box>
      )}
      <Button color="error" variant="outlined" onClick={onDelete} aria-label="Step を削除">
        Step を削除
      </Button>
    </Stack>
  );
}
