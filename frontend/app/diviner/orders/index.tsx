/**
 * 订单管理列表页面
 */

import React, { useEffect, useState } from 'react';
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
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import {
  DivinerOrderCard,
  Order,
  OrderStatus,
  ORDER_STATUS_CONFIG,
  DivinationType,
} from '@/features/diviner';

const THEME_COLOR = '#B2955D';

// Mock 数据
const mockOrders: Order[] = [
  {
    id: 1001,
    customer: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    packageId: 1,
    divinationType: DivinationType.Meihua,
    questionCid: 'QmXxx...',
    totalAmount: BigInt(10 * 1e10),
    platformFee: BigInt(1.5 * 1e10),
    providerEarnings: BigInt(8.5 * 1e10),
    isUrgent: false,
    status: OrderStatus.Paid,
    createdAt: Date.now() - 3600000,
    followUpsUsed: 0,
    followUpsTotal: 3,
  },
  {
    id: 1002,
    customer: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
    provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    packageId: 2,
    divinationType: DivinationType.Bazi,
    questionCid: 'QmYyy...',
    totalAmount: BigInt(25 * 1e10),
    platformFee: BigInt(3.75 * 1e10),
    providerEarnings: BigInt(21.25 * 1e10),
    isUrgent: true,
    status: OrderStatus.Accepted,
    createdAt: Date.now() - 7200000,
    acceptedAt: Date.now() - 3600000,
    followUpsUsed: 1,
    followUpsTotal: 5,
  },
  {
    id: 1003,
    customer: '5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw',
    provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    packageId: 1,
    divinationType: DivinationType.Meihua,
    questionCid: 'QmZzz...',
    answerCid: 'QmAaa...',
    totalAmount: BigInt(10 * 1e10),
    platformFee: BigInt(1.5 * 1e10),
    providerEarnings: BigInt(8.5 * 1e10),
    isUrgent: false,
    status: OrderStatus.Completed,
    createdAt: Date.now() - 86400000,
    acceptedAt: Date.now() - 82800000,
    completedAt: Date.now() - 79200000,
    followUpsUsed: 2,
    followUpsTotal: 3,
  },
];

type FilterStatus = 'all' | OrderStatus;

export default function OrdersListPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [orders, setOrders] = useState<Order[]>([]);
  const [filter, setFilter] = useState<FilterStatus>('all');

  const loadData = async () => {
    await new Promise(resolve => setTimeout(resolve, 500));
    setOrders(mockOrders);
  };

  useEffect(() => {
    loadData().finally(() => setLoading(false));
  }, []);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const filteredOrders = filter === 'all'
    ? orders
    : orders.filter(o => o.status === filter);

  const handleViewOrder = (orderId: number) => {
    router.push(`/diviner/orders/${orderId}` as any);
  };

  const handleAcceptOrder = async (orderId: number) => {
    try {
      const { divinationMarketService } = await import('@/services/divination-market.service');
      await divinationMarketService.acceptOrder(orderId, (status) => {
        console.log('Accept order status:', status);
      });
      // 更新本地状态
      setOrders(prev =>
        prev.map(o => (o.id === orderId ? { ...o, status: OrderStatus.Accepted, acceptedAt: Date.now() } : o))
      );
    } catch (error: any) {
      Alert.alert('接单失败', error.message || '请稍后重试');
    }
  };

  const handleRejectOrder = async (orderId: number) => {
    try {
      const { divinationMarketService } = await import('@/services/divination-market.service');
      await divinationMarketService.rejectOrder(orderId, '暂时无法接单', (status) => {
        console.log('Reject order status:', status);
      });
      // 更新本地状态
      setOrders(prev =>
        prev.map(o => (o.id === orderId ? { ...o, status: OrderStatus.Cancelled } : o))
      );
    } catch (error: any) {
      Alert.alert('拒单失败', error.message || '请稍后重试');
    }
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="订单管理" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="订单管理" />

      {/* 筛选标签 */}
      <View style={styles.filterContainer}>
        <ScrollView horizontal showsHorizontalScrollIndicator={false} contentContainerStyle={styles.filterScroll}>
          <Pressable
            style={[styles.filterTab, filter === 'all' && styles.filterTabActive]}
            onPress={() => setFilter('all')}
          >
            <Text style={[styles.filterText, filter === 'all' && styles.filterTextActive]}>全部</Text>
          </Pressable>
          {[OrderStatus.Paid, OrderStatus.Accepted, OrderStatus.Completed, OrderStatus.Cancelled].map(status => (
            <Pressable
              key={status}
              style={[styles.filterTab, filter === status && styles.filterTabActive]}
              onPress={() => setFilter(status)}
            >
              <Text style={[styles.filterText, filter === status && styles.filterTextActive]}>
                {ORDER_STATUS_CONFIG[status].label}
              </Text>
            </Pressable>
          ))}
        </ScrollView>
      </View>

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={THEME_COLOR} />}
      >
        {filteredOrders.length === 0 ? (
          <View style={styles.emptyContainer}>
            <Text style={styles.emptyIcon}>📋</Text>
            <Text style={styles.emptyText}>暂无订单</Text>
          </View>
        ) : (
          filteredOrders.map(order => (
            <DivinerOrderCard
              key={order.id}
              order={order}
              onAccept={() => handleAcceptOrder(order.id)}
              onReject={() => handleRejectOrder(order.id)}
              onViewDetail={() => handleViewOrder(order.id)}
            />
          ))
        )}
      </ScrollView>

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
    padding: 16,
    paddingBottom: 100,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  filterContainer: {
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  filterScroll: {
    paddingHorizontal: 12,
    paddingVertical: 12,
    gap: 8,
    flexDirection: 'row',
  },
  filterTab: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
    backgroundColor: '#F5F5F7',
  },
  filterTabActive: {
    backgroundColor: THEME_COLOR,
  },
  filterText: {
    fontSize: 14,
    color: '#666',
  },
  filterTextActive: {
    color: '#FFF',
    fontWeight: '500',
  },
  emptyContainer: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 48,
    alignItems: 'center',
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
  },
});
