/**
 * 交易状态对话框组件
 * 显示交易签名和确认状态
 */

import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  Modal,
  ActivityIndicator,
} from 'react-native';

interface TransactionStatusDialogProps {
  visible: boolean;
  status: string;
  title?: string;
  onClose?: () => void;
}

export const TransactionStatusDialog: React.FC<TransactionStatusDialogProps> = ({
  visible,
  status,
  title = '交易处理中',
  onClose,
}) => {
  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      onRequestClose={onClose}
    >
      <View style={styles.overlay}>
        <View style={styles.dialog}>
          <ActivityIndicator size="large" color="#B2955D" />
          <Text style={styles.title}>{title}</Text>
          <Text style={styles.status}>{status}</Text>
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
    padding: 32,
    alignItems: 'center',
    minWidth: 200,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000000',
    marginTop: 16,
    marginBottom: 8,
  },
  status: {
    fontSize: 14,
    color: '#666666',
    textAlign: 'center',
  },
});
