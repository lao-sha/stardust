/**
 * 合婚请求列表页面
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  Pressable,
  Alert,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { matchmakingService, MatchRequest, MatchRequestStatus } from '@/services/matchmaking.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

export default function RequestsPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [requests, setRequests] = useState<MatchRequest[]>([]);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingAction, setPendingAction] = useState<{ requestId: number; action: 'authorize' | 'reject' } | null>(null);

  const loadRequests = useCallback(async () => {
    if (!address) return;

    try {
      const requestIds = await matchmakingService.getUserMatchRequests(address);
      const requestList: MatchRequest[] = [];

      for (const id of requestIds) {
        const request = await matchmakingService.getMatchRequest(id);
        if (request) {
          requestList.push(request);
        }
      }

      setRequests(requestList);
    } catch (error) {
      console.error('Load requests error:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [address]);

  useEffect(() => {
    loadRequests();
  }, [loadRequests]);

  const onRefresh = () => {
    setRefreshing(true);
    loadRequests();
  };

  const handleAction = async (requestId: number, action: 'authorize' | 'reject') => {
    if (!isSignerUnlocked()) {
      setPendingAction({ requestId, action });
      setShowUnlockDialog(true);
      return;
    }

    await executeAction(requestId, action);
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      if (pendingAction) {
        await executeAction(pendingAction.requestId, pendingAction.action);
        setPendingAction(null);
      }
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeAction = async (requestId: number, action: 'authorize' | 'reject') => {
    setShowTxStatus(true);
    const actionText = action === 'authorize' ? '授权' : '拒绝';
    setTxStatus(`正在${actionText}...`);

    try {
      if (action === 'authorize') {
        await matchmakingService.authorizeMatchRequest(requestId, (status) => setTxStatus(status));
      } else {
        await matchmakingService.rejectMatchRequest(requestId, (status) => setTxStatus(status));
      }

      setTxStatus('操作成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        loadRequests();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('操作失败', error.message || '请稍后重试');
    }
  };

  const getStatusText = (status: MatchRequestStatus) => {
    switch (status) {
      case MatchRequestStatus.Pending:
        return '待授权';
      case MatchRequestStatus.Authorized:
        return '已授权';
      case MatchRequestStatus.Rejected:
        return '已拒绝';
      case MatchRequestStatus.Completed:
        return '已完成';
      case MatchRequestStatus.Cancelled:
        return '已取消';
      default:
        return '未知';
    }
  };

  const getStatusColor = (status: MatchRequestStatus) => {
    switch (status) {
      case MatchRequestStatus.Pending:
        return '#FF9500';
      case MatchRequestStatus.Authorized:
        return '#4CD964';
      case MatchRequestStatus.Rejected:
        return '#FF3B30';
      case MatchRequestStatus.Completed:
        return THEME_COLOR;
      case MatchRequestStatus.Cancelled:
        return '#999';
      default:
        return '#999';
    }
  };

  const renderRequestItem = ({ item }: { item: MatchRequest }) => {
    const isRequester = item.requester === address;
    const isPending = item.status === MatchRequestStatus.Pending;

    return (
      <View style={styles.requestItem}>
        <View style={styles.requestHeader}>
          <View style={styles.requestType}>
            <Ionicons name="git-compare" size={20} color={THEME_COLOR} />
            <Text style={styles.requestTypeText}>八字合婚请求</Text>
          </View>
          <View style={[styles.statusBadge, { backgroundColor: getStatusColor(item.status) + '20' }]}>
            <Text style={[styles.statusText, { color: getStatusColor(item.status) }]}>
              {getStatusText(item.status)}
            </Text>
          </View>
        </View>

        <View style={styles.requestInfo}>
          <Text style={styles.requestLabel}>
            {isRequester ? '发起方（我）' : '接收方（我）'}
          </Text>
          <Text style={styles.requestAddress}>
            {isRequester ? `对方: ${item.target.slice(0, 8)}...${item.target.slice(-6)}` : `发起人: ${item.requester.slice(0, 8)}...${item.requester.slice(-6)}`}
          </Text>
        </View>

        <Text style={styles.requestTime}>
          创建于 {new Date(item.createdAt * 1000).toLocaleString()}
        </Text>

        {/* 操作按钮 - 仅接收方且待授权状态显示 */}
        {!isRequester && isPending && (
          <View style={styles.actionButtons}>
            <Pressable
              style={[styles.actionButton, styles.rejectButton]}
              onPress={() => handleAction(item.id, 'reject')}
            >
              <Text style={styles.rejectButtonText}>拒绝</Text>
            </Pressable>
            <Pressable
              style={[styles.actionButton, styles.authorizeButton]}
              onPress={() => handleAction(item.id, 'authorize')}
            >
              <Text style={styles.authorizeButtonText}>授权</Text>
            </Pressable>
          </View>
        )}

        {/* 查看报告按钮 - 已完成状态显示 */}
        {item.status === MatchRequestStatus.Completed && item.reportCid && (
          <Pressable
            style={styles.viewReportButton}
            onPress={() => router.push(`/matchmaking/report/${item.id}` as any)}
          >
            <Ionicons name="document-text" size={16} color={THEME_COLOR} />
            <Text style={styles.viewReportText}>查看合婚报告</Text>
          </Pressable>
        )}
      </View>
    );
  };

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="合婚请求" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="合婚请求" showBack />

      {requests.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Ionicons name="git-merge-outline" size={80} color="#ccc" />
          <Text style={styles.emptyTitle}>暂无请求</Text>
          <Text style={styles.emptyText}>
            匹配成功后可发起八字合婚请求
          </Text>
        </View>
      ) : (
        <FlatList
          data={requests}
          renderItem={renderRequestItem}
          keyExtractor={(item) => item.id.toString()}
          contentContainerStyle={styles.listContent}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }
        />
      )}

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
  },
  listContent: {
    padding: 16,
  },
  requestItem: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  requestHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  requestType: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  requestTypeText: {
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
  requestInfo: {
    marginTop: 12,
  },
  requestLabel: {
    fontSize: 13,
    color: '#666',
  },
  requestAddress: {
    fontSize: 14,
    color: '#333',
    marginTop: 4,
  },
  requestTime: {
    fontSize: 12,
    color: '#999',
    marginTop: 8,
  },
  actionButtons: {
    flexDirection: 'row',
    marginTop: 16,
  },
  actionButton: {
    flex: 1,
    paddingVertical: 10,
    borderRadius: 8,
    alignItems: 'center',
  },
  rejectButton: {
    backgroundColor: '#f5f5f5',
    marginRight: 8,
  },
  rejectButtonText: {
    color: '#666',
    fontSize: 14,
    fontWeight: '500',
  },
  authorizeButton: {
    backgroundColor: THEME_COLOR,
    marginLeft: 8,
  },
  authorizeButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  viewReportButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#f8f4e8',
    paddingVertical: 10,
    borderRadius: 8,
    marginTop: 12,
  },
  viewReportText: {
    color: THEME_COLOR,
    fontSize: 14,
    fontWeight: '500',
    marginLeft: 6,
  },
});
