// 対応 §: ロードマップ §3.6.2 §11.3 §11.2.2 §10.6
// ヘッダ専用コンポーネント。下記を 1 行に保ち、視線移動を最小化する:
//   - 識別子（ユーザ／タスク状態）／オンライン状態 / ロケール / ドロワー切替 / ログアウト
// ドロワー切替は `IconButton + aria-label` を使い、文字列ベタ書き UI を排除する。

import { palette, fontSize, fontWeight, radius, space } from '../../../tokens/access';
import { Icon } from '../icon/icon';
import type { IconName } from '../icon/glyphs';
import { IconButton } from '../primitives/icon-button';
import { StatusPill } from '../primitives/status-pill';
import { LOCALE_KEYS, type LocaleKey } from '../../../i18n';
import type { ThemeMode } from '../../hooks/use-theme';

export interface NavigationHeaderProps {
  userDisplayName: string;
  userId: string;
  selectedTaskState: string;
  selectedTaskId: string | null;
  taskStatePrefix: string;
  taskIdPrefix: string;
  taskIdUnselectedLabel: string;
  online: boolean;
  onlineLabel: string;
  offlineLabel: string;
  networkAriaLabel: string;
  locale: LocaleKey;
  onLocaleChange(locale: LocaleKey): void;
  onLogout(): void;
  logoutLabel: string;
  taskDrawerOpen: boolean;
  stepMapDrawerOpen: boolean;
  onToggleTaskDrawer(): void;
  onToggleStepMapDrawer(): void;
  taskDrawerLabel: string;
  stepMapDrawerLabel: string;
  themeMode: ThemeMode;
  onCycleTheme(): void;
  themeLabel: string;
  onShowShortcuts(): void;
  shortcutsLabel: string;
}

function themeIcon(mode: ThemeMode): IconName {
  if (mode === 'outdoor') return 'sun';
  if (mode === 'dark') return 'moon';
  if (mode === 'auto') return 'theme-auto';
  return 'sun';
}

export function NavigationHeader(props: NavigationHeaderProps): JSX.Element {
  return (
    <header
      style={{
        gridArea: 'header',
        display: 'flex',
        alignItems: 'center',
        gap: space[3],
        padding: `${space[2]} ${space[4]}`,
        background: palette.white,
        borderBottom: `1px solid ${palette.neutral[200]}`,
        minHeight: 64
      }}
    >
      <IconButton
        iconName="menu"
        ariaLabel={props.taskDrawerLabel}
        onClick={props.onToggleTaskDrawer}
        selected={props.taskDrawerOpen}
      />
      <Icon name="map-pin" size={20} color={palette.info.default} />
      <strong style={{ fontSize: fontSize.body, fontWeight: fontWeight.medium, color: palette.neutral[900] }}>
        {props.userDisplayName}
        <span style={{ marginLeft: space[1], color: palette.neutral[500], fontWeight: fontWeight.regular, fontSize: fontSize.caption }}>
          ({props.userId})
        </span>
      </strong>
      <span style={{ color: palette.neutral[300] }}>|</span>
      <span style={{ fontSize: fontSize.caption, color: palette.neutral[600] }}>
        {props.taskStatePrefix}: <strong style={{ color: palette.neutral[800] }}>{props.selectedTaskState}</strong>
      </span>
      <span style={{ fontSize: fontSize.caption, color: palette.neutral[600] }}>
        {props.taskIdPrefix}: <strong style={{ color: palette.neutral[800] }}>{props.selectedTaskId ?? props.taskIdUnselectedLabel}</strong>
      </span>
      <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: space[3] }}>
        <StatusPill
          toneName={props.online ? 'success' : 'danger'}
          iconName={props.online ? 'wifi' : 'wifi-off'}
          ariaLabel={props.networkAriaLabel}
          size="sm"
        >
          {props.online ? props.onlineLabel : props.offlineLabel}
        </StatusPill>
        <label
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: space[1],
            padding: `${space[1]} ${space[2]}`,
            border: `1px solid ${palette.neutral[300]}`,
            borderRadius: radius.medium,
            color: palette.neutral[700],
            fontSize: fontSize.caption
          }}
        >
          <Icon name="globe" size={16} />
          <select
            aria-label="locale"
            value={props.locale}
            onChange={(e) => props.onLocaleChange(e.target.value as LocaleKey)}
            style={{
              background: 'transparent',
              border: 'none',
              fontSize: fontSize.caption,
              color: palette.neutral[800]
            }}
          >
            {LOCALE_KEYS.map((l) => (
              <option key={l} value={l}>
                {l.toUpperCase()}
              </option>
            ))}
          </select>
        </label>
        <IconButton
          iconName={themeIcon(props.themeMode)}
          ariaLabel={`${props.themeLabel}: ${props.themeMode}`}
          onClick={props.onCycleTheme}
          title={`${props.themeLabel}: ${props.themeMode}`}
        />
        <IconButton
          iconName="keyboard"
          ariaLabel={props.shortcutsLabel}
          onClick={props.onShowShortcuts}
          title={props.shortcutsLabel}
        />
        <IconButton
          iconName="list"
          ariaLabel={props.stepMapDrawerLabel}
          onClick={props.onToggleStepMapDrawer}
          selected={props.stepMapDrawerOpen}
        />
        <IconButton iconName="logout" ariaLabel={props.logoutLabel} onClick={props.onLogout} />
      </div>
    </header>
  );
}
