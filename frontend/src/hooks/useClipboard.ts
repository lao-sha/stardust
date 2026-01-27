/**
 * 剪贴板操作 Hook
 */

import { useState, useCallback } from 'react';
import * as Clipboard from 'expo-clipboard';
import { Alert } from 'react-native';

export function useClipboard() {
  const [copiedText, setCopiedText] = useState<string>('');

  const copyToClipboard = useCallback(async (text: string, showAlert = true) => {
    try {
      await Clipboard.setStringAsync(text);
      setCopiedText(text);
      if (showAlert) {
        Alert.alert('已复制', text);
      }
      return true;
    } catch (error) {
      console.error('Copy to clipboard error:', error);
      if (showAlert) {
        Alert.alert('复制失败', '请稍后重试');
      }
      return false;
    }
  }, []);

  const getFromClipboard = useCallback(async () => {
    try {
      const text = await Clipboard.getStringAsync();
      return text;
    } catch (error) {
      console.error('Get from clipboard error:', error);
      return '';
    }
  }, []);

  return {
    copiedText,
    copyToClipboard,
    getFromClipboard,
  };
}
