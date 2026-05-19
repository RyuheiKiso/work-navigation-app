// SQLite は INTEGER で boolean を表現するため、0/1 ↔ false/true の相互変換が必要
export const boolTransformer = {
  to: (v: boolean): number => (v ? 1 : 0),
  from: (v: number | boolean): boolean => Boolean(v),
};
