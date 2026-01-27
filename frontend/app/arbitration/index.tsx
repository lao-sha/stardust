/**
 * 仲裁模块首页
 * 显示用户的争议列表
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  Pressable,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { arbitrationService, Dispute, DisputeStatus, DisputeType } from '@/services/arbitration.service';
import { useWalletStore } from '@/stores/wallet.store';

const THEME_COLOR = '#B2955D';

export default function ArbitrationIndexPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [disputes, setDisputes] = useState<Dispute[]>([]);

  const loadDisputes = useCallback(async () => {
    if (!address) return;

    try {
      const userDisputes = await arbitrationService.getUserDisputes(address);
      setDisputes(userDisputes);
    } catch (error) {
      console.error('Load disputes error:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [address]);

  useEffect(() => {
    loadDisputes();
  }, [loadDisputes]);

  const onRefresh = () => {
    setRefreshing(true);
    loadDisputes();
  };

  const getStatusText = (status: DisputeStatus) => {
    switch (status) {
      case DisputeStatus.Pending:
        return '待处理';
      case DisputeStatus.UnderReview:
        return '审核中';
      case DisputeStatus.Resolved:
        return '已解决';
      case DisputeStatus.Appealed:
        return '已申诉';
      case DisputeStatus.Closed:
        return '已关闭';
      default:
        return '未知';
    }
  };

  const getStatusColor = (status: DisputeStatus) => {
    switch (status) {
      case DisputeStatus.Pending:
        return '#FF9500';
      case DisputeStatus.UnderReview:
        return '#4A90D9';
      case DisputeStatus.Resolved:
        return '#4CD964';
      case DisputeStatus.Appealed:
        return '#FF6B6B';
      case DisputeStatus.Closed:
        return '#999';
      default:
        return '#999';
    }
  };

  const getTypeText = (type: DisputeType) => {
    switch (type) {
      case DisputeType.Order:
        return '订单争议';
      case DisputeType.Swap:
        return '兑换争议';
      case DisputeType.Service:
        return '服务争议';
      default:
        return '其他争议';
    }
  };

  const renderDisputeItem = ({ item }: { item: Dispute }) => {
    const isPlaintiff = item.plaintiff === address;

    return (
      <Pressable
        style={styles.disputeItem}
        onPress={() => router.push(`/arbitration/${item.id}` as any)}
      >
        <View style={styles.disputeHeader}>
          <View style={styles.disputeType}>
            <Ionicons name="shield-checkmark" size={20} color={THEME_COLOR} />
            <Text style={styles.disputeTypeText}>{getTypeText(item.disputeType)}</Text>
          </View>
          <View style={[styles.statusBadge, { backgroundColor: getStatusColor(item.status) + '20' }]}>
            <Text style={[styles.statusText, { color: getStatusColor(item.status) }]}>
              {getStatusText(item.status)}
            </Text>
          </View>
        </View>

        <Text style={styles.disputeReason} numberOfLines={2}>
          {item.reason}
        </Text>

        <View style={styles.disputeInfo}>
          <Text style={styles.disputeRole}>
            {isPlaintiff ? '我是原告' : '我是被告'}
          </Text>
          <Text style={styles.disputeTime}>
            {new Date(item.createdAt * 1000).toLocaleDateString()}
          </Text>
        </View>

        <View style={styles.disputeArrow}>
          <Ionicons name="chevron-forward" size={20} color="#999" />
        </View>
      </Pressable>
    );
  };

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="争议仲裁" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="争议仲裁" showBack />

      {disputes.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Ionicons name="shield-checkmark-outline" size={80} color="#ccc" />
          <Text style={styles.emptyTitle}>暂无争议</Text>
          <Text style={styles.emptyText}>
            如果您在交易中遇到问题，可以发起争议
          </Text>
          <Pressable
            style={styles.createButton}
            onPress={() => router.push('/arbitration/create' as any)}
          >
            <Text style={styles.createButtonText}>发起争议</Text>
          </Pressable>
        </View>
      ) : (
        <>
          <FlatList
            data={disputes}
            renderItem={renderDisputeItem}
            keyExtractor={(item) => item.id.toString()}
            contentContainerStyle={styles.listContent}
            refreshControl={
              <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
            }
          />
          <Pressable
            style={styles.fab}
            onPress={() => router.push('/arbitration/create' as any)}
          >
            <Ionicons name="add" size={28} color="#fff" />
          </Pressable>
        </>
      )}

      <BottomNavBar />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
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
  emptyTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#333',
    marginTop: 20,
  },
  emptyText: {
    fontSize: 14,
    color: '#666',
    marginTop: 8,
    textAlign: 'center',
  },
  createButton: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 20,
    marginTop: 20,
  },
  createButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  listContent: {
    padding: 16,
  },
  disputeItem: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  disputeHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  disputeType: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  disputeTypeText: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
    marginLeft: 8,
  },
  statusBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  disputeReason: {
    fontSize: 14,
    color: '#666',
    marginTop: 12,
    lineHeight: 20,
  },
  disputeInfo: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 12,
  },
  disputeRole: {
    fontSize: 13,
    color: THEME_COLOR,
  },
  disputeTime: {
    fontSize: 12,
    color: '#999',
  },
  disputeArrow: {
    position: 'absolute',
    right: 16,
    top: '50%',
  },
  fab: {
    position: 'absolute',
    right: 20,
    bottom: 100,
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.2,
    shadowRadius: 4,
    elevation: 4,
  },
});
