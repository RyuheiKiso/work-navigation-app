import { createTheme, type Theme } from '@mui/material/styles';
import {
  brand,
  state,
  surface,
  text,
  typography,
  spacing,
  radius,
  elevation,
} from '@wnav/shared/design-tokens';

// shared/design-tokens を MUI createTheme に注入し terminal と UI を同期させる（src/CLAUDE.md）
function buildTheme(mode: 'light' | 'dark'): Theme {
  const isLight = mode === 'light';
  const brandPrimary = isLight ? brand.primary.light : brand.primary.dark;
  const brandAccent = isLight ? brand.accent.light : brand.accent.dark;
  const stateSuccess = isLight ? state.success.light : state.success.dark;
  const stateWarning = isLight ? state.warning.light : state.warning.dark;
  const stateDanger = isLight ? state.danger.light : state.danger.dark;
  const stateInfo = isLight ? state.info.light : state.info.dark;
  const surfaceTokens = isLight ? surface.light : surface.dark;
  const textTokens = isLight ? text.light : text.dark;
  const elevationTokens = isLight ? elevation.light : elevation.dark;

  return createTheme({
    palette: {
      mode,
      primary: {
        main: brandPrimary[500],
        light: brandPrimary[300],
        dark: brandPrimary[700],
        contrastText: textTokens.inverse,
      },
      secondary: {
        main: brandAccent[500],
        light: brandAccent[300],
        dark: brandAccent[700],
        contrastText: textTokens.inverse,
      },
      success: { main: stateSuccess[500], light: stateSuccess[300], dark: stateSuccess[700] },
      warning: { main: stateWarning[500], light: stateWarning[300], dark: stateWarning[700] },
      error: { main: stateDanger[500], light: stateDanger[300], dark: stateDanger[700] },
      info: { main: stateInfo[500] },
      background: { default: surfaceTokens.bg, paper: surfaceTokens.raised },
      divider: surfaceTokens.divider,
      text: {
        primary: textTokens.primary,
        secondary: textTokens.secondary,
        disabled: textTokens.disabled,
      },
    },
    typography: {
      fontFamily: typography.family.sansJp,
      htmlFontSize: 16,
      h1: { fontSize: typography.size.h1, fontWeight: typography.weight.semibold, lineHeight: typography.lineHeight.h1, letterSpacing: typography.letterSpacing.h1 },
      h2: { fontSize: typography.size.h2, fontWeight: typography.weight.semibold, lineHeight: typography.lineHeight.h2, letterSpacing: typography.letterSpacing.h2 },
      h3: { fontSize: typography.size.h3, fontWeight: typography.weight.medium, lineHeight: typography.lineHeight.h3, letterSpacing: typography.letterSpacing.h3 },
      body1: { fontSize: typography.size.body, lineHeight: typography.lineHeight.body },
      body2: { fontSize: typography.size.caption, lineHeight: typography.lineHeight.caption },
      caption: { fontSize: typography.size.caption, lineHeight: typography.lineHeight.caption },
      button: { fontWeight: typography.weight.medium, textTransform: 'none' },
    },
    shape: { borderRadius: radius.md },
    spacing: spacing[1],
    components: {
      MuiCssBaseline: {
        styleOverrides: {
          'html, body, #root': { height: '100%' },
          body: { backgroundColor: surfaceTokens.bg, color: textTokens.primary },
        },
      },
      MuiButton: { defaultProps: { disableElevation: true } },
      MuiPaper: {
        styleOverrides: {
          root: { backgroundImage: 'none' },
          elevation1: { boxShadow: elevationTokens[1] },
          elevation2: { boxShadow: elevationTokens[2] },
        },
      },
    },
  });
}

export const lightTheme = buildTheme('light');
export const darkTheme = buildTheme('dark');
