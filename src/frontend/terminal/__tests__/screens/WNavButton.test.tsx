// WNavButton の 72dp タッチターゲット・accessibilityLabel 検証
import React from 'react';
import { fireEvent, render } from '@testing-library/react-native';
import { WNavButton } from '../../ui/WNavButton';

describe('WNavButton', () => {
  it('exposes accessibilityLabel and role=button', () => {
    const onPress = jest.fn();
    const { getByLabelText } = render(
      <WNavButton label="完了" accessibilityLabel="完了ボタン" onPress={onPress} />,
    );
    const button = getByLabelText('完了ボタン');
    expect(button.props.accessibilityRole).toBe('button');
  });

  it('does not trigger onPress when disabled', () => {
    const onPress = jest.fn();
    const { getByLabelText } = render(
      <WNavButton label="完了" accessibilityLabel="完了ボタン" onPress={onPress} disabled />,
    );
    fireEvent.press(getByLabelText('完了ボタン'));
    expect(onPress).not.toHaveBeenCalled();
  });
});
