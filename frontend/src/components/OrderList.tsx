/**
 * 星尘玄鉴 - 订单列表组件
 * 
 * 使用虚拟滚动和分页加载的高性能订单列表
 */

import React, { useCallback, useMemo } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  ListRenderItem,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { VirtualizedList } from './VirtualizedList';
import { usePaginatedList } from '@/hooks/usePaginatedList';
import type { Order, OrderStatus } from '@/divination/market/types';

// ==================== 类型定义 ====================

interface OrderListProps {
  /** 获取订单的函数 */
  fetchOrders: (page: number, pageSize: number, status?: OrderStatus) => Promise<{
    data: Order[];
    total?: number;
    hasMore?: boolean;
  }>;
  /** 状态筛选 */
  statusFilter?: OrderStatus | 'all';
  /** 点击订单回调 */
  onOrderPress?: (order: Order) => void;
  /** 每页数量 */
  pageSize?: number;
  /** 列表头部组件 */
  headerComponent?: React.ReactNode;
  /** 空状态文本 */
  emptyText?: string;
}

// ==================== 订单卡片组件 ====================

interface OrderCardProps {
  order: Order;
  onPress?: (order: Order) => void;
}

function OrderCard({ order, onPress }: OrderCardProps): React.ReactElement {
  const handlePress = useCallback(() => {
    onPress?.(order);
  }, [order, onPress]);

  const statusColor = useMemo(() => {
    const status = order.status as string;
    switch (status) {
      case 'PendingPayment':
        return '#f59e0b';
      case 'Paid':
        return '#3b82f6';
      case 'Accepted':
        return '#8b5cf6';
      case 'Completed':
        return '#10b981';
      case 'Reviewed':
        return '#6b7280';
      case 'Cancelled':
        return '#ef4444';
      case 'Disputed':
        return '#dc2626';
      default:
        return '#6b7280';
    }
  }, [order.status]);

  const statusText = useMemo(() => {
    const status = order.status as string;
    switch (status) {
      case 'PendingPayment':
        return '待支付';
      case 'Paid':
        return '待接单';
      case 'Accepted':
        return '进行中';
      case 'Completed':
        return '已完成';
      case 'Reviewed':
        return '已评价';
      case 'Cancelled':
        return '已取消';
      case 'Disputed':
        return '争议中';
      default:
        return status;
    }
  }, [order.status]);

  return (
    <TouchableOpacity
      style={styles.orderCard}
      onPress={handlePress}
      activeOpacity={0.7}
    >
      {/* 头部 */}
      <View style={styles.orderHeader}>
        <View style={styles.orderHeaderLeft}>
          <View style={styles.typeBadge}>
            <Text style={styles.typeBadgeText}>
              {getDivinationTypeName(order.divinationType)}
            </Text>
          </View>
          {order.isUrgent && (
            <View style={styles.urgentTag}>
              <Ionicons name="flash" size={10} color="#f59e0b" />
              <Text style={styles.urgentText}>加急</Text>
            </View>
          )}
        </View>
        <View style={[styles.statusBadge, { backgroundColor: statusColor + '20' }]}>
          <Text style={[styles.statusText, { color: statusColor }]}>
            {statusText}
          </Text>
        </View>
      </View>

      {/* 中间信息 */}
      <View style={styles.orderBody}>
        <Text style={styles.orderId}>订单号: {order.id}</Text>
        <Text style={styles.orderProvider}>
          解卦师: {order.providerName || truncateAddress(order.provider)}
        </Text>
      </View>

      {/* 底部 */}
      <View style={styles.orderFooter}>
        <Text style={styles.orderAmount}>
          {formatDustAmount(order.amount)} DUST
        </Text>
        <Text style={styles.orderTime}>{formatTimeAgo(order.createdAt)}</Text>
      </View>
    </TouchableOpacity>
  );
}

// ==================== 主组件 ====================

export function OrderList({
  fetchOrders,
  statusFilter = 'all',
  onOrderPress,
  pageSize = 20,
  headerComponent,
  emptyText = '暂无订单',
}: OrderListProps): React.ReactElement {
  // 使用分页 Hook
  const {
    data: orders,
    pagination,
    isLoading,
    isRefreshing,
    isLoadingMore,
    error,
    refresh,
    loadMore,
    retry,
  } = usePaginatedList<Order>({
    fetchData: async (page, size) => {
      const status = statusFilter === 'all' ? undefined : statusFilter;
      return fetchOrders(page, size, status);
    },
    pageSize,
    getItemKey: (order) => order.id,
  });

  // 渲染订单项
  const renderItem: ListRenderItem<Order> = useCallback(
    ({ item }) => <OrderCard order={item} onPress={onOrderPress} />,
    [onOrderPress]
  );

  // 键提取器
  const keyExtractor = useCallback((item: Order) => item.id.toString(), []);

  return (
    <VirtualizedList
      data={orders}
      renderItem={renderItem}
      keyExtractor={keyExtractor}
      pagination={pagination}
      onLoadMore={loadMore}
      isLoadingMore={isLoadingMore}
      onRefresh={refresh}
      isRefreshing={isRefreshing}
      isLoading={isLoading}
      error={error}
      onRetry={retry}
      headerComponent={headerComponent}
      emptyText={emptyText}
      emptyIcon="📋"
      estimatedItemSize={120}
      containerStyle={styles.container}
    />
  );
}

// ==================== 辅助函数 ====================

function getDivinationTypeName(type: number): string {
  const types: Record<number, string> = {
    0: '八字',
    1: '紫微',
    2: '奇门',
    3: '六爻',
    4: '梅花',
    5: '塔罗',
    6: '大六壬',
    7: '小六壬',
  };
  return types[type] ?? '占卜';
}

function truncateAddress(address: string): string {
  if (!address || address.length < 12) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

function formatDustAmount(amount: bigint | number): string {
  const num = typeof amount === 'bigint' ? Number(amount) : amount;
  return (num / 1e12).toFixed(2);
}

function formatTimeAgo(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;
  
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);
  
  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes}分钟前`;
  if (hours < 24) return `${hours}小时前`;
  if (days < 30) return `${days}天前`;
  
  const date = new Date(timestamp);
  return `${date.getMonth() + 1}/${date.getDate()}`;
}

// ==================== 样式 ====================

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  orderCard: {
    backgroundColor: '#ffffff',
    borderRadius: 12,
    padding: 14,
    marginHorizontal: 16,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 1,
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
  typeBadge: {
    backgroundColor: '#e94560',
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 4,
  },
  typeBadgeText: {
    fontSize: 11,
    color: '#ffffff',
    fontWeight: '500',
  },
  urgentTag: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fef3c7',
    paddingHorizontal: 5,
    paddingVertical: 2,
    borderRadius: 4,
    gap: 2,
  },
  urgentText: {
    fontSize: 10,
    color: '#f59e0b',
    fontWeight: '500',
  },
  statusBadge: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 4,
  },
  statusText: {
    fontSize: 11,
    fontWeight: '500',
  },
  orderBody: {
    marginBottom: 10,
  },
  orderId: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 4,
  },
  orderProvider: {
    fontSize: 13,
    color: '#6b7280',
  },
  orderFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingTop: 10,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: '#e5e7eb',
  },
  orderAmount: {
    fontSize: 14,
    fontWeight: '600',
    color: '#e94560',
  },
  orderTime: {
    fontSize: 12,
    color: '#9ca3af',
  },
});

export default OrderList;
