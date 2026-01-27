/**
 * 做市商控制台首页
 * 路径: /maker/dashboard
 */

import React, { useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore, selectIsMaker, selectUnappealedPenaltiesCount } from '@/stores/maker.store';
import { MakerService } from '@/services/maker.service';
import { MakerStatusCard, DepositStatus } from '@/features/maker/components';
import { PageHeader } from '@/components/PageHeader';
import { Card, LoadingSpinner } from '@/components/common';

export default function DashboardPage() {
  const router = useRouter();
  const {
    makerApp,
    depositUsdValue,
    dustPrice,
    penalties,
    isLoading,
    refreshAll,
  } = useMakerStore();

  const isMaker = useMakerStore(selectIsMaker);
  const unappealedCount = useMakerStore(selectUnappealedPenaltiesCount);

  const [refreshing, setRefreshing] = React.useState(false);

  useEffect(() => {
    refreshAll();
  }, []);

  const onRefresh = async () => {
    setRefreshing(true);
    await refreshAll();
    setRefreshing(false);
  };

  if (isLoading && !makerApp) {
    return (
      <View style={styles.loadingContainer}>
        <LoadingSpinner text="加载中..." />
      </View>
    );
  }

  if (!isMaker || !makerApp) {
    return (
      <View style={styles.container}>
        <PageHeader title="做市商控制台" showBack />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>您还不是做市商</Text>
          <TouchableOpacity
            style={styles.applyButton}
            onPress={() => router.push('/maker')}
          >
            <Text style={styles.applyButtonText}>立即申请</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  }

  // 模拟今日统计数据
  const todayStats = {
    orders: 12,
    volume: 2500,
    earnings: 25,
  };

  return (
    <View style={styles.container}>
      <PageHeader title="做市商控制台" showBack />

      <ScrollView
        style={styles.content}
        showsVerticalScrollIndicator={false}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
      >
        {/* 做市商状态卡片 */}
        <MakerStatusCard
          maker={makerApp}
          depositUsdValue={depositUsdValue}
        />

        {/* 押金状态 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>💰 押金状态</Text>
            <TouchableOpacity onPress={() => router.push('/maker/deposit')}>
              <Text style={styles.sectionAction}>管理 →</Text>
            </TouchableOpacity>
          </View>
          <DepositStatus
            depositAmount={makerApp.deposit}
            depositUsdValue={depositUsdValue}
          />
        </View>

        {/* 今日统计 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>📊 今日统计</Text>
          <Card style={styles.section}>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{todayStats.orders}</Text>
              <Text style={styles.statLabel}>订单数</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{todayStats.volume.toLocaleString()}</Text>
              <Text style={styles.statLabel}>交易额 (USDT)</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={[styles.statValue, styles.statValueGreen]}>+{todayStats.earnings}</Text>
              <Text style={styles.statLabel}>收益 (USDT)</Text>
            </View>
          </Card>
        </View>

        {/* 快捷操作 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>快捷操作</Text>
          <View style={styles.actionGrid}>
            <TouchableOpacity
              style={styles.actionItem}
              onPress={() => router.push('/maker/deposit')}
            >
              <Text style={styles.actionIcon}>💰</Text>
              <Text style={styles.actionText}>押金管理</Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={styles.actionItem}
              onPress={() => router.push('/maker/settings')}
            >
              <Text style={styles.actionIcon}>⚙️</Text>
              <Text style={styles.actionText}>设置</Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={styles.actionItem}
              onPress={() => router.push('/maker/penalties')}
            >
              <View style={styles.actionIconContainer}>
                <Text style={styles.actionIcon}>📜</Text>
                {unappealedCount > 0 && (
                  <View style={styles.badge}>
                    <Text style={styles.badgeText}>{unappealedCount}</Text>
                  </View>
                )}
              </View>
              <Text style={styles.actionText}>扣除记录</Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={styles.actionItem}
              onPress={() => {/* TODO: 订单列表 */}}
            >
              <Text style={styles.actionIcon}>📝</Text>
              <Text style={styles.actionText}>订单</Text>
            </TouchableOpacity>
          </View>
        </View>

        {/* 押金不足警告 */}
        {makerApp.depositWarning && (
          <TouchableOpacity
            style={styles.warningCard}
            onPress={() => router.push('/maker/deposit/replenish')}
          >
            <Text style={styles.warningIcon}>⚠️</Text>
            <View style={styles.warningContent}>
              <Text style={styles.warningTitle}>押金不足警告</Text>
              <Text style={styles.warningDesc}>
                您的押金价值已低于 $950 阈值，请及时补充
              </Text>
            </View>
            <Text style={styles.warningArrow}>→</Text>
          </TouchableOpacity>
        )}

        {/* 溢价信息 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>溢价设置</Text>
            <TouchableOpacity onPress={() => router.push('/maker/settings')}>
              <Text style={styles.sectionAction}>修改 →</Text>
            </TouchableOpacity>
          </View>
          <View style={styles.premiumCard}>
            <View style={styles.premiumItem}>
              <Text style={styles.premiumLabel}>买入溢价 (Bridge)</Text>
              <Text style={[styles.premiumValue, makerApp.buyPremiumBps >= 0 ? styles.premiumPositive : styles.premiumNegative]}>
                {makerApp.buyPremiumBps >= 0 ? '+' : ''}{(makerApp.buyPremiumBps / 100).toFixed(1)}%
              </Text>
            </View>
            <View style={styles.premiumItem}>
              <Text style={styles.premiumLabel}>卖出溢价 (OTC)</Text>
              <Text style={[styles.premiumValue, makerApp.sellPremiumBps >= 0 ? styles.premiumPositive : styles.premiumNegative]}>
                {makerApp.sellPremiumBps >= 0 ? '+' : ''}{(makerApp.sellPremiumBps / 100).toFixed(1)}%
              </Text>
            </View>
          </View>
        </View>
      </ScrollView>
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
  section: {
    marginTop: 20,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 12,
  },
  sectionAction: {
    fontSize: 14,
    color: '#B2955D',
  },
  section: {
    marginBottom: 16,
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 20,
    fontWeight: '700',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  statValueGreen: {
    color: '#4CD964',
  },
  statLabel: {
    fontSize: 12,
    color: '#8E8E93',
  },
  statDivider: {
    width: 1,
    backgroundColor: '#F2F2F7',
    marginHorizontal: 8,
  },
  // 快捷操作
  actionGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 12,
  },
  actionItem: {
    width: '47%',
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
  },
  actionIconContainer: {
    position: 'relative',
  },
  actionIcon: {
    fontSize: 28,
    marginBottom: 8,
  },
  actionText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1C1C1E',
  },
  badge: {
    position: 'absolute',
    top: -4,
    right: -8,
    backgroundColor: '#FF3B30',
    borderRadius: 10,
    minWidth: 18,
    height: 18,
    justifyContent: 'center',
    alignItems: 'center',
  },
  badgeText: {
    fontSize: 11,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  // 警告卡片
  warningCard: {
    backgroundColor: '#FF950020',
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 20,
  },
  warningIcon: {
    fontSize: 24,
    marginRight: 12,
  },
  warningContent: {
    flex: 1,
  },
  warningTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#FF9500',
    marginBottom: 4,
  },
  warningDesc: {
    fontSize: 13,
    color: '#996600',
  },
  warningArrow: {
    fontSize: 18,
    color: '#FF9500',
  },
  // 溢价卡片
  premiumCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
  },
  premiumItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  premiumLabel: {
    fontSize: 14,
    color: '#8E8E93',
  },
  premiumValue: {
    fontSize: 16,
    fontWeight: '600',
  },
  premiumPositive: {
    color: '#4CD964',
  },
  premiumNegative: {
    color: '#FF3B30',
  },
});
