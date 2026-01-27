/**
 * 购买 DUST 首页
 * 显示当前价格、首购特惠、做市商列表
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useTradingStore } from '@/stores/trading.store';
import {
  PriceDisplay,
  MakerCard,
  MakerOfflineWarning,
  TradingErrorBoundary,
} from '@/features/trading/components';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import { Card, Button, LoadingSpinner, EmptyState } from '@/components/common';
import { useAsync } from '@/hooks';
import type { Maker } from '@/stores/trading.store';

function BuyDustPageContent() {
  const router = useRouter();
  const {
    makers,
    loadingMakers,
    dustPrice,
    marketStats,
    isFirstPurchase,
    hasCompletedFirstPurchase,
    fetchMakers,
    fetchMarketStats,
    checkFirstPurchaseStatus,
    selectMaker,
  } = useTradingStore();

  const { execute, isLoading } = useAsync();
  const [showOfflineWarning, setShowOfflineWarning] = useState(false);
  const [pendingMaker, setPendingMaker] = useState<Maker | null>(null);

  useEffect(() => {
    // 初始化数据
    execute(async () => {
      await Promise.all([
        fetchMakers(),
        fetchMarketStats(),
        checkFirstPurchaseStatus(),
      ]);
    });
  }, []);

  const handleFirstPurchase = () => {
    router.push('/wallet/buy-dust/first-purchase');
  };

  const handleSelectMaker = (makerId: number) => {
    const maker = makers.find(m => m.id === makerId);
    if (!maker) return;

    // 检查做市商是否离线
    if (maker.isOnline === false) {
      setPendingMaker(maker);
      setShowOfflineWarning(true);
      return;
    }

    // 正常流程
    proceedWithMaker(maker);
  };

  const proceedWithMaker = (maker: Maker) => {
    selectMaker(maker.id);
    if (isFirstPurchase && !hasCompletedFirstPurchase) {
      router.push('/wallet/buy-dust/first-purchase');
    } else {
      router.push('/wallet/buy-dust/order');
    }
  };

  const handleOfflineConfirm = () => {
    setShowOfflineWarning(false);
    if (pendingMaker) {
      proceedWithMaker(pendingMaker);
      setPendingMaker(null);
    }
  };

  const handleOfflineCancel = () => {
    setShowOfflineWarning(false);
    setPendingMaker(null);
  };

  // 计算在线做市商数量
  const onlineMakersCount = makers.filter(m => m.isOnline !== false).length;

  return (
    <View style={styles.wrapper}>
      {/* 页面头部 */}
      <PageHeader title="购买 DUST" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 价格显示 */}
        <View style={styles.section}>
          <PriceDisplay
            price={marketStats?.weightedPrice || dustPrice || 0.10}
            priceChange24h={marketStats?.priceChange24h}
            label="💰 当前价格"
          />
        </View>

        {/* 首购特惠 */}
        {isFirstPurchase && !hasCompletedFirstPurchase && (
          <View style={styles.section}>
            <Card style={styles.firstPurchaseCard}>
              <Text style={styles.firstPurchaseTitle}>🎁 首购特惠</Text>
              <Text style={styles.firstPurchaseDesc}>
                首次购买固定 10 USD
              </Text>
              <Text style={styles.firstPurchaseDesc}>
                享受新用户专属价格
              </Text>
              <Button
                title="立即首购"
                onPress={handleFirstPurchase}
                style={styles.firstPurchaseButton}
              />
            </Card>
          </View>
        )}

        {/* 做市商列表 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>
              {isFirstPurchase && !hasCompletedFirstPurchase
                ? '或选择做市商'
                : '选择做市商'}
            </Text>
            <Text style={styles.sectionSubtitle}>
              {onlineMakersCount} 位做市商在线
            </Text>
          </View>

          {loadingMakers || isLoading ? (
            <LoadingSpinner text="加载做市商列表..." />
          ) : makers.length === 0 ? (
            <EmptyState
              icon="people-outline"
              title="暂无可用做市商"
              description="请稍后再试"
            />
          ) : (
            makers.map((maker) => (
              <MakerCard
                key={maker.id}
                maker={maker}
                onPress={() => handleSelectMaker(maker.id)}
              />
            ))
          )}
        </View>

        {/* 底部说明 */}
        <View style={styles.footer}>
          <Text style={styles.footerTitle}>💡 购买说明</Text>
          <Text style={styles.footerText}>• 首次购买固定 10 USD</Text>
          <Text style={styles.footerText}>• 普通订单 20-200 USD</Text>
          <Text style={styles.footerText}>• 支付方式：USDT (TRC20)</Text>
          <Text style={styles.footerText}>• 订单超时：30 分钟</Text>
        </View>
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />

      {/* 做市商离线警告 */}
      {pendingMaker && (
        <MakerOfflineWarning
          visible={showOfflineWarning}
          maker={pendingMaker}
          onConfirm={handleOfflineConfirm}
          onCancel={handleOfflineCancel}
        />
      )}
    </View>
  );
}

export default function BuyDustPage() {
  return (
    <TradingErrorBoundary>
      <BuyDustPageContent />
    </TradingErrorBoundary>
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
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000000',
  },
  sectionSubtitle: {
    fontSize: 14,
    color: '#666666',
  },
  firstPurchaseCard: {
    backgroundColor: '#FFF9F0',
    borderWidth: 2,
    borderColor: '#B2955D',
  },
  firstPurchaseTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  firstPurchaseDesc: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 4,
  },
  firstPurchaseButton: {
    marginTop: 16,
  },
  footer: {
    padding: 16,
    paddingBottom: 32,
  },
  footerTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  footerText: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 6,
  },
});
