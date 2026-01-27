/**
 * 申请提现页面
 * 路径: /maker/deposit/withdraw
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore, selectHasPendingWithdrawal } from '@/stores/maker.store';
import { MakerService } from '@/services/maker.service';
import { PageHeader } from '@/components/PageHeader';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { Card, Button, LoadingSpinner } from '@/components/common';
import { useAsync } from '@/hooks';

const DEPOSIT_TARGET_USD = 1000;

export default function WithdrawPage() {
  const router = useRouter();
  const {
    makerApp,
    depositUsdValue,
    dustPrice,
    requestWithdrawal,
    txStatus,
    error,
    clearError,
    fetchMakerInfo,
    fetchDustPrice,
  } = useMakerStore();

  const hasPendingWithdrawal = useMakerStore(selectHasPendingWithdrawal);
  const { execute, isLoading } = useAsync();

  const [showTxDialog, setShowTxDialog] = useState(false);
  const [withdrawAmount, setWithdrawAmount] = useState('');

  useEffect(() => {
    fetchMakerInfo();
    fetchDustPrice();
  }, []);

  // 如果已有待处理的提现，跳转到状态页
  useEffect(() => {
    if (hasPendingWithdrawal) {
      router.replace('/maker/deposit/withdraw/status');
    }
  }, [hasPendingWithdrawal]);

  if (!makerApp) {
    return (
      <View style={styles.loadingContainer}>
        <LoadingSpinner text="加载中..." />
      </View>
    );
  }

  const maxWithdrawDust = Number(makerApp.deposit) / 1e12;
  const withdrawDust = parseFloat(withdrawAmount) || 0;
  const withdrawDustBigInt = BigInt(Math.floor(withdrawDust * 1e12));
  const withdrawUsd = withdrawDust * dustPrice;

  // 计算提现后的押金
  const remainingDust = maxWithdrawDust - withdrawDust;
  const remainingUsd = remainingDust * dustPrice;
  const isBelowTarget = remainingUsd < DEPOSIT_TARGET_USD;

  const isValidAmount = withdrawDust > 0 && withdrawDust <= maxWithdrawDust;

  const handleQuickAmount = (percent: number) => {
    const amount = (maxWithdrawDust * percent / 100).toFixed(4);
    setWithdrawAmount(amount);
  };

  const handleSubmit = async () => {
    if (!isValidAmount) {
      Alert.alert('金额无效', '请输入有效的提现金额');
      return;
    }

    if (isBelowTarget) {
      Alert.alert(
        '提现警告',
        `提现后押金将低于目标值 $${DEPOSIT_TARGET_USD}，可能影响您的服务。确定继续？`,
        [
          { text: '取消', style: 'cancel' },
          {
            text: '确定提现',
            onPress: async () => {
              await submitWithdrawal();
            },
          },
        ]
      );
    } else {
      await submitWithdrawal();
    }
  };

  const submitWithdrawal = async () => {
    setShowTxDialog(true);
    await execute(async () => {
      await requestWithdrawal(withdrawDustBigInt);
      setTimeout(() => {
        setShowTxDialog(false);
        router.replace('/maker/deposit/withdraw/status');
      }, 1500);
    });
  };

  const handleCloseTxDialog = () => {
    setShowTxDialog(false);
    clearError();
  };

  return (
    <View style={styles.container}>
      <PageHeader title="申请提现" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 可提现押金 */}
        <Card style={styles.section}>
          <Text style={styles.cardTitle}>可提现押金</Text>
          <Text style={styles.availableAmount}>
            {MakerService.formatDustAmount(makerApp.deposit)} DUST
          </Text>
          <Text style={styles.availableUsd}>
            ≈ ${MakerService.formatUsdAmount(depositUsdValue)} USD
          </Text>
        </Card>

        {/* 提现金额 */}
        <Card style={styles.section}>
          <Text style={styles.cardTitle}>提现金额</Text>
          <View style={styles.inputContainer}>
            <TextInput
              style={styles.input}
              placeholder="0.0000"
              value={withdrawAmount}
              onChangeText={setWithdrawAmount}
              keyboardType="decimal-pad"
            />
            <Text style={styles.inputSuffix}>DUST</Text>
          </View>
          {withdrawDust > 0 && (
            <Text style={styles.inputUsd}>
              ≈ ${MakerService.formatUsdAmount(withdrawUsd)} USD
            </Text>
          )}
        </Card>

        {/* 快捷金额 */}
        <View style={styles.quickAmounts}>
          <TouchableOpacity style={styles.quickButton} onPress={() => handleQuickAmount(25)}>
            <Text style={styles.quickButtonText}>25%</Text>
          </TouchableOpacity>
          <TouchableOpacity style={styles.quickButton} onPress={() => handleQuickAmount(50)}>
            <Text style={styles.quickButtonText}>50%</Text>
          </TouchableOpacity>
          <TouchableOpacity style={styles.quickButton} onPress={() => handleQuickAmount(75)}>
            <Text style={styles.quickButtonText}>75%</Text>
          </TouchableOpacity>
          <TouchableOpacity style={styles.quickButton} onPress={() => handleQuickAmount(100)}>
            <Text style={styles.quickButtonText}>全部</Text>
          </TouchableOpacity>
        </View>

        {/* 提现说明 */}
        <Card style={[styles.section, styles.infoCard]}>
          <Text style={styles.infoIcon}>⚠️</Text>
          <Text style={styles.infoTitle}>提现说明</Text>
          <View style={styles.infoList}>
            <Text style={styles.infoItem}>• 提现需要 7 天冷却期</Text>
            <Text style={styles.infoItem}>• 冷却期内可取消提现</Text>
            <Text style={styles.infoItem}>• 提现后押金可能低于阈值</Text>
            <Text style={styles.infoItem}>• 押金不足将暂停服务</Text>
          </View>
        </Card>

        {/* 提现后状态 */}
        {withdrawDust > 0 && (
          <Card style={styles.section}>
            <Text style={styles.resultTitle}>提现后押金</Text>
            <Text style={styles.resultAmount}>
              {remainingDust.toFixed(4)} DUST (${MakerService.formatUsdAmount(remainingUsd)})
            </Text>
            <View style={[styles.resultStatus, isBelowTarget ? styles.statusWarning : styles.statusOk]}>
              <Text style={[styles.resultStatusText, isBelowTarget ? styles.statusTextWarning : styles.statusTextOk]}>
                {isBelowTarget ? '⚠️ 低于目标值' : '✅ 正常'}
              </Text>
            </View>
          </Card>
        )}

        {/* 提交按钮 */}
        <Button
          title="提交申请"
          onPress={handleSubmit}
          loading={isLoading}
          disabled={!isValidAmount || isLoading}
        />
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
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  section: {
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 8,
  },
  availableAmount: {
    fontSize: 24,
    fontWeight: '600',
    color: '#1C1C1E',
  },
  availableUsd: {
    fontSize: 14,
    color: '#8E8E93',
    marginTop: 4,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 12,
  },
  input: {
    flex: 1,
    fontSize: 24,
    fontWeight: '600',
    color: '#1C1C1E',
    paddingVertical: 12,
  },
  inputSuffix: {
    fontSize: 16,
    color: '#8E8E93',
    marginLeft: 8,
  },
  inputUsd: {
    fontSize: 14,
    color: '#8E8E93',
    marginTop: 8,
    textAlign: 'right',
  },
  quickAmounts: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 16,
  },
  quickButton: {
    flex: 1,
    backgroundColor: '#FFFFFF',
    borderRadius: 8,
    paddingVertical: 10,
    alignItems: 'center',
  },
  quickButtonText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#B2955D',
  },
  infoCard: {
    backgroundColor: '#FFF9E6',
  },
  infoIcon: {
    fontSize: 20,
    marginBottom: 8,
  },
  infoTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 12,
  },
  infoList: {
    gap: 6,
  },
  infoItem: {
    fontSize: 14,
    color: '#666666',
    lineHeight: 20,
  },
  resultTitle: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 8,
  },
  resultAmount: {
    fontSize: 16,
    fontWeight: '500',
    color: '#1C1C1E',
    marginBottom: 8,
  },
  resultStatus: {
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 6,
    alignSelf: 'flex-start',
  },
  statusOk: {
    backgroundColor: '#4CD96420',
  },
  statusWarning: {
    backgroundColor: '#FF950020',
  },
  resultStatusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  statusTextOk: {
    color: '#4CD964',
  },
  statusTextWarning: {
    color: '#FF9500',
  },
});
