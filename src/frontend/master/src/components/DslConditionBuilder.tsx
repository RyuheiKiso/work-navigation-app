import type React from 'react';
import { useMemo } from 'react';
import Editor from '@monaco-editor/react';
import { Box, Alert, Typography } from '@mui/material';

// JSON Logic DSL エディタ。eval() 禁止規約（src/CLAUDE.md §動的評価禁止）に従い、文字列を JSON.parse でのみ評価する。
// バリデーションは json-logic-js のホワイトリスト演算子で行う（実装はタスク #10）。
const WHITELIST_OPS = ['==', '!=', '<', '>', '<=', '>=', 'and', 'or', '!', 'in', 'var', 'if', '+', '-', '*', '/', '%'] as const;

export function DslConditionBuilder({
  value,
  onChange,
  readOnly = false,
  height = 240,
}: {
  value: string;
  onChange: (next: string) => void;
  readOnly?: boolean;
  height?: number;
}): React.ReactElement {
  const validation = useMemo<{ ok: boolean; message: string }>(() => {
    try {
      const parsed: unknown = JSON.parse(value || 'null');
      if (parsed === null) return { ok: true, message: '空式（true 扱い）' };
      if (typeof parsed !== 'object') return { ok: false, message: 'オブジェクト形式である必要があります' };
      const op = Object.keys(parsed as Record<string, unknown>)[0];
      if (op && !WHITELIST_OPS.includes(op as (typeof WHITELIST_OPS)[number])) {
        return { ok: false, message: `演算子 "${op}" はホワイトリスト外です` };
      }
      return { ok: true, message: '構文 OK' };
    } catch (e) {
      return { ok: false, message: e instanceof Error ? e.message : 'JSON パース失敗' };
    }
  }, [value]);

  return (
    <Box>
      <Typography variant="caption" color="text.secondary">
        JSON Logic 式（eval 不使用・ホワイトリスト演算子のみ）
      </Typography>
      <Editor
        height={height}
        defaultLanguage="json"
        value={value}
        onChange={(next) => onChange(next ?? '')}
        options={{
          readOnly,
          minimap: { enabled: false },
          fontSize: 13,
          lineNumbers: 'on',
          scrollBeyondLastLine: false,
        }}
      />
      <Alert severity={validation.ok ? 'success' : 'error'} sx={{ mt: 1 }}>
        {validation.message}
      </Alert>
    </Box>
  );
}
