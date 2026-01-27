/**
 * 提现进度页面
 * 路径: /maker/deposit/withdraw/status
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore, selectCanExecuteWithdrawal } from '@/stores/maker.store';
import { WithdrawalProgress } from '@/features/maker/components';
import { PageHeader } from '@/components/PageHeader';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { Button } from '@/components/common';
import { useAsync } from '@/hooks';

export default function WithdrawStatusPage() {
  const router = useRouter();
  const {
    withdrawalRequest,
    executeWithdrawal,
    cancelWithdrawal,
    txStatus,
    error,
    clearError,
    fetchWithdrawalRequest,
  } = useMakerStore();

  const canExecute = useMakerStore(selectCanExecuteWithdrawal);
  const { execute, isLoading } = useAsync();

  const [showTxDialog, setShowTxDialog] = useState(false);

  useEffect(() => {
    fetchWithdrawalRequest();
  }, []);

  const handleExecute = async () => {
    Alert.alert(
      '执行提现',
      '确定要执行提现吗？押金将转入您的账户。',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定执行',
          onPress: async () => {
            setShowTxDialog(true);
            await execute(async () => {
              await executeWithdrawal();
              setTimeout(() => {
                setShowTxDialog(false);
                router.replace('/maker/deposit');
              }, 1500);
            });
          },
        },
      ]
    );
  };

  const handleCancel = async () => {
    Alert.alert(
      '取消提现',
      '确定要取消提现申请吗？',
      [
        { text: '再想想', style: 'cancel' },
        {
          text: '确定取消',
          style: 'destructive',
          onPress: async () => {
            setShowTxDialog(true);
            await execute(async () => {
              await cancelWithdrawal();
              setTimeout(() => {
                setShowTxDialog(false);
                router.replace('/maker/deposit');
              }, 1500);
            });
          },
        },
      ]
    );
  };

  const handleCloseTxDialog = () => {
    setShowTxDialog(false);
    clearError();
  };

  if (!withdrawalRequest) {
    return (
      <View style={styles.container}>
        <PageHeader title="提现进度" showBack />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>暂无提现申请</Text>
          <TouchableOpacity
            style={styles.applyButton}
            onPress={() => router.replace('/maker/deposit/withdraw')}
          >
            <Text style={styles.applyButtonText}>申请提现</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="提现进度" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 提现进度组件 */}
        <WithdrawalProgress request={withdrawalRequest} />

        {/* 操作按钮 */}
        <View style={styles.actions}>
          {canExecute && (
            <Button
              title="执行提现"
              onPress={handleExecute}
              loading={isLoading}
              disabled={isLoading}
              variant="primary"
            />
          )}

          <Button
            title="取消提现"
            onPress={handleCancel}
            loading={isLoading}
            disabled={isLoading}
            variant="outline"
          />
        </View>
      </ScrollView>

      {/* 交易状态弹窗 */}
      <TransactionStatusDialog
        visible={showTxDialog}
        status={txStatus || ''}
        error={error}
        onClose={handleCloseTxDialog}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 24,
  },
  emptyText: {
    fontSize: 16,
    color: '#8E8E93',
    marginBottom: 16,
  },
  applyButton: {
    backgroundColor: '#B2955D',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  applyButtonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  actions: {
    marginTop: 24,
    gap: 12,
  },
});
