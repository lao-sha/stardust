/**
 * 通用按钮组件
 */

import React from 'react';
import {
  Pressable,
  Text,
  StyleSheet,
  ViewStyle,
  TextStyle,
  ActivityIndicator,
  StyleProp,
} from 'react-native';

const THEME_COLOR = '#B2955D';

type ButtonVariant = 'primary' | 'secondary' | 'outline' | 'text';
type ButtonSize = 'small' | 'medium' | 'large';

interface ButtonProps {
  title: string;
  onPress: () => void;
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  loading?: boolean;
  style?: StyleProp<ViewStyle>;
  textStyle?: StyleProp<TextStyle>;
}

export const Button: React.FC<ButtonProps> = ({
  title,
  onPress,
  variant = 'primary',
  size = 'medium',
  disabled = false,
  loading = false,
  style,
  textStyle,
}) => {
  const isDisabled = disabled || loading;

  return (
    <Pressable
      style={({ pressed }) => [
        styles.button,
        styles[`button_${variant}`],
        styles[`button_${size}`],
        isDisabled && styles.button_disabled,
        pressed && !isDisabled && styles.button_pressed,
        style,
      ]}
      onPress={onPress}
      disabled={isDisabled}
    >
      {loading ? (
        <ActivityIndicator
          color={variant === 'primary' ? '#FFF' : THEME_COLOR}
        />
      ) : (
        <Text
          style={[
            styles.text,
            styles[`text_${variant}`],
            styles[`text_${size}`],
            isDisabled && styles.text_disabled,
            textStyle,
          ]}
        >
          {title}
        </Text>
      )}
    </Pressable>
  );
};

const styles = StyleSheet.create({
  button: {
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
  },
  button_primary: {
    backgroundColor: THEME_COLOR,
  },
  button_secondary: {
    backgroundColor: '#F5F5F7',
  },
  button_outline: {
    backgroundColor: 'transparent',
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  button_text: {
    backgroundColor: 'transparent',
  },
  button_small: {
    height: 32,
    paddingHorizontal: 12,
  },
  button_medium: {
    height: 48,
    paddingHorizontal: 24,
  },
  button_large: {
    height: 56,
    paddingHorizontal: 32,
  },
  button_disabled: {
    opacity: 0.5,
  },
  button_pressed: {
    opacity: 0.8,
  },
  text: {
    fontWeight: '600',
  },
  text_primary: {
    color: '#FFF',
  },
  text_secondary: {
    color: '#333',
  },
  text_outline: {
    color: THEME_COLOR,
  },
  text_text: {
    color: THEME_COLOR,
  },
  text_small: {
    fontSize: 14,
  },
  text_medium: {
    fontSize: 16,
  },
  text_large: {
    fontSize: 18,
  },
  text_disabled: {
    opacity: 0.5,
  },
});
