/**
 * IPFS 存储管理页面
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
import {
  ipfsStorageService,
  IpfsStorageService,
  PinnedContent,
  PinStatus,
  PinTier,
} from '@/services/ipfs-storage.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

export default function StorageIndexPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [pinnedContents, setPinnedContents] = useState<PinnedContent[]>([]);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingUnpin, setPendingUnpin] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    if (!address) return;

    try {
      const contents = await ipfsStorageService.getUserPinnedContents(address);
      setPinnedContents(contents);
    } catch (error) {
      console.error('Load storage data error:', error);
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

  const handleUnpin = async (cidHash: string) => {
    if (!isSignerUnlocked()) {
      setPendingUnpin(cidHash);
      setShowUnlockDialog(true);
      return;
    }

    await executeUnpin(cidHash);
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      if (pendingUnpin) {
        await executeUnpin(pendingUnpin);
        setPendingUnpin(null);
      }
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeUnpin = async (cidHash: string) => {
    setShowTxStatus(true);
    setTxStatus('正在取消固定...');

    try {
      await ipfsStorageService.requestUnpin(cidHash, (status) => setTxStatus(status));

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

  const getStatusText = (status: PinStatus) => {
    switch (status) {
      case PinStatus.Pending:
        return '等待固定';
      case PinStatus.Pinned:
        return '已固定';
      case PinStatus.Failed:
        return '固定失败';
      case PinStatus.Unpinned:
        return '已取消';
      default:
        return '未知';
    }
  };

  const getStatusColor = (status: PinStatus) => {
    switch (status) {
      case PinStatus.Pending:
        return '#FF9500';
      case PinStatus.Pinned:
        return '#4CD964';
      case PinStatus.Failed:
        return '#FF3B30';
      case PinStatus.Unpinned:
        return '#999';
      default:
        return '#999';
    }
  };

  const getTierText = (tier: PinTier) => {
    switch (tier) {
      case PinTier.Critical:
        return '关键';
      case PinTier.Standard:
        return '标准';
      case PinTier.Temporary:
        return '临时';
      default:
        return '未知';
    }
  };

  const renderContentItem = ({ item }: { item: PinnedContent }) => (
    <View style={styles.contentItem}>
      <View style={styles.contentHeader}>
        <View style={styles.contentType}>
          <Ionicons name="cloud" size={20} color={THEME_COLOR} />
          <Text style={styles.contentTypeText}>{item.subjectType}</Text>
        </View>
        <View style={[styles.statusBadge, { backgroundColor: getStatusColor(item.status) + '20' }]}>
          <Text style={[styles.statusText, { color: getStatusColor(item.status) }]}>
            {getStatusText(item.status)}
          </Text>
        </View>
      </View>

      <Text style={styles.cidText} numberOfLines={1}>
        CID: {item.cid || item.cidHash}
      </Text>

      <View style={styles.contentDetails}>
        <View style={styles.detailItem}>
          <Ionicons name="layers-outline" size={14} color="#666" />
          <Text style={styles.detailText}>{getTierText(item.tier)}</Text>
        </View>
        {item.size && (
          <View style={styles.detailItem}>
            <Ionicons name="document-outline" size={14} color="#666" />
            <Text style={styles.detailText}>{IpfsStorageService.formatSize(item.size)}</Text>
          </View>
        )}
        <View style={styles.detailItem}>
          <Ionicons name="time-outline" size={14} color="#666" />
          <Text style={styles.detailText}>
            {new Date(item.pinnedAt * 1000).toLocaleDateString()}
          </Text>
        </View>
      </View>

      {item.status === PinStatus.Pinned && (
        <Pressable
          style={styles.unpinButton}
          onPress={() => {
            Alert.alert(
              '确认取消固定',
              '取消固定后，内容可能会从 IPFS 网络中删除',
              [
                { text: '取消', style: 'cancel' },
                { text: '确认', onPress: () => handleUnpin(item.cidHash) },
              ]
            );
          }}
        >
          <Ionicons name="cloud-offline-outline" size={16} color="#FF6B6B" />
          <Text style={styles.unpinButtonText}>取消固定</Text>
        </Pressable>
      )}
    </View>
  );

  // 统计
  const totalPinned = pinnedContents.filter((c) => c.status === PinStatus.Pinned).length;
  const totalSize = pinnedContents.reduce((sum, c) => sum + (c.size || 0), 0);

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="存储管理" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="存储管理" showBack />

      {/* 统计卡片 */}
      <View style={styles.statsCard}>
        <View style={styles.statItem}>
          <Text style={styles.statNumber}>{totalPinned}</Text>
          <Text style={styles.statLabel}>已固定文件</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={styles.statNumber}>{IpfsStorageService.formatSize(totalSize)}</Text>
          <Text style={styles.statLabel}>总存储大小</Text>
        </View>
      </View>

      {/* 功能入口 */}
      <View style={styles.menuRow}>
        <Pressable
          style={styles.menuItem}
          onPress={() => router.push('/storage/fund' as any)}
        >
          <View style={[styles.menuIcon, { backgroundColor: '#E8F5E9' }]}>
            <Ionicons name="wallet" size={24} color="#4CAF50" />
          </View>
          <Text style={styles.menuText}>充值</Text>
        </Pressable>
        <Pressable
          style={styles.menuItem}
          onPress={() => router.push('/storage/billing' as any)}
        >
          <View style={[styles.menuIcon, { backgroundColor: '#FFF3E0' }]}>
            <Ionicons name="receipt" size={24} color="#FF9800" />
          </View>
          <Text style={styles.menuText}>计费</Text>
        </Pressable>
        <Pressable
          style={styles.menuItem}
          onPress={() => router.push('/storage/accounts' as any)}
        >
          <View style={[styles.menuIcon, { backgroundColor: '#E3F2FD' }]}>
            <Ionicons name="folder" size={24} color="#2196F3" />
          </View>
          <Text style={styles.menuText}>账户</Text>
        </Pressable>
      </View>

      {/* 已固定内容列表 */}
      <View style={styles.listHeader}>
        <Text style={styles.listTitle}>已固定内容</Text>
      </View>

      {pinnedContents.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Ionicons name="cloud-outline" size={60} color="#ccc" />
          <Text style={styles.emptyText}>暂无固定内容</Text>
        </View>
      ) : (
        <FlatList
          data={pinnedContents}
          renderItem={renderContentItem}
          keyExtractor={(item) => item.cidHash}
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
  statsCard: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    margin: 16,
    marginBottom: 8,
    borderRadius: 12,
    padding: 20,
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statDivider: {
    width: 1,
    backgroundColor: '#e0e0e0',
  },
  statNumber: {
    fontSize: 24,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  statLabel: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
  },
  menuRow: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    marginBottom: 8,
  },
  menuItem: {
    flex: 1,
    alignItems: 'center',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginHorizontal: 4,
  },
  menuIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    justifyContent: 'center',
    alignItems: 'center',
  },
  menuText: {
    fontSize: 13,
    color: '#333',
    marginTop: 8,
  },
  listHeader: {
    paddingHorizontal: 16,
    paddingVertical: 12,
  },
  listTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
  },
  listContent: {
    paddingHorizontal: 16,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    fontSize: 14,
    color: '#999',
    marginTop: 12,
  },
  contentItem: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  contentHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  contentType: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  contentTypeText: {
    fontSize: 15,
    fontWeight: '500',
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
  cidText: {
    fontSize: 12,
    color: '#666',
    marginTop: 8,
    fontFamily: 'monospace',
  },
  contentDetails: {
    flexDirection: 'row',
    marginTop: 12,
  },
  detailItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginRight: 16,
  },
  detailText: {
    fontSize: 12,
    color: '#666',
    marginLeft: 4,
  },
  unpinButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#FFF5F5',
    borderRadius: 8,
    paddingVertical: 8,
    marginTop: 12,
  },
  unpinButtonText: {
    fontSize: 13,
    color: '#FF6B6B',
    marginLeft: 4,
  },
});
