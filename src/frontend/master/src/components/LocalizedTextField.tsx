import type React from 'react';
import { useState } from 'react';
import { Tabs, Tab, Box, TextField } from '@mui/material';
import type { Locale, LocalizedText } from '@wnav/shared/types';

const LOCALES: Locale[] = ['ja', 'en', 'zh'];
const LOCALE_LABEL: Record<Locale, string> = { ja: '日本語', en: 'English', zh: '中文' };

// 多言語入力（JSONB 形式 {ja,en,zh}）。タブ切替で 3 言語を編集する。
export function LocalizedTextField({
  label,
  value,
  onChange,
  multiline = false,
  rows = 1,
  required = false,
  maxLength,
}: {
  label: string;
  value: LocalizedText;
  onChange: (next: LocalizedText) => void;
  multiline?: boolean;
  rows?: number;
  required?: boolean;
  maxLength?: number;
}): React.ReactElement {
  const [active, setActive] = useState<Locale>('ja');
  return (
    <Box>
      <Tabs value={active} onChange={(_, v: Locale) => setActive(v)} aria-label={`${label} 言語切替`}>
        {LOCALES.map((l) => (
          <Tab key={l} value={l} label={LOCALE_LABEL[l]} />
        ))}
      </Tabs>
      <Box mt={1}>
        <TextField
          fullWidth
          label={`${label}（${LOCALE_LABEL[active]}）`}
          value={value[active] ?? ''}
          onChange={(e) => onChange({ ...value, [active]: e.target.value })}
          multiline={multiline}
          rows={rows}
          required={required && active === 'ja'}
          inputProps={{
            ...(maxLength ? { maxLength } : {}),
            'aria-label': `${label} ${LOCALE_LABEL[active]} 入力`,
          }}
        />
      </Box>
    </Box>
  );
}
