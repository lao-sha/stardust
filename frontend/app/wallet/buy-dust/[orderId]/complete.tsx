/**
 * 交易完成页面
 * 显示交易成功，获得的 DUST，返回钱包或继续购买
 */

import React, { useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useTradingStore } from '@/stores/trading.store';
import { TradingService } from '@/services/trading.service';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import { Card, Button, LoadingSpinner } from '@/components/common';

export default function CompletePage() {
  const router = useRouter();
  const { orderId } = useLocalSearchParams<{ orderId: string }>();
  const { currentOrder, fetchOrder } = useTradingStore();

  useEffect(() => {
    if (orderId) {
      fetchOrder(parseInt(orderId));
    }
  }, [orderId]);

  const handleViewWallet = () => {
    router.push('/profile');
  };

  const handleContinueBuying = () => {
    router.push('/wallet/buy-dust');
  };

  if (!currentOrder) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="交易完成" showBack={false} />
        <LoadingSpinner text="加载订单信息..." />
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      {/* 页面头部 */}
      <PageHeader title="交易完成" showBack={false} />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 成功状态 */}
        <View style={styles.section}>
          <Card style={styles.successCard}>
            <Text style={styles.successIcon}>✅</Text>
            <Text style={styles.successTitle}>交易成功</Text>
          </Card>
        </View>

        {/* 交易详情 */}
        <View style={styles.section}>
          <Card style={styles.detailCard}>
            <Text style={styles.detailLabel}>您已成功购买</Text>
            <Text style={styles.dustAmount}>
              {TradingService.formatDustAmount(currentOrder.qty)} DUST
            </Text>
            <View style={styles.detailRow}>
              <Text style={styles.detailText}>
                支付: {TradingService.formatUsdAmount(currentOrder.amount)} USDT
              </Text>
            </View>
            <View style={styles.detailRow}>
              <Text style={styles.detailText}>
                订单号: #{currentOrder.id}
              </Text>
            </View>
          </Card>
        </View>

        {/* 操作按钮 */}
        <View style={styles.section}>
          <Button
            title="查看钱包"
            onPress={handleViewWallet}
            style={styles.walletButton}
          />

          <Button
            title="继续购买"
            onPress={handleContinueBuying}
            variant="outline"
            style={styles.continueButton}
          />
        </View>

        {/* 提示信息 */}
        <View style={styles.section}>
          <Card style={styles.tipCard}>
            <Text style={styles.tipTitle}>💡 温馨提示</Text>
            <Text style={styles.tipText}>
              DUST 已到账，您可以在钱包中查看余额
            </Text>
            <Text style={styles.tipText}>
              如有问题，请联系客服或查看订单历史
            </Text>
          </Card>
        </View>
      </ScrollView>

      {/* 底部导航栏 */}
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
    paddingBottom: 20,
  },
  section: {
    padding: 16,
  },
  successCard: {
    alignItems: 'center',
  },
  successIcon: {
    fontSize: 80,
    marginBottom: 16,
  },
  successTitle: {
    fontSize: 24,
    fontWeight: '600',
    color: '#4CD964',
  },
  detailCard: {
    alignItems: 'center',
  },
  detailLabel: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 12,
  },
  dustAmount: {
    fontSize: 36,
    fontWeight: '700',
    color: '#B2955D',
    marginBottom: 16,
  },
  detailRow: {
    marginBottom: 8,
  },
  detailText: {
    fontSize: 14,
    color: '#666666',
  },
  walletButton: {
    marginBottom: 12,
  },
  continueButton: {},
  tipCard: {
    backgroundColor: '#FFF9F0',
  },
  tipTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  tipText: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 6,
  },
});
