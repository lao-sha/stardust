/**
 * 补充押金页面
 * 路径: /maker/deposit/replenish
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore } from '@/stores/maker.store';
import { useWalletStore } from '@/stores/wallet.store';
import { MakerService } from '@/services/maker.service';
import { PageHeader } from '@/components/PageHeader';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { Card, Button, LoadingSpinner } from '@/components/common';
import { useAsync } from '@/hooks';

const DEPOSIT_TARGET_USD = 1050;
const DEPOSIT_THRESHOLD_USD = 950;

export default function ReplenishPage() {
  const router = useRouter();
  const {
    makerApp,
    depositUsdValue,
    dustPrice,
    replenishDeposit,
    txStatus,
    error,
    clearError,
    fetchMakerInfo,
    fetchDustPrice,
  } = useMakerStore();
  const { balance } = useWalletStore();
  const { execute, isLoading } = useAsync();

  const [showTxDialog, setShowTxDialog] = useState(false);

  useEffect(() => {
    fetchMakerInfo();
    fetchDustPrice();
  }, []);

  // 计算需要补充的金额
  const needsReplenish = depositUsdValue < DEPOSIT_THRESHOLD_USD;
  const replenishUsd = Math.max(0, DEPOSIT_TARGET_USD - depositUsdValue);
  const replenishDust = dustPrice > 0 ? replenishUsd / dustPrice : 0;
  const replenishDustBigInt = BigInt(Math.ceil(replenishDust * 1e12));

  // 检查余额
  const balanceBigInt = balance ? BigInt(balance) : BigInt(0);
  const isBalanceSufficient = balanceBigInt >= replenishDustBigInt;

  const getStatusConfig = () => {
    if (depositUsdValue >= DEPOSIT_TARGET_USD) {
      return { text: '正常', color: '#4CD964', icon: '✅' };
    }
    if (depositUsdValue >= DEPOSIT_THRESHOLD_USD) {
      return { text: '接近阈值', color: '#FF9500', icon: '⚠️' };
    }
    return { text: '低于阈值', color: '#FF3B30', icon: '❌' };
  };

  const statusConfig = getStatusConfig();

  const handleReplenish = async () => {
    setShowTxDialog(true);
    await execute(async () => {
      await replenishDeposit();
      setTimeout(() => {
        setShowTxDialog(false);
        router.back();
      }, 1500);
    });
  };

  const handleCloseTxDialog = () => {
    setShowTxDialog(false);
    clearError();
  };

  if (!makerApp) {
    return (
      <View style={styles.loadingContainer}>
        <LoadingSpinner text="加载中..." />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="补充押金" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 当前状态 */}
        <Card style={styles.section}>
          <Text style={styles.cardTitle}>当前状态</Text>
          <View style={styles.statusRow}>
            <View>
              <Text style={styles.depositLabel}>当前押金</Text>
              <Text style={styles.depositAmount}>
                {MakerService.formatDustAmount(makerApp.deposit)} DUST
              </Text>
              <Text style={styles.depositUsd}>
                USD价值: ${MakerService.formatUsdAmount(depositUsdValue)}
              </Text>
            </View>
            <View style={[styles.statusBadge, { backgroundColor: statusConfig.color + '20' }]}>
              <Text style={[styles.statusText, { color: statusConfig.color }]}>
                {statusConfig.icon} {statusConfig.text}
              </Text>
            </View>
          </View>
        </Card>

        {/* 补充计算 */}
        <Card style={styles.section}>
          <Text style={styles.cardTitle}>补充计算</Text>
          <View style={styles.calcRow}>
            <Text style={styles.calcLabel}>目标价值</Text>
            <Text style={styles.calcValue}>${DEPOSIT_TARGET_USD}</Text>
          </View>
          <View style={styles.calcRow}>
            <Text style={styles.calcLabel}>当前价值</Text>
            <Text style={styles.calcValue}>${MakerService.formatUsdAmount(depositUsdValue)}</Text>
          </View>
          <View style={[styles.calcRow, styles.calcRowHighlight]}>
            <Text style={styles.calcLabel}>需补充</Text>
            <Text style={styles.calcValueHighlight}>${MakerService.formatUsdAmount(replenishUsd)}</Text>
          </View>
          <View style={styles.dustAmount}>
            <Text style={styles.dustAmountText}>
              ≈ {replenishDust.toFixed(2)} DUST
            </Text>
            <Text style={styles.priceNote}>
              (按当前价格 {dustPrice.toFixed(4)} USD/DUST)
            </Text>
          </View>
        </Card>

        {/* 账户余额 */}
        <Card style={styles.section}>
          <Text style={styles.cardTitle}>您的可用余额</Text>
          <Text style={styles.balanceAmount}>
            {MakerService.formatDustAmount(balanceBigInt)} DUST
          </Text>
          <View style={[styles.balanceStatus, isBalanceSufficient ? styles.statusOk : styles.statusError]}>
            <Text style={[styles.balanceStatusText, isBalanceSufficient ? styles.statusTextOk : styles.statusTextError]}>
              {isBalanceSufficient ? '✅ 余额充足' : '❌ 余额不足'}
            </Text>
          </View>
        </Card>

        {/* 补充按钮 */}
        <Button
          title={replenishUsd <= 0 ? '押金已充足' : '确认补充'}
          onPress={handleReplenish}
          loading={isLoading}
          disabled={!isBalanceSufficient || isLoading || replenishUsd <= 0}
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
    marginBottom: 12,
  },
  statusRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
  },
  depositLabel: {
    fontSize: 12,
    color: '#8E8E93',
    marginBottom: 4,
  },
  depositAmount: {
    fontSize: 20,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  depositUsd: {
    fontSize: 14,
    color: '#8E8E93',
  },
  statusBadge: {
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 6,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  calcRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  calcRowHighlight: {
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#F2F2F7',
  },
  calcLabel: {
    fontSize: 14,
    color: '#8E8E93',
  },
  calcValue: {
    fontSize: 14,
    color: '#1C1C1E',
  },
  calcValueHighlight: {
    fontSize: 18,
    fontWeight: '600',
    color: '#B2955D',
  },
  dustAmount: {
    alignItems: 'center',
    marginTop: 8,
  },
  dustAmountText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#B2955D',
  },
  priceNote: {
    fontSize: 12,
    color: '#8E8E93',
    marginTop: 4,
  },
  balanceAmount: {
    fontSize: 24,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 12,
  },
  balanceStatus: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    alignSelf: 'flex-start',
  },
  statusOk: {
    backgroundColor: '#4CD96420',
  },
  statusError: {
    backgroundColor: '#FF3B3020',
  },
  balanceStatusText: {
    fontSize: 14,
    fontWeight: '500',
  },
  statusTextOk: {
    color: '#4CD964',
  },
  statusTextError: {
    color: '#FF3B30',
  },
});
