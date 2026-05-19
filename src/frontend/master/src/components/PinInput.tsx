import type React from 'react';
import { TextField } from '@mui/material';

// 4〜8 桁数字 PIN 入力。サーバー側で TOTP 等の二要素ではなく、デバイスローカルな承認弱認証として使用（FR-MA-009）。
export function PinInput({
  value,
  onChange,
  label = 'PIN',
}: {
  value: string;
  onChange: (next: string) => void;
  label?: string;
}): React.ReactElement {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    const next = e.target.value.replace(/[^0-9]/g, '').slice(0, 8);
    onChange(next);
  };
  return (
    <TextField
      label={label}
      type="password"
      value={value}
      onChange={handleChange}
      inputProps={{
        inputMode: 'numeric',
        pattern: '[0-9]*',
        maxLength: 8,
        'aria-label': 'PIN コード',
      }}
      autoComplete="off"
      required
    />
  );
}
