/**
 * 等待放币页面
 * 显示等待状态，联系做市商，申请仲裁
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useTradingStore, OrderState } from '@/stores/trading.store';
import { TradingService, tradingService } from '@/services/trading.service';
import {
  ContactMakerDialog,
  DisputeDialog,
  ReleaseTimeoutAlert,
} from '@/features/trading/components';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import { Card, Button, LoadingSpinner } from '@/components/common';
import { useAsync } from '@/hooks';
import type { Maker } from '@/stores/trading.store';

export default function WaitingPage() {
  const router = useRouter();
  const { orderId } = useLocalSearchParams<{ orderId: string }>();
  const {
    currentOrder,
    fetchOrder,
    subscribeToOrder,
    dispute,
  } = useTradingStore();

  const { execute, isLoading } = useAsync();
  const [maker, setMaker] = useState<Maker | null>(null);
  const [showContactDialog, setShowContactDialog] = useState(false);
  const [showDisputeDialog, setShowDisputeDialog] = useState(false);
  const [paidAt, setPaidAt] = useState<number | null>(null);

  useEffect(() => {
    if (orderId) {
      fetchOrder(parseInt(orderId));
      const unsub = subscribeToOrder(parseInt(orderId));
      return () => {
        if (unsub) unsub();
      };
    }
    return undefined;
  }, [orderId]);

  // 获取做市商信息
  useEffect(() => {
    if (currentOrder) {
      tradingService.getMaker(currentOrder.makerId).then(setMaker);
      // 记录付款时间（用于超时计算）
      if (!paidAt) {
        setPaidAt(Date.now());
      }
    }
  }, [currentOrder]);

  // 订单状态变化时跳转
  useEffect(() => {
    if (currentOrder && currentOrder.state === OrderState.Released) {
      router.replace(`/wallet/buy-dust/${orderId}/complete` as any);
    }
  }, [currentOrder]);

  const handleContactMaker = () => {
    setShowContactDialog(true);
  };

  const handleDispute = () => {
    setShowDisputeDialog(true);
  };

  const handleDisputeSubmit = async (reason: string, evidenceUri?: string) => {
    if (!currentOrder) return;

    // TODO: 如果有证据图片，先上传到 IPFS 获取 CID
    const evidenceCid = evidenceUri ? 'placeholder-cid' : undefined;

    await dispute(currentOrder.id, reason, evidenceCid);
  };

  if (!currentOrder) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="等待放币" />
        <LoadingSpinner text="加载订单信息..." />
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      {/* 页面头部 */}
      <PageHeader title="等待放币" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 放币超时提醒 */}
        {paidAt && (
          <ReleaseTimeoutAlert
            paidAt={paidAt}
            onDispute={handleDispute}
            onContactMaker={handleContactMaker}
          />
        )}

        {/* 等待状态 */}
        <View style={styles.section}>
          <Card style={styles.statusCard}>
            <Text style={styles.statusIcon}>⏳</Text>
            <Text style={styles.statusTitle}>等待做市商确认</Text>
            <Text style={styles.statusDesc}>
              做市商通常在 5-30 分钟内确认并释放 DUST
            </Text>
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
                已付款，等待放币
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
          </Card>
        </View>

        {/* 做市商信息 */}
        {maker && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>做市商信息</Text>
            <Card style={styles.makerCard}>
              <View style={styles.makerHeader}>
                <Text style={styles.makerName}>👤 {maker.maskedFullName}</Text>
                <View style={styles.makerRating}>
                  <Text style={styles.ratingText}>⭐ {maker.rating.toFixed(1)}</Text>
                </View>
              </View>
              <Text style={styles.makerStats}>
                已服务 {maker.usersServed} 位用户
              </Text>
            </Card>
          </View>
        )}

        {/* 操作按钮 */}
        <View style={styles.section}>
          <Button
            title="联系做市商"
            onPress={handleContactMaker}
            loading={isLoading}
            style={styles.contactButton}
          />

          <View style={styles.disputeContainer}>
            <Text style={styles.disputeLabel}>遇到问题？</Text>
            <TouchableOpacity onPress={handleDispute}>
              <Text style={styles.disputeLink}>申请仲裁</Text>
            </TouchableOpacity>
          </View>
        </View>

        {/* 提示信息 */}
        <View style={styles.section}>
          <Card style={styles.tipCard}>
            <Text style={styles.tipTitle}>💡 温馨提示</Text>
            <Text style={styles.tipText}>• 请耐心等待做市商确认</Text>
            <Text style={styles.tipText}>• 如超过 2 小时未放币，可申请仲裁</Text>
            <Text style={styles.tipText}>• 仲裁期间订单将被冻结</Text>
          </Card>
        </View>
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />

      {/* 联系做市商对话框 */}
      <ContactMakerDialog
        visible={showContactDialog}
        maker={maker}
        orderId={currentOrder.id}
        onClose={() => setShowContactDialog(false)}
      />

      {/* 申请仲裁对话框 */}
      <DisputeDialog
        visible={showDisputeDialog}
        order={currentOrder}
        onSubmit={handleDisputeSubmit}
        onClose={() => setShowDisputeDialog(false)}
      />
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
  statusCard: {
    alignItems: 'center',
  },
  statusIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  statusTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  statusDesc: {
    fontSize: 14,
    color: '#666666',
    textAlign: 'center',
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
    color: '#007AFF',
  },
  makerCard: {
    padding: 16,
  },
  makerHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  makerName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
  },
  makerRating: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  ratingText: {
    fontSize: 14,
    color: '#666666',
  },
  makerStats: {
    fontSize: 14,
    color: '#666666',
  },
  contactButton: {
    marginBottom: 16,
  },
  disputeContainer: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
  },
  disputeLabel: {
    fontSize: 14,
    color: '#666666',
    marginRight: 8,
  },
  disputeLink: {
    fontSize: 14,
    fontWeight: '600',
    color: '#FF3B30',
  },
  tipCard: {
    backgroundColor: '#FFF9F0',
  },
  tipTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  tipText: {
    fontSize: 13,
    color: '#666666',
    marginBottom: 4,
  },
});
