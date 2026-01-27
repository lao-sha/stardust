/**
 * 通用卡片组件
 */

import React from 'react';
import { View, StyleSheet, ViewStyle, StyleProp } from 'react-native';

const THEME_COLOR = '#B2955D';

interface CardProps {
  children: React.ReactNode;
  style?: StyleProp<ViewStyle>;
  padding?: number;
  elevation?: number;
}

export const Card: React.FC<CardProps> = ({
  children,
  style,
  padding = 16,
  elevation = 2,
}) => {
  return (
    <View
      style={[
        styles.card,
        { padding, elevation, shadowOpacity: elevation * 0.05 },
        style,
      ]}
    >
      {children}
    </View>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowRadius: 4,
  },
});
