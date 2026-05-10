// 対応 §: ロードマップ §9.5.1 §11.2.2
// 形状で識別する SVG パス集（24 単位グリッド）。
// `tokens.icon.semantic.shape` (triangle / circle / octagon / shield / envelope / stack)
// と整合する形を採用し、色覚多様性 P/D/T でも区別がつくようにする（§11.2.2 SC 1.4.1）。
// `currentColor` で塗るため、呼び側で `color` を指定すれば任意配色になる。

export type GlyphPath = ReadonlyArray<string>;

export interface Glyph {
  // viewBox は全て 24x24 に正規化
  paths: GlyphPath;
  // fill か stroke のいずれかで描画。輪郭主体は stroke、塗り主体は fill。
  mode: 'fill' | 'stroke';
}

export type IconName =
  | 'check'
  | 'play'
  | 'pause'
  | 'circle-filled'
  | 'circle-outline'
  | 'menu'
  | 'close'
  | 'chevron-left'
  | 'chevron-right'
  | 'globe'
  | 'lock-closed'
  | 'logout'
  | 'mic'
  | 'clipboard'
  | 'inbox-empty'
  | 'sparkle'
  | 'note'
  | 'warning-triangle'
  | 'octagon-exclamation'
  | 'andon-tower'
  | 'shield-check'
  | 'wifi'
  | 'wifi-off'
  | 'user'
  | 'map-pin'
  | 'list'
  | 'sun'
  | 'moon'
  | 'theme-auto'
  | 'image'
  | 'video'
  | 'diagram'
  | 'keyboard'
  | 'help-circle'
  | 'sparkle-burst';

export const GLYPHS: Readonly<Record<IconName, Glyph>> = {
  check: {
    mode: 'stroke',
    paths: ['M5 12.5 L10 17.5 L19 6.5']
  },
  play: {
    mode: 'fill',
    paths: ['M7 5 L19 12 L7 19 Z']
  },
  pause: {
    mode: 'fill',
    paths: ['M7 5 H10 V19 H7 Z', 'M14 5 H17 V19 H14 Z']
  },
  'circle-filled': {
    mode: 'fill',
    paths: ['M12 6 a6 6 0 1 0 0.001 0 Z']
  },
  'circle-outline': {
    mode: 'stroke',
    paths: ['M12 5 a7 7 0 1 0 0.001 0 Z']
  },
  menu: {
    mode: 'stroke',
    paths: ['M4 7 H20', 'M4 12 H20', 'M4 17 H20']
  },
  close: {
    mode: 'stroke',
    paths: ['M6 6 L18 18', 'M18 6 L6 18']
  },
  'chevron-left': {
    mode: 'stroke',
    paths: ['M15 5 L8 12 L15 19']
  },
  'chevron-right': {
    mode: 'stroke',
    paths: ['M9 5 L16 12 L9 19']
  },
  globe: {
    mode: 'stroke',
    paths: [
      'M12 3 a9 9 0 1 0 0.001 0 Z',
      'M3 12 H21',
      'M12 3 C8 7 8 17 12 21',
      'M12 3 C16 7 16 17 12 21'
    ]
  },
  'lock-closed': {
    mode: 'stroke',
    paths: [
      'M6 11 H18 V20 H6 Z',
      'M9 11 V8 a3 3 0 0 1 6 0 V11'
    ]
  },
  logout: {
    mode: 'stroke',
    paths: [
      'M14 4 H5 V20 H14',
      'M11 12 H21',
      'M17 8 L21 12 L17 16'
    ]
  },
  mic: {
    mode: 'stroke',
    paths: [
      'M12 3 a3 3 0 0 0 -3 3 V12 a3 3 0 0 0 6 0 V6 a3 3 0 0 0 -3 -3 Z',
      'M5 11 a7 7 0 0 0 14 0',
      'M12 18 V22',
      'M8 22 H16'
    ]
  },
  clipboard: {
    mode: 'stroke',
    paths: [
      'M8 4 H16 V7 H8 Z',
      'M5 6 H8 M16 6 H19 V21 H5 V6',
      'M9 11 H17',
      'M9 15 H17'
    ]
  },
  'inbox-empty': {
    mode: 'stroke',
    paths: [
      'M4 13 H9 L11 16 H13 L15 13 H20',
      'M4 13 L6 4 H18 L20 13 V20 H4 Z'
    ]
  },
  sparkle: {
    mode: 'fill',
    paths: ['M12 3 L14 10 L21 12 L14 14 L12 21 L10 14 L3 12 L10 10 Z']
  },
  note: {
    mode: 'stroke',
    paths: [
      'M5 4 H15 L19 8 V20 H5 Z',
      'M15 4 V8 H19',
      'M8 13 H16',
      'M8 17 H13'
    ]
  },
  // §11.2.2 形状: triangle（警告）
  'warning-triangle': {
    mode: 'fill',
    paths: ['M12 3 L22 20 H2 Z']
  },
  // §11.2.2 形状: octagon（危険）
  'octagon-exclamation': {
    mode: 'fill',
    paths: ['M8 3 H16 L21 8 V16 L16 21 H8 L3 16 V8 Z']
  },
  // §17 Andon シグナルタワー — 5 段の積層
  'andon-tower': {
    mode: 'fill',
    paths: [
      'M9 3 H15 V6 H9 Z',
      'M9 7 H15 V10 H9 Z',
      'M9 11 H15 V14 H9 Z',
      'M9 15 H15 V18 H9 Z',
      'M7 19 H17 V21 H7 Z'
    ]
  },
  // §11.2.2 形状: shield（監査）
  'shield-check': {
    mode: 'stroke',
    paths: [
      'M12 3 L20 6 V12 C20 17 16 20 12 21 C8 20 4 17 4 12 V6 Z',
      'M8 12 L11 15 L16 9'
    ]
  },
  wifi: {
    mode: 'stroke',
    paths: [
      'M3 9 C8 4 16 4 21 9',
      'M6 12 C9 9 15 9 18 12',
      'M9 15 C10.5 13.5 13.5 13.5 15 15',
      'M12 18 v0.01'
    ]
  },
  'wifi-off': {
    mode: 'stroke',
    paths: [
      'M3 9 C8 4 16 4 21 9',
      'M6 12 C9 9 15 9 18 12',
      'M9 15 C10.5 13.5 13.5 13.5 15 15',
      'M12 18 v0.01',
      'M3 3 L21 21'
    ]
  },
  user: {
    mode: 'stroke',
    paths: [
      'M12 4 a4 4 0 1 0 0.001 0 Z',
      'M4 21 C4 16 8 14 12 14 C16 14 20 16 20 21'
    ]
  },
  'map-pin': {
    mode: 'stroke',
    paths: [
      'M12 22 C7 16 4 13 4 9 a8 8 0 0 1 16 0 C20 13 17 16 12 22 Z',
      'M12 11 a2 2 0 1 0 0.001 0 Z'
    ]
  },
  list: {
    mode: 'stroke',
    paths: [
      'M4 6 H20',
      'M4 12 H20',
      'M4 18 H20'
    ]
  },
  // テーマ: 屋外モード（高輝度）
  sun: {
    mode: 'stroke',
    paths: [
      'M12 7 a5 5 0 1 0 0.001 0 Z',
      'M12 2 V4',
      'M12 20 V22',
      'M2 12 H4',
      'M20 12 H22',
      'M5 5 L6.5 6.5',
      'M17.5 17.5 L19 19',
      'M19 5 L17.5 6.5',
      'M6.5 17.5 L5 19'
    ]
  },
  // テーマ: 暗所モード
  moon: {
    mode: 'fill',
    paths: ['M20 14 a8 8 0 1 1 -10 -10 a6 6 0 0 0 10 10 Z']
  },
  // テーマ: OS 連動
  'theme-auto': {
    mode: 'stroke',
    paths: [
      'M12 4 a8 8 0 1 0 0.001 0 Z',
      'M12 4 V20'
    ]
  },
  // メディア: 写真
  image: {
    mode: 'stroke',
    paths: [
      'M4 5 H20 V19 H4 Z',
      'M8 12 a2 2 0 1 0 0.001 0 Z',
      'M4 17 L9 12 L13 16 L16 13 L20 17'
    ]
  },
  // メディア: 動画
  video: {
    mode: 'stroke',
    paths: [
      'M3 7 H15 V17 H3 Z',
      'M15 10 L21 7 V17 L15 14 Z'
    ]
  },
  // メディア: 図面
  diagram: {
    mode: 'stroke',
    paths: [
      'M4 4 H20 V20 H4 Z',
      'M4 9 H20',
      'M9 9 V20',
      'M14 14 H20'
    ]
  },
  keyboard: {
    mode: 'stroke',
    paths: [
      'M3 7 H21 V17 H3 Z',
      'M6 10 V10.01',
      'M10 10 V10.01',
      'M14 10 V10.01',
      'M18 10 V10.01',
      'M7 14 H17'
    ]
  },
  'help-circle': {
    mode: 'stroke',
    paths: [
      'M12 4 a8 8 0 1 0 0.001 0 Z',
      'M9.5 9 a2.5 2.5 0 1 1 3.5 2.3 C12 12 12 13 12 14',
      'M12 17 V17.01'
    ]
  },
  // 完了演出用の放射スパーク
  'sparkle-burst': {
    mode: 'stroke',
    paths: [
      'M12 2 V6',
      'M12 18 V22',
      'M2 12 H6',
      'M18 12 H22',
      'M5 5 L8 8',
      'M16 16 L19 19',
      'M19 5 L16 8',
      'M8 16 L5 19'
    ]
  }
};
