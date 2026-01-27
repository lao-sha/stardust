/**
 * 占卜师仪表盘页面
 */

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  ActivityIndicator,
  RefreshControl,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import {
  DashboardStats,
  DivinerOrderCard,
  Provider,
  ProviderStatus,
  ProviderTier,
  Order,
  OrderStatus,
  DivinationType,
} from '@/features/diviner';
import { divinationMarketService, OrderStatus as ServiceOrderStatus } from '@/services/divination-market.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';


export default function DivinerDashboardPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [provider, setProvider] = useState<Provider | null>(null);
  const [pendingOrders, setPendingOrders] = useState<Order[]>([]);
  const [availableBalance, setAvailableBalance] = useState(BigInt(0));
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingAction, setPendingAction] = useState<'pause' | 'resume' | null>(null);

  const loadData = useCallback(async () => {
    if (!address) return;

    try {
      // 从链上加载解卦师信息
      const providerData = await divinationMarketService.getProviderByAccount(address);
      if (providerData) {
        setProvider({
          account: providerData.account,
          name: providerData.name,
          bio: providerData.bio,
          specialties: providerData.specialties,
          supportedTypes: providerData.supportedTypes,
          status: providerData.status === 'Active' ? ProviderStatus.Active : 
                  providerData.status === 'Paused' ? ProviderStatus.Paused : ProviderStatus.Pending,
          tier: ProviderTier.Certified,
          totalOrders: providerData.totalOrders,
          completedOrders: providerData.completedOrders,
          totalEarnings: BigInt(0),
          averageRating: providerData.rating / 100,
          ratingCount: 0,
          acceptsUrgent: true,
          registeredAt: providerData.createdAt,
        });

        // 加载待处理订单
        const orders = await divinationMarketService.getProviderOrders(providerData.id);
        const pending = orders.filter((o: any) => o.status === ServiceOrderStatus.Paid || o.status === ServiceOrderStatus.Accepted);
        setPendingOrders(pending.map((o: any) => ({
          id: o.id,
          customer: o.customer,
          provider: address,
          packageId: o.packageId,
          divinationType: DivinationType.Meihua,
          questionCid: o.questionCid,
          totalAmount: o.amount,
          platformFee: o.amount / 10n,
          providerEarnings: o.amount * 9n / 10n,
          isUrgent: false,
          status: o.status === ServiceOrderStatus.Paid ? OrderStatus.Paid : OrderStatus.Accepted,
          createdAt: o.createdAt,
          followUpsUsed: 0,
          followUpsTotal: 3,
        })));
      }
    } catch (error) {
      console.error('Load dashboard data error:', error);
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

  // 暂停/恢复服务
  const handleToggleStatus = () => {
    if (!provider) return;

    const isPaused = provider.status === ProviderStatus.Paused;
    const action = isPaused ? 'resume' : 'pause';

    Alert.alert(
      isPaused ? '恢复服务' : '暂停服务',
      isPaused ? '恢复后您将可以接收新订单' : '暂停后您将不会接收新订单，已有订单不受影响',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确认',
          onPress: () => {
            if (!isSignerUnlocked()) {
              setPendingAction(action);
              setShowUnlockDialog(true);
              return;
            }
            executeToggleStatus(action);
          },
        },
      ]
    );
  };

  const executeToggleStatus = async (action: 'pause' | 'resume') => {
    setShowTxStatus(true);
    setTxStatus(action === 'pause' ? '正在暂停服务...' : '正在恢复服务...');

    try {
      if (action === 'pause') {
        await divinationMarketService.pauseProvider((status) => setTxStatus(status));
      } else {
        await divinationMarketService.resumeProvider((status) => setTxStatus(status));
      }

      setTxStatus('操作成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        loadData();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('操作失败', error.message || '请稍后重试');
    }
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      if (pendingAction) {
        await executeToggleStatus(pendingAction);
        setPendingAction(null);
      }
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const handleAcceptOrder = (orderId: number) => {
    router.push(`/diviner/orders/${orderId}` as any);
  };

  const handleRejectOrder = (orderId: number) => {
    router.push(`/diviner/orders/${orderId}` as any);
  };

  const handleViewOrder = (orderId: number) => {
    router.push(`/diviner/orders/${orderId}` as any);
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="占卜师中心" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  if (!provider) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="占卜师中心" />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>您还不是占卜师</Text>
          <Pressable style={styles.registerBtn} onPress={() => router.push('/diviner' as any)}>
            <Text style={styles.registerBtnText}>立即注册</Text>
          </Pressable>
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="占卜师中心" />

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={THEME_COLOR} />}
      >
        {/* 统计卡片 */}
        <View style={styles.section}>
          <DashboardStats
            provider={provider}
            pendingOrders={pendingOrders.filter(o => o.status === OrderStatus.Paid).length}
            todayEarnings={BigInt(85 * 1e10)}
            monthlyEarnings={BigInt(2350 * 1e10)}
            availableBalance={availableBalance}
          />
        </View>

        {/* 快捷操作 */}
        <View style={styles.section}>
          <View style={styles.quickActions}>
            <Pressable style={styles.actionItem} onPress={() => router.push('/diviner/orders' as any)}>
              <Text style={styles.actionIcon}>📋</Text>
              <Text style={styles.actionLabel}>订单管理</Text>
            </Pressable>
            <Pressable style={styles.actionItem} onPress={() => router.push('/diviner/packages' as any)}>
              <Text style={styles.actionIcon}>📦</Text>
              <Text style={styles.actionLabel}>套餐管理</Text>
            </Pressable>
            <Pressable style={styles.actionItem} onPress={() => router.push('/diviner/reviews' as any)}>
              <Text style={styles.actionIcon}>⭐</Text>
              <Text style={styles.actionLabel}>评价管理</Text>
            </Pressable>
            <Pressable style={styles.actionItem} onPress={() => router.push('/diviner/earnings' as any)}>
              <Text style={styles.actionIcon}>💰</Text>
              <Text style={styles.actionLabel}>收益提现</Text>
            </Pressable>
          </View>
        </View>

        {/* 待处理订单 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>待处理订单</Text>
            <Pressable onPress={() => router.push('/diviner/orders' as any)}>
              <Text style={styles.viewAllText}>查看全部 ›</Text>
            </Pressable>
          </View>

          {pendingOrders.length === 0 ? (
            <View style={styles.emptyOrders}>
              <Text style={styles.emptyOrdersText}>暂无待处理订单</Text>
            </View>
          ) : (
            pendingOrders.map(order => (
              <DivinerOrderCard
                key={order.id}
                order={order}
                onAccept={() => handleAcceptOrder(order.id)}
                onReject={() => handleRejectOrder(order.id)}
                onViewDetail={() => handleViewOrder(order.id)}
              />
            ))
          )}
        </View>

        {/* 服务状态卡片 */}
        <View style={styles.section}>
          <View style={styles.statusCard}>
            <View style={styles.statusInfo}>
              <Text style={styles.statusLabel}>服务状态</Text>
              <View style={[
                styles.statusBadge,
                { backgroundColor: provider.status === ProviderStatus.Active ? '#4CD96420' : '#FF950020' }
              ]}>
                <Text style={[
                  styles.statusBadgeText,
                  { color: provider.status === ProviderStatus.Active ? '#4CD964' : '#FF9500' }
                ]}>
                  {provider.status === ProviderStatus.Active ? '正常接单' : '已暂停'}
                </Text>
              </View>
            </View>
            <Pressable
              style={[
                styles.toggleButton,
                { backgroundColor: provider.status === ProviderStatus.Active ? '#FF950020' : '#4CD96420' }
              ]}
              onPress={handleToggleStatus}
            >
              <Ionicons
                name={provider.status === ProviderStatus.Active ? 'pause' : 'play'}
                size={16}
                color={provider.status === ProviderStatus.Active ? '#FF9500' : '#4CD964'}
              />
              <Text style={[
                styles.toggleButtonText,
                { color: provider.status === ProviderStatus.Active ? '#FF9500' : '#4CD964' }
              ]}>
                {provider.status === ProviderStatus.Active ? '暂停服务' : '恢复服务'}
              </Text>
            </Pressable>
          </View>
        </View>

        {/* 更多操作 */}
        <View style={styles.section}>
          <Pressable style={styles.menuItem} onPress={() => router.push('/diviner/profile' as any)}>
            <Text style={styles.menuIcon}>👤</Text>
            <Text style={styles.menuLabel}>编辑资料</Text>
            <Text style={styles.menuArrow}>›</Text>
          </Pressable>
          <Pressable style={styles.menuItem} onPress={() => router.push(`/diviner/${provider.account}` as any)}>
            <Text style={styles.menuIcon}>🔗</Text>
            <Text style={styles.menuLabel}>查看公开主页</Text>
            <Text style={styles.menuArrow}>›</Text>
          </Pressable>
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
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  emptyText: {
    fontSize: 16,
    color: '#666',
    marginBottom: 20,
  },
  registerBtn: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 32,
    paddingVertical: 12,
    borderRadius: 8,
  },
  registerBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
  section: {
    padding: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000',
  },
  viewAllText: {
    fontSize: 14,
    color: THEME_COLOR,
  },
  quickActions: {
    flexDirection: 'row',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  actionItem: {
    flex: 1,
    alignItems: 'center',
  },
  actionIcon: {
    fontSize: 28,
    marginBottom: 8,
  },
  actionLabel: {
    fontSize: 12,
    color: '#333',
  },
  emptyOrders: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 32,
    alignItems: 'center',
  },
  emptyOrdersText: {
    fontSize: 14,
    color: '#999',
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 8,
  },
  menuIcon: {
    fontSize: 20,
    marginRight: 12,
  },
  menuLabel: {
    flex: 1,
    fontSize: 16,
    color: '#333',
  },
  statusCard: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  statusInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  statusLabel: {
    fontSize: 14,
    color: '#666',
  },
  statusBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusBadgeText: {
    fontSize: 12,
    fontWeight: '500',
  },
  toggleButton: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 16,
    gap: 4,
  },
  toggleButtonText: {
    fontSize: 13,
    fontWeight: '500',
  },
  menuArrow: {
    fontSize: 20,
    color: '#C7C7CC',
  },
});
