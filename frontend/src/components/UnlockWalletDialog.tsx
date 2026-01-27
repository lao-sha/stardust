/**
 * 解锁钱包对话框组件
 * 用于移动端交易签名前解锁钱包
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  TouchableOpacity,
  Modal,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { unlockWalletForSigning } from '@/lib/signer';

interface UnlockWalletDialogProps {
  visible: boolean;
  onUnlock: ((password: string) => void) | (() => void);
  onCancel?: () => void;
  onClose?: () => void;
}

export const UnlockWalletDialog: React.FC<UnlockWalletDialogProps> = ({
  visible,
  onUnlock,
  onCancel,
  onClose,
}) => {
  const handleClose = onClose || onCancel || (() => {});
  const [password, setPassword] = useState('');
  const [isUnlocking, setIsUnlocking] = useState(false);

  const handleUnlock = async () => {
    if (!password) {
      Alert.alert('提示', '请输入密码');
      return;
    }

    setIsUnlocking(true);

    try {
      await unlockWalletForSigning(password);
      const pwd = password;
      setPassword('');
      // 支持带 password 参数和不带参数两种调用方式
      if (onUnlock.length > 0) {
        (onUnlock as (password: string) => void)(pwd);
      } else {
        (onUnlock as () => void)();
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '密码错误';
      Alert.alert('错误', errorMessage);
    } finally {
      setIsUnlocking(false);
    }
  };

  const handleCancel = () => {
    setPassword('');
    handleClose();
  };

  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      onRequestClose={handleCancel}
    >
      <View style={styles.overlay}>
        <View style={styles.dialog}>
          {/* 标题 */}
          <View style={styles.header}>
            <Text style={styles.title}>解锁钱包</Text>
            <Text style={styles.subtitle}>
              请输入密码以签名交易
            </Text>
          </View>

          {/* 密码输入 */}
          <View style={styles.content}>
            <TextInput
              style={styles.input}
              value={password}
              onChangeText={setPassword}
              placeholder="请输入钱包密码"
              placeholderTextColor="#999999"
              secureTextEntry
              autoFocus
              editable={!isUnlocking}
            />
          </View>

          {/* 按钮 */}
          <View style={styles.footer}>
            <TouchableOpacity
              style={[styles.button, styles.cancelButton]}
              onPress={handleCancel}
              disabled={isUnlocking}
            >
              <Text style={styles.cancelButtonText}>取消</Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={[styles.button, styles.unlockButton]}
              onPress={handleUnlock}
              disabled={isUnlocking}
            >
              {isUnlocking ? (
                <ActivityIndicator color="#FFFFFF" />
              ) : (
                <Text style={styles.unlockButtonText}>解锁</Text>
              )}
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </Modal>
  );
};

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  dialog: {
    backgroundColor: '#FFFFFF',
    borderRadius: 16,
    width: '100%',
    maxWidth: 400,
    overflow: 'hidden',
  },
  header: {
    padding: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#666666',
  },
  content: {
    padding: 20,
  },
  input: {
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 12,
    fontSize: 16,
    color: '#000000',
    borderWidth: 1,
    borderColor: '#E5E5EA',
  },
  footer: {
    flexDirection: 'row',
    padding: 20,
    gap: 12,
  },
  button: {
    flex: 1,
    borderRadius: 8,
    paddingVertical: 12,
    alignItems: 'center',
    justifyContent: 'center',
  },
  cancelButton: {
    backgroundColor: '#F5F5F7',
  },
  cancelButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#666666',
  },
  unlockButton: {
    backgroundColor: '#B2955D',
  },
  unlockButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
});
