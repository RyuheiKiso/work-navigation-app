// GlanceableBanner の表示要素を 200ms 設計どおりに視認可能か検証
import React from 'react';
import { render } from '@testing-library/react-native';
import { GlanceableBanner } from '../../ui/GlanceableBanner';

describe('GlanceableBanner', () => {
  it('renders current and next step labels', () => {
    const { getByLabelText } = render(
      <GlanceableBanner currentStepLabel="Step 1" nextStepLabel="Step 2" />,
    );
    expect(getByLabelText('現在のステップ')).toBeTruthy();
    expect(getByLabelText('次のステップ')).toBeTruthy();
  });

  it('shows alert banner when alertLabel provided', () => {
    const { getByLabelText } = render(
      <GlanceableBanner currentStepLabel="Step 1" alertLabel="温度異常" />,
    );
    expect(getByLabelText('異常アラート')).toBeTruthy();
  });
});
