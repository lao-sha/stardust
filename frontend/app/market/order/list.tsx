// frontend/app/market/order/list.tsx

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  RefreshControl,
  SafeAreaView,
  StatusBar,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useWalletStore } from '@/stores/wallet.store';
import { useOrders } from '@/divination/market/hooks';
import {
  PriceDisplay,
  DivinationTypeBadge,
  OrderStatusBadge,
  LoadingSpinner,
  EmptyState,
} from '@/divination/market/components';
import { useAsync } from '@/hooks';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { Order, OrderStatus } from '@/divination/market/types';
import { formatTimeAgo, truncateAddress } from '@/divination/market/utils/market.utils';
import { ORDER_STATUS_CONFIG } from '@/divination/market/constants/market.constants';

type OrderType = 'my' | 'received';

const STATUS_TABS: { value: OrderStatus | 'all'; label: string }[] = [
  { value: 'all', label: '全部' },
  { value: 'PendingPayment', label: '待支付' },
  { value: 'Paid', label: '待接单' },
  { value: 'Accepted', label: '进行中' },
  { value: 'Completed', label: '已完成' },
  { value: 'Reviewed', label: '已评价' },
];

export default function OrderListScreen() {
  const router = useRouter();
  const params = useLocalSearchParams<{ type?: string; status?: string }>();
  const { address } = useWalletStore();
  const { getMyOrders, getReceivedOrders } = useOrders();
  const { execute, isLoading } = useAsync();

  const [orders, setOrders] = useState<Order[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [orderType, setOrderType] = useState<OrderType>(
    (params.type as OrderType) || 'my'
  );
  const [statusFilter, setStatusFilter] = useState<OrderStatus | 'all'>(
    (params.status as OrderStatus) || 'all'
  );

  const loadOrders = useCallback(async () => {
    if (!address) return;

    await execute(async () => {
      const fetchFn = orderType === 'my' ? getMyOrders : getReceivedOrders;
      const result = await fetchFn(
        statusFilter === 'all' ? undefined : statusFilter
      );
      setOrders(result);
    });
  }, [address, orderType, statusFilter, getMyOrders, getReceivedOrders, execute]);

  useEffect(() => {
    loadOrders();
  }, [loadOrders]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadOrders();
    setRefreshing(false);
  }, [loadOrders]);

  const renderOrderItem = ({ item }: { item: Order }) => (
    <TouchableOpacity
      style={[styles.orderCard, SHADOWS.small]}
      onPress={() => router.push(`/market/order/${item.id}`)}
      activeOpacity={0.7}
    >
      {/* 头部 */}
      <View style={styles.orderHeader}>
        <View style={styles.orderHeaderLeft}>
          <DivinationTypeBadge type={item.divinationType} size="small" />
          {item.isUrgent && (
            <View style={styles.urgentTag}>
              <Ionicons name="flash" size={10} color={THEME.warning} />
              <Text style={styles.urgentText}>加急</Text>
            </View>
          )}
        </View>
        <OrderStatusBadge status={item.status} size="small" />
      </View>

      {/* 中间信息 */}
      <View style={styles.orderBody}>
        <Text style={styles.orderId}>订单号: {item.id}</Text>
        <Text style={styles.orderPerson}>
          {orderType === 'my'
            ? `解卦师: ${item.providerName || truncateAddress(item.provider)}`
            : `客户: ${truncateAddress(item.customer)}`}
        </Text>
      </View>

      {/* 底部 */}
      <View style={styles.orderFooter}>
        <PriceDisplay amount={item.amount} size="small" />
        <Text style={styles.orderTime}>{formatTimeAgo(item.createdAt)}</Text>
      </View>
    </TouchableOpacity>
  );

  if (!address) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.header}>
          <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
            <Ionicons name="arrow-back" size={24} color={THEME.text} />
          </TouchableOpacity>
          <Text style={styles.headerTitle}>我的订单</Text>
          <View style={styles.backBtn} />
        </View>
        <EmptyState
          icon="wallet-outline"
          title="请先连接钱包"
          actionText="返回"
          onAction={() => router.back()}
        />
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.card} />

      {/* 顶部导航 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
          <Ionicons name="arrow-back" size={24} color={THEME.text} />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>
          {orderType === 'my' ? '我的订单' : '收到的订单'}
        </Text>
        <View style={styles.backBtn} />
      </View>

      {/* 订单类型切换 */}
      <View style={styles.typeSwitch}>
        <TouchableOpacity
          style={[styles.typeBtn, orderType === 'my' && styles.typeBtnActive]}
          onPress={() => setOrderType('my')}
        >
          <Text
            style={[
              styles.typeBtnText,
              orderType === 'my' && styles.typeBtnTextActive,
            ]}
          >
            我下的单
          </Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.typeBtn, orderType === 'received' && styles.typeBtnActive]}
          onPress={() => setOrderType('received')}
        >
          <Text
            style={[
              styles.typeBtnText,
              orderType === 'received' && styles.typeBtnTextActive,
            ]}
          >
            我接的单
          </Text>
        </TouchableOpacity>
      </View>

      {/* 状态筛选 */}
      <View style={styles.statusTabs}>
        <FlatList
          horizontal
          showsHorizontalScrollIndicator={false}
          data={STATUS_TABS}
          keyExtractor={(item) => item.value}
          contentContainerStyle={styles.statusTabsContent}
          renderItem={({ item }) => (
            <TouchableOpacity
              style={[
                styles.statusTab,
                statusFilter === item.value && styles.statusTabActive,
              ]}
              onPress={() => setStatusFilter(item.value)}
            >
              <Text
                style={[
                  styles.statusTabText,
                  statusFilter === item.value && styles.statusTabTextActive,
                ]}
              >
                {item.label}
              </Text>
            </TouchableOpacity>
          )}
        />
      </View>

      {/* 订单列表 */}
      <FlatList
        data={orders}
        keyExtractor={(item) => item.id.toString()}
        renderItem={renderOrderItem}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={onRefresh}
            colors={[THEME.primary]}
            tintColor={THEME.primary}
          />
        }
        ListEmptyComponent={
          isLoading ? (
            <LoadingSpinner text="加载中..." />
          ) : (
            <EmptyState
              icon="document-outline"
              title="暂无订单"
              description={
                statusFilter === 'all'
                  ? '您还没有相关订单'
                  : `没有${STATUS_TABS.find((t) => t.value === statusFilter)?.label}的订单`
              }
            />
          )
        }
      />
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: THEME.card,
    paddingHorizontal: 8,
    paddingVertical: 10,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  backBtn: {
    padding: 8,
    width: 40,
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME.text,
  },
  typeSwitch: {
    flexDirection: 'row',
    backgroundColor: THEME.card,
    paddingHorizontal: 16,
    paddingVertical: 10,
    gap: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  typeBtn: {
    paddingVertical: 6,
    paddingHorizontal: 16,
    borderRadius: 16,
    backgroundColor: THEME.background,
  },
  typeBtnActive: {
    backgroundColor: THEME.primary,
  },
  typeBtnText: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  typeBtnTextActive: {
    color: THEME.textInverse,
    fontWeight: '500',
  },
  statusTabs: {
    backgroundColor: THEME.card,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  statusTabsContent: {
    paddingHorizontal: 12,
    paddingVertical: 10,
    gap: 8,
  },
  statusTab: {
    paddingVertical: 4,
    paddingHorizontal: 12,
    borderRadius: 12,
    marginRight: 8,
  },
  statusTabActive: {
    backgroundColor: THEME.primary + '15',
  },
  statusTabText: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  statusTabTextActive: {
    color: THEME.primary,
    fontWeight: '500',
  },
  listContent: {
    padding: 16,
    paddingBottom: 32,
  },
  orderCard: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 14,
    marginBottom: 12,
  },
  orderHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 10,
  },
  orderHeaderLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  urgentTag: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.warning + '15',
    paddingHorizontal: 5,
    paddingVertical: 2,
    borderRadius: 4,
    gap: 2,
  },
  urgentText: {
    fontSize: 10,
    color: THEME.warning,
    fontWeight: '500',
  },
  orderBody: {
    marginBottom: 10,
  },
  orderId: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginBottom: 4,
  },
  orderPerson: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  orderFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingTop: 10,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: THEME.borderLight,
  },
  orderTime: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
});
