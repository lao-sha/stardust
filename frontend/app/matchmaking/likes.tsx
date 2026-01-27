/**
 * 喜欢我的 / 超级喜欢页面
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
import { matchmakingService } from '@/services/matchmaking.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

interface SuperLikeItem {
  senderHash: string;
  timestamp: number;
  viewed: boolean;
}

export default function LikesPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [superLikes, setSuperLikes] = useState<SuperLikeItem[]>([]);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingSender, setPendingSender] = useState<string | null>(null);

  const loadSuperLikes = useCallback(async () => {
    if (!address) return;

    try {
      const likes = await matchmakingService.getSuperLikesReceived(address);
      setSuperLikes(likes);
    } catch (error) {
      console.error('Load super likes error:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [address]);

  useEffect(() => {
    loadSuperLikes();
  }, [loadSuperLikes]);

  const onRefresh = () => {
    setRefreshing(true);
    loadSuperLikes();
  };

  const handleMarkViewed = async (senderHash: string) => {
    if (!isSignerUnlocked()) {
      setPendingSender(senderHash);
      setShowUnlockDialog(true);
      return;
    }

    await executeMarkViewed(senderHash);
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      if (pendingSender) {
        await executeMarkViewed(pendingSender);
        setPendingSender(null);
      }
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeMarkViewed = async (senderHash: string) => {
    setShowTxStatus(true);
    setTxStatus('正在标记已查看...');

    try {
      await matchmakingService.markSuperLikeViewed(senderHash, (status) => setTxStatus(status));

      setTxStatus('操作成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        loadSuperLikes();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('操作失败', error.message || '请稍后重试');
    }
  };

  const unviewedCount = superLikes.filter(sl => !sl.viewed).length;

  const renderSuperLikeItem = ({ item }: { item: SuperLikeItem }) => (
    <View style={styles.likeItem}>
      <View style={styles.likeIcon}>
        <Ionicons name="star" size={24} color="#FFD700" />
      </View>

      <View style={styles.likeInfo}>
        <View style={styles.likeHeader}>
          <Text style={styles.likeTitle}>收到超级喜欢</Text>
          {!item.viewed && (
            <View style={styles.newBadge}>
              <Text style={styles.newBadgeText}>新</Text>
            </View>
          )}
        </View>
        <Text style={styles.likeHash}>
          来自: {item.senderHash.slice(0, 10)}...{item.senderHash.slice(-8)}
        </Text>
        <Text style={styles.likeTime}>
          {new Date(item.timestamp * 1000).toLocaleString()}
        </Text>
      </View>

      {!item.viewed && (
        <Pressable
          style={styles.viewButton}
          onPress={() => handleMarkViewed(item.senderHash)}
        >
          <Text style={styles.viewButtonText}>查看</Text>
        </Pressable>
      )}
    </View>
  );

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="喜欢我的" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="喜欢我的" showBack />

      {/* 统计信息 */}
      <View style={styles.statsContainer}>
        <View style={styles.statItem}>
          <Text style={styles.statNumber}>{superLikes.length}</Text>
          <Text style={styles.statLabel}>超级喜欢</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={[styles.statNumber, { color: '#FF6B6B' }]}>{unviewedCount}</Text>
          <Text style={styles.statLabel}>未查看</Text>
        </View>
      </View>

      {superLikes.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Ionicons name="star-outline" size={80} color="#ccc" />
          <Text style={styles.emptyTitle}>暂无超级喜欢</Text>
          <Text style={styles.emptyText}>
            完善您的资料，吸引更多关注
          </Text>
        </View>
      ) : (
        <FlatList
          data={superLikes}
          renderItem={renderSuperLikeItem}
          keyExtractor={(item, index) => `${item.senderHash}-${index}`}
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
  statsContainer: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    padding: 20,
    marginHorizontal: 16,
    marginTop: 16,
    borderRadius: 12,
    justifyContent: 'center',
    alignItems: 'center',
  },
  statItem: {
    alignItems: 'center',
    paddingHorizontal: 30,
  },
  statNumber: {
    fontSize: 28,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  statLabel: {
    fontSize: 13,
    color: '#666',
    marginTop: 4,
  },
  statDivider: {
    width: 1,
    height: 40,
    backgroundColor: '#e0e0e0',
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
  likeItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  likeIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: '#FFF8E1',
    justifyContent: 'center',
    alignItems: 'center',
  },
  likeInfo: {
    flex: 1,
    marginLeft: 12,
  },
  likeHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  likeTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
  },
  newBadge: {
    backgroundColor: '#FF6B6B',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 8,
    marginLeft: 8,
  },
  newBadgeText: {
    fontSize: 10,
    color: '#fff',
    fontWeight: 'bold',
  },
  likeHash: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
  },
  likeTime: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  viewButton: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
  },
  viewButtonText: {
    color: '#fff',
    fontSize: 13,
    fontWeight: '500',
  },
});
