/**
 * 订单详情/支付页面
 * 显示订单信息、收款地址、倒计时、标记已付款
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
  Clipboard,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useTradingStore } from '@/stores/trading.store';
import { TradingService } from '@/services/trading.service';
import { CountdownTimer } from '@/features/trading/components';
import { OrderState } from '@/stores/trading.store';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import { Card, Button, LoadingSpinner } from '@/components/common';
import { useAsync } from '@/hooks';

export default function OrderDetailPage() {
  const router = useRouter();
  const { orderId } = useLocalSearchParams<{ orderId: string }>();
  const {
    currentOrder,
    loadingOrder,
    fetchOrder,
    markPaid,
    cancelOrder,
    subscribeToOrder,
  } = useTradingStore();

  const { execute, isLoading } = useAsync();
  const [unsubscribe, setUnsubscribe] = useState<(() => void) | null>(null);

  useEffect(() => {
    if (orderId) {
      fetchOrder(parseInt(orderId));
      const unsub = subscribeToOrder(parseInt(orderId));
      setUnsubscribe(() => unsub);
    }

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [orderId]);

  useEffect(() => {
    if (currentOrder) {
      // 根据订单状态跳转
      if (currentOrder.state === OrderState.Paid) {
        router.replace(`/wallet/buy-dust/${orderId}/waiting` as any);
      } else if (currentOrder.state === OrderState.Released) {
        router.replace(`/wallet/buy-dust/${orderId}/complete` as any);
      }
    }
  }, [currentOrder]);

  const handleCopyAddress = () => {
    if (currentOrder) {
      Clipboard.setString(currentOrder.makerTronAddress);
      Alert.alert('成功', '收款地址已复制');
    }
  };

  const handleMarkPaid = async () => {
    if (!currentOrder) return;

    Alert.alert(
      '确认付款',
      '请确认您已完成 USDT 转账',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确认',
          onPress: async () => {
            try {
              await execute(async () => {
                await markPaid(currentOrder.id);
              });
              Alert.alert('成功', '已标记为已付款，等待做市商确认');
            } catch (error) {
              Alert.alert('错误', '操作失败，请重试');
            }
          },
        },
      ]
    );
  };

  const handleCancelOrder = async () => {
    if (!currentOrder) return;

    Alert.alert(
      '取消订单',
      '确定要取消此订单吗？',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定',
          style: 'destructive',
          onPress: async () => {
            try {
              await execute(async () => {
                await cancelOrder(currentOrder.id);
              });
              Alert.alert('成功', '订单已取消', [
                { text: '确定', onPress: () => router.back() },
              ]);
            } catch (error) {
              Alert.alert('错误', '取消失败，请重试');
            }
          },
        },
      ]
    );
  };

  if (loadingOrder || !currentOrder) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="订单详情" />
        <LoadingSpinner text="加载订单信息..." />
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      {/* 页面头部 */}
      <PageHeader title="订单详情" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
      {/* 倒计时 */}
      <View style={styles.section}>
        <Card style={styles.timerCard}>
          <Text style={styles.timerLabel}>请在以下时间内完成付款</Text>
          <CountdownTimer
            expireAt={currentOrder.expireAt}
            onExpire={() => Alert.alert('提示', '订单已超时')}
            style={styles.timer}
          />
        </Card>
      </View>

      {/* 订单信息 */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>订单信息</Text>
        <Card style={styles.infoCard}>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>订单号</Text>
            <Text style={styles.infoValue}>#{currentOrder.id}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>状态</Text>
            <Text style={[styles.infoValue, styles.statusText]}>
              {currentOrder.state === OrderState.Created ? '待付款' : currentOrder.state}
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>金额</Text>
            <Text style={styles.infoValue}>
              {TradingService.formatUsdAmount(currentOrder.amount)} USDT
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>数量</Text>
            <Text style={styles.infoValue}>
              {TradingService.formatDustAmount(currentOrder.qty)} DUST
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>单价</Text>
            <Text style={styles.infoValue}>
              {(Number(currentOrder.price) / 1_000_000).toFixed(6)} USDT
            </Text>
          </View>
        </Card>
      </View>

      {/* 收款信息 */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>收款信息</Text>
        <Card style={styles.paymentCard}>
          <Text style={styles.paymentLabel}>收款地址 (TRC20)</Text>
          <View style={styles.addressContainer}>
            <Text style={styles.address} numberOfLines={1}>
              {currentOrder.makerTronAddress}
            </Text>
            <TouchableOpacity
              style={styles.copyButton}
              onPress={handleCopyAddress}
            >
              <Text style={styles.copyButtonText}>复制</Text>
            </TouchableOpacity>
          </View>
        </Card>
      </View>

      {/* 付款提示 */}
      <View style={styles.section}>
        <Card style={styles.warningCard}>
          <Text style={styles.warningTitle}>⚠️ 付款提示</Text>
          <Text style={styles.warningText}>• 请使用 TRC20 网络转账</Text>
          <Text style={styles.warningText}>• 金额必须精确匹配</Text>
          <Text style={styles.warningText}>• 超时未付款订单将自动取消</Text>
        </Card>
      </View>

      {/* 操作按钮 */}
      <View style={styles.section}>
        <Button
          title="我已付款"
          onPress={handleMarkPaid}
          loading={isLoading}
          style={styles.paidButton}
        />

        <Button
          title="取消订单"
          onPress={handleCancelOrder}
          variant="outline"
          loading={isLoading}
          style={styles.cancelButton}
        />
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
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  timerCard: {
    backgroundColor: '#FFF9F0',
    borderWidth: 2,
    borderColor: '#FF9500',
    alignItems: 'center',
  },
  timerLabel: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 8,
  },
  timer: {
    fontSize: 24,
  },
  infoCard: {
    padding: 16,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  infoLabel: {
    fontSize: 14,
    color: '#666666',
  },
  infoValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#000000',
  },
  statusText: {
    color: '#FF9500',
  },
  paymentCard: {
    padding: 16,
  },
  paymentLabel: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 8,
  },
  addressContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    padding: 12,
  },
  address: {
    flex: 1,
    fontSize: 14,
    fontFamily: 'monospace',
    color: '#000000',
  },
  copyButton: {
    backgroundColor: '#B2955D',
    borderRadius: 6,
    paddingHorizontal: 12,
    paddingVertical: 6,
    marginLeft: 8,
  },
  copyButtonText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  warningCard: {
    backgroundColor: '#FFF3F3',
    borderWidth: 1,
    borderColor: '#FFE0E0',
  },
  warningTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FF3B30',
    marginBottom: 12,
  },
  warningText: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 6,
  },
  paidButton: {
    marginBottom: 12,
  },
  cancelButton: {},
});
