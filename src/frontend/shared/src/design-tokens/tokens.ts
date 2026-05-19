// 全デザイントークンを単一ソースで管理する（docs/04 §03_画面設計/05_共通UIコンポーネントとデザインシステム.md §1）
// terminal/master 両方で参照される。React Native と Web で型定義を共有するため値のみのオブジェクトとする。

export type ThemeMode = 'light' | 'dark' | 'nightShift';

export const brand = {
  primary: {
    light: {
      50: '#EEF2F7',
      100: '#CCD7E8',
      200: '#9AB0D1',
      300: '#6789B9',
      400: '#3E65A2',
      500: '#1E3A5F',
      600: '#193358',
      700: '#132948',
      800: '#0D1E37',
      900: '#071226',
    },
    dark: {
      50: '#111A27',
      100: '#182334',
      200: '#1F3047',
      300: '#2B4568',
      400: '#3E65A2',
      500: '#4D7AB3',
      600: '#6B96C9',
      700: '#8FB5E0',
      800: '#B3D3F2',
      900: '#D9ECFC',
    },
  },
  accent: {
    light: {
      50: '#EBF3FE',
      100: '#C5DCF9',
      200: '#9EC5F5',
      300: '#77AEF1',
      400: '#5A9DF4',
      500: '#3D8EF0',
      600: '#2E7ADE',
      700: '#2264C2',
      800: '#1850A6',
      900: '#0F3D8A',
    },
    dark: {
      50: '#0D1F3C',
      100: '#132D52',
      200: '#1A3D6B',
      300: '#2254A3',
      400: '#3D7FDC',
      500: '#5A9DF4',
      600: '#77AEF1',
      700: '#9EC5F5',
      800: '#C5DCF9',
      900: '#EBF3FE',
    },
  },
} as const;

export const state = {
  success: {
    light: { 50: '#ECFDF5', 100: '#D1FAE5', 300: '#6EE7B7', 500: '#059669', 700: '#065F46' },
    dark: { 50: '#052E16', 100: '#064E23', 300: '#059669', 500: '#34D399', 700: '#6EE7B7' },
  },
  warning: {
    light: { 50: '#FFFBEB', 100: '#FEF3C7', 300: '#FCD34D', 500: '#F59E0B', 700: '#92400E' },
    dark: { 50: '#2D1A00', 100: '#3D2500', 300: '#D97706', 500: '#F5A524', 700: '#FCD34D' },
  },
  danger: {
    light: { 50: '#FEF2F2', 100: '#FEE2E2', 300: '#FCA5A5', 500: '#DC2626', 700: '#991B1B' },
    dark: { 50: '#2D0000', 100: '#400000', 300: '#B91C1C', 500: '#F87171', 700: '#FCA5A5' },
  },
  destructive: {
    light: { 50: '#FFF1F2', 500: '#E11D48', 700: '#9F1239' },
    dark: { 50: '#2D0010', 500: '#FB7185', 700: '#FDA4AF' },
  },
  info: {
    light: { 50: '#F0F9FF', 500: '#0284C7', 700: '#0369A1' },
    dark: { 50: '#0A1A2D', 500: '#38BDF8', 700: '#7DD3FC' },
  },
  andon: {
    light: { base: '#FF0000', pulse: '#CC0000', bg: '#FFF0F0', text: '#FFFFFF' },
    dark: { base: '#FF4444', pulse: '#CC3333', bg: '#2D0000', text: '#FFFFFF' },
  },
} as const;

export const neutral = {
  0: '#FFFFFF',
  50: '#F8FAFC',
  100: '#F1F5F9',
  200: '#E2E8F0',
  300: '#CBD5E1',
  400: '#94A3B8',
  500: '#64748B',
  600: '#475569',
  700: '#334155',
  800: '#1E293B',
  900: '#0F172A',
  950: '#03050A',
  1000: '#000000',
} as const;

export const surface = {
  light: {
    bg: '#FFFFFF',
    raised: '#FFFFFF',
    subtle: '#F8FAFC',
    sunken: '#F1F5F9',
    overlay: 'rgba(15,23,42,.5)',
    divider: '#E2E8F0',
    disabled: '#F1F5F9',
  },
  dark: {
    bg: '#0F172A',
    raised: '#1E293B',
    subtle: '#1E293B',
    sunken: '#0F172A',
    overlay: 'rgba(0,0,0,.7)',
    divider: '#334155',
    disabled: '#1E293B',
  },
  nightShift: {
    bg: '#0B1220',
    raised: '#131E30',
    subtle: '#1A2840',
    sunken: '#070F1A',
    overlay: 'rgba(0,0,0,.8)',
    divider: '#1F2E42',
    disabled: '#131E30',
  },
} as const;

export const text = {
  light: {
    primary: '#0F172A',
    secondary: '#475569',
    tertiary: '#94A3B8',
    inverse: '#F8FAFC',
    link: '#2E7ADE',
    disabled: '#CBD5E1',
    danger: '#DC2626',
    warning: '#92400E',
    success: '#065F46',
  },
  dark: {
    primary: '#F1F5F9',
    secondary: '#94A3B8',
    tertiary: '#64748B',
    inverse: '#0F172A',
    link: '#77AEF1',
    disabled: '#334155',
    danger: '#F87171',
    warning: '#F5A524',
    success: '#34D399',
  },
  nightShift: {
    primary: '#E2E8F0',
    secondary: '#7B93AB',
    tertiary: '#4E6880',
    inverse: '#0B1220',
    link: '#77AEF1',
    disabled: '#283A4E',
    danger: '#F87171',
    warning: '#F5A524',
    success: '#34D399',
  },
} as const;

export const typography = {
  family: {
    sansJp: '"Noto Sans JP", "Noto Sans", sans-serif',
    sans: '"Inter", "Noto Sans", sans-serif',
    mono: '"JetBrains Mono", "Courier New", monospace',
  },
  size: {
    displayXl: 40,
    displayLg: 32,
    h1: 24,
    h2: 20,
    h3: 18,
    bodyLg: 16,
    body: 14,
    caption: 12,
    overline: 11,
    code: 13,
  },
  weight: {
    regular: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },
  lineHeight: {
    displayXl: 1.1,
    displayLg: 1.1,
    h1: 1.2,
    h2: 1.3,
    h3: 1.3,
    bodyLg: 1.6,
    body: 1.6,
    caption: 1.5,
    overline: 1.4,
    code: 1.6,
  },
  letterSpacing: {
    displayXl: '-0.02em',
    displayLg: '-0.02em',
    h1: '-0.01em',
    h2: '-0.01em',
    h3: '-0.005em',
    body: '0',
    overline: '0.08em',
  },
} as const;

export const spacing = {
  0: 0,
  0.5: 2,
  1: 4,
  2: 8,
  3: 12,
  4: 16,
  5: 20,
  6: 24,
  8: 32,
  10: 40,
  12: 48,
  16: 64,
  20: 80,
} as const;

export const radius = {
  none: 0,
  xs: 2,
  sm: 4,
  md: 8,
  lg: 12,
  xl: 16,
  '2xl': 24,
  full: 9999,
} as const;

export const elevation = {
  light: {
    0: 'none',
    1: '0 1px 2px rgba(15,23,42,.04), 0 1px 1px rgba(15,23,42,.06)',
    2: '0 4px 8px rgba(15,23,42,.08), 0 2px 4px rgba(15,23,42,.06)',
    3: '0 8px 24px rgba(15,23,42,.12), 0 4px 8px rgba(15,23,42,.08)',
    4: '0 16px 40px rgba(15,23,42,.16), 0 8px 16px rgba(15,23,42,.10)',
    popup: '0 24px 60px rgba(15,23,42,.20), 0 12px 24px rgba(15,23,42,.12)',
  },
  dark: {
    0: 'none',
    1: '0 1px 2px rgba(0,0,0,.30), 0 1px 1px rgba(0,0,0,.40)',
    2: '0 4px 12px rgba(0,0,0,.50), 0 2px 6px rgba(0,0,0,.40)',
    3: '0 8px 24px rgba(0,0,0,.55), 0 4px 8px rgba(0,0,0,.45)',
    4: '0 16px 40px rgba(0,0,0,.60), 0 8px 16px rgba(0,0,0,.50)',
    popup: '0 24px 60px rgba(0,0,0,.70), 0 12px 24px rgba(0,0,0,.60)',
  },
} as const;

export const motion = {
  duration: {
    fast: 120,
    base: 200,
    slow: 320,
    xslow: 480,
  },
  easing: {
    standard: 'cubic-bezier(0.2, 0, 0, 1)',
    decelerated: 'cubic-bezier(0, 0, 0, 1)',
    accelerated: 'cubic-bezier(0.3, 0, 1, 1)',
    emphasized: 'cubic-bezier(0.2, 0, 0, 1)',
  },
} as const;

export const focusRing = {
  width: 2,
  offset: 2,
  light: { color: brand.accent.light[600], gap: surface.light.bg },
  dark: { color: brand.accent.dark[400], gap: surface.dark.bg },
} as const;

export const border = {
  width: {
    default: 1,
    strong: 2,
  },
  color: {
    light: {
      subtle: neutral[200],
      default: neutral[300],
      strong: neutral[400],
      focus: brand.accent.light[600],
      error: state.danger.light[500],
      success: state.success.light[500],
      warning: state.warning.light[500],
    },
    dark: {
      subtle: neutral[700],
      default: neutral[600],
      strong: neutral[500],
      focus: brand.accent.dark[400],
      error: state.danger.dark[500],
      success: state.success.dark[500],
      warning: state.warning.dark[500],
    },
  },
} as const;

export const icon = {
  size: {
    xs: 12,
    sm: 16,
    md: 20,
    lg: 24,
    xl: 32,
    '2xl': 40,
  },
  strokeWidth: 1.5,
} as const;

export const tokens = {
  brand,
  state,
  neutral,
  surface,
  text,
  typography,
  spacing,
  radius,
  elevation,
  motion,
  focusRing,
  border,
  icon,
} as const;

export type Tokens = typeof tokens;
