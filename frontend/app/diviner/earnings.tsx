/**
 * 收益管理页面
 */

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  TextInput,
  Alert,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { WithdrawalRecord } from '@/features/diviner';
import { divinationMarketService } from '@/services/divination-market.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

export default function EarningsPage() {
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [availableBalance, setAvailableBalance] = useState(BigInt(0));
  const [totalEarnings, setTotalEarnings] = useState(BigInt(0));
  const [withdrawals, setWithdrawals] = useState<WithdrawalRecord[]>([]);
  const [withdrawAmount, setWithdrawAmount] = useState('');
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const loadData = useCallback(async () => {
    if (!address) return;

    try {
      const providerData = await divinationMarketService.getProviderByAccount(address);
      if (providerData) {
        // 从链上获取收益数据
        const earnings = await divinationMarketService.getProviderEarnings(providerData.id);
        setAvailableBalance(earnings.availableBalance);
        setTotalEarnings(earnings.totalEarnings);
        
        // 获取提现记录
        const withdrawalHistory = await divinationMarketService.getWithdrawalHistory(providerData.id);
        setWithdrawals(withdrawalHistory);
      }
    } catch (error) {
      console.error('Load earnings data error:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [address]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const onRefresh = () => {
    setRefreshing(true);
    loadData();
  };

  const formatDust = (amount: bigint) => (Number(amount) / 1e10).toFixed(2);

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return `${date.getFullYear()}/${date.getMonth() + 1}/${date.getDate()}`;
  };

  const handleWithdraw = () => {
    const amount = parseFloat(withdrawAmount);
    if (isNaN(amount) || amount <= 0) {
      Alert.alert('提示', '请输入有效的提现金额');
      return;
    }

    const amountBigInt = BigInt(Math.floor(amount * 1e12)); // DUST 有 12 位小数
    if (amountBigInt > availableBalance) {
      Alert.alert('提示', '提现金额超过可用余额');
      return;
    }

    if (!isSignerUnlocked()) {
      setShowUnlockDialog(true);
      return;
    }

    executeWithdraw(amountBigInt);
  };

  const executeWithdraw = async (amountBigInt: bigint) => {
    setShowTxStatus(true);
    setTxStatus('正在提交提现申请...');

    try {
      await divinationMarketService.requestWithdrawal(amountBigInt, (status) => setTxStatus(status));

      setTxStatus('提现申请已提交！');
      setWithdrawAmount('');
      setTimeout(() => {
        setShowTxStatus(false);
        loadData();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('提现失败', error.message || '请稍后重试');
    }
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      const amount = parseFloat(withdrawAmount);
      const amountBigInt = BigInt(Math.floor(amount * 1e12));
      await executeWithdraw(amountBigInt);
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const handleWithdrawAll = () => {
    setWithdrawAmount(formatDust(availableBalance));
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="收益管理" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="收益管理" />

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={THEME_COLOR} />}
      >
        {/* 余额卡片 */}
        <View style={styles.balanceCard}>
          <Text style={styles.balanceLabel}>可提现余额</Text>
          <Text style={styles.balanceValue}>{formatDust(availableBalance)} DUST</Text>
          <View style={styles.totalRow}>
            <Text style={styles.totalLabel}>累计收益：</Text>
            <Text style={styles.totalValue}>{formatDust(totalEarnings)} DUST</Text>
          </View>
        </View>

        {/* 提现表单 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>申请提现</Text>
          <View style={styles.withdrawCard}>
            <View style={styles.inputRow}>
              <TextInput
                style={styles.amountInput}
                value={withdrawAmount}
                onChangeText={setWithdrawAmount}
                placeholder="输入提现金额"
                placeholderTextColor="#999"
                keyboardType="decimal-pad"
              />
              <Pressable style={styles.allBtn} onPress={handleWithdrawAll}>
                <Text style={styles.allBtnText}>全部</Text>
              </Pressable>
            </View>
            <Pressable
              style={[styles.withdrawBtn, submitting && styles.withdrawBtnDisabled]}
              onPress={handleWithdraw}
              disabled={submitting}
            >
              {submitting ? (
                <ActivityIndicator color="#FFF" />
              ) : (
                <Text style={styles.withdrawBtnText}>立即提现</Text>
              )}
            </Pressable>
            <Text style={styles.withdrawNote}>提现将即时到账您的钱包</Text>
          </View>
        </View>

        {/* 提现记录 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>提现记录</Text>
          {withdrawals.length === 0 ? (
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyText}>暂无提现记录</Text>
            </View>
          ) : (
            withdrawals.map(record => (
              <View key={record.id} style={styles.recordCard}>
                <View style={styles.recordLeft}>
                  <Text style={styles.recordAmount}>-{formatDust(record.amount)} DUST</Text>
                  <Text style={styles.recordTime}>{formatTime(record.createdAt)}</Text>
                </View>
                <View style={styles.recordStatus}>
                  <Text style={styles.recordStatusText}>已完成</Text>
                </View>
              </View>
            ))
          )}
        </View>
      </ScrollView>

      <UnlockWalletDialog
        visible={showUnlockDialog}
        onClose={() => setShowUnlockDialog(false)}
        onUnlock={handleWalletUnlocked}
      />

      <TransactionStatusDialog
        visible={showTxStatus}
        status={txStatus}
        onClose={() => setShowTxStatus(false)}
      />

      <BottomNavBar activeTab="profile" />
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  container: {
    flex: 1,
  },
  contentContainer: {
    paddingBottom: 100,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  balanceCard: {
    backgroundColor: THEME_COLOR,
    margin: 16,
    borderRadius: 12,
    padding: 24,
    alignItems: 'center',
  },
  balanceLabel: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
    marginBottom: 8,
  },
  balanceValue: {
    fontSize: 36,
    fontWeight: '700',
    color: '#FFF',
    marginBottom: 16,
  },
  totalRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  totalLabel: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
  },
  totalValue: {
    fontSize: 14,
    color: '#FFF',
    fontWeight: '500',
  },
  section: {
    paddingHorizontal: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000',
    marginBottom: 12,
  },
  withdrawCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  inputRow: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 12,
  },
  amountInput: {
    flex: 1,
    height: 48,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 16,
    color: '#333',
    backgroundColor: '#FAFAFA',
  },
  allBtn: {
    height: 48,
    paddingHorizontal: 16,
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 1,
    borderColor: THEME_COLOR,
    borderRadius: 8,
  },
  allBtnText: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  withdrawBtn: {
    height: 48,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 8,
  },
  withdrawBtnDisabled: {
    opacity: 0.5,
  },
  withdrawBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
  withdrawNote: {
    fontSize: 12,
    color: '#999',
    textAlign: 'center',
  },
  emptyContainer: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 32,
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 14,
    color: '#999',
  },
  recordCard: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 8,
  },
  recordLeft: {},
  recordAmount: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  recordTime: {
    fontSize: 12,
    color: '#999',
  },
  recordStatus: {
    backgroundColor: '#E8F8EB',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  recordStatusText: {
    fontSize: 12,
    color: '#4CD964',
  },
});
