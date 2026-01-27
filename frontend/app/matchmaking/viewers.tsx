/**
 * 谁看过我页面
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
  Image,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { matchmakingService, UserProfile, Gender } from '@/services/matchmaking.service';
import { useWalletStore } from '@/stores/wallet.store';

const THEME_COLOR = '#B2955D';

interface ViewerItem {
  viewer: string;
  viewedAt: number;
  profile?: UserProfile;
}

export default function ViewersPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [viewers, setViewers] = useState<ViewerItem[]>([]);
  const [quota, setQuota] = useState({ likes: 0, superLikes: 0, views: 0 });

  const loadViewers = useCallback(async () => {
    if (!address) return;

    try {
      // 获取查看过我的用户列表
      const viewerList = await matchmakingService.getProfileViewers(address);
      
      // 获取每个查看者的资料
      const viewersWithProfile: ViewerItem[] = [];
      for (const item of viewerList) {
        const profile = await matchmakingService.getProfile(item.viewer);
        viewersWithProfile.push({
          viewer: item.viewer,
          viewedAt: item.viewedAt,
          profile: profile || undefined,
        });
      }

      // 按时间倒序排列
      viewersWithProfile.sort((a, b) => b.viewedAt - a.viewedAt);
      setViewers(viewersWithProfile);

      // 获取剩余配额
      const remainingQuota = await matchmakingService.getRemainingQuota(address);
      setQuota(remainingQuota);
    } catch (error) {
      console.error('Load viewers error:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [address]);

  useEffect(() => {
    loadViewers();
  }, [loadViewers]);

  const onRefresh = () => {
    setRefreshing(true);
    loadViewers();
  };

  const renderViewerItem = ({ item }: { item: ViewerItem }) => (
    <Pressable
      style={styles.viewerItem}
      onPress={() => {
        if (item.profile) {
          router.push(`/matchmaking/profile/${item.viewer}` as any);
        }
      }}
    >
      <View style={styles.avatarContainer}>
        {item.profile?.photoCids && item.profile.photoCids.length > 0 ? (
          <Image
            source={{ uri: `https://ipfs.io/ipfs/${item.profile.photoCids[0]}` }}
            style={styles.avatar}
          />
        ) : (
          <View style={styles.avatarPlaceholder}>
            <Ionicons name="person" size={24} color="#ccc" />
          </View>
        )}
      </View>

      <View style={styles.viewerInfo}>
        {item.profile ? (
          <>
            <View style={styles.viewerHeader}>
              <Text style={styles.viewerName}>{item.profile.nickname || '匿名用户'}</Text>
              <Ionicons
                name={item.profile.gender === Gender.Male ? 'male' : 'female'}
                size={14}
                color={item.profile.gender === Gender.Male ? '#4A90D9' : '#FF6B9D'}
              />
            </View>
            <Text style={styles.viewerDetail}>
              {new Date().getFullYear() - item.profile.birthYear}岁 · {item.profile.location}
            </Text>
          </>
        ) : (
          <Text style={styles.viewerAddress}>
            {item.viewer.slice(0, 8)}...{item.viewer.slice(-6)}
          </Text>
        )}
        <Text style={styles.viewTime}>
          {formatTimeAgo(item.viewedAt)}
        </Text>
      </View>

      <Ionicons name="chevron-forward" size={20} color="#ccc" />
    </Pressable>
  );

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="谁看过我" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="谁看过我" showBack />

      {/* 配额显示 */}
      <View style={styles.quotaContainer}>
        <View style={styles.quotaItem}>
          <Ionicons name="heart" size={20} color="#FF6B6B" />
          <Text style={styles.quotaNumber}>{quota.likes}</Text>
          <Text style={styles.quotaLabel}>剩余点赞</Text>
        </View>
        <View style={styles.quotaDivider} />
        <View style={styles.quotaItem}>
          <Ionicons name="star" size={20} color="#FFD700" />
          <Text style={styles.quotaNumber}>{quota.superLikes}</Text>
          <Text style={styles.quotaLabel}>超级喜欢</Text>
        </View>
        <View style={styles.quotaDivider} />
        <View style={styles.quotaItem}>
          <Ionicons name="eye" size={20} color={THEME_COLOR} />
          <Text style={styles.quotaNumber}>{quota.views === 4294967295 ? '∞' : quota.views}</Text>
          <Text style={styles.quotaLabel}>剩余查看</Text>
        </View>
      </View>

      {viewers.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Ionicons name="eye-outline" size={80} color="#ccc" />
          <Text style={styles.emptyTitle}>暂无访客</Text>
          <Text style={styles.emptyText}>
            完善您的资料，吸引更多关注
          </Text>
        </View>
      ) : (
        <FlatList
          data={viewers}
          renderItem={renderViewerItem}
          keyExtractor={(item, index) => `${item.viewer}-${index}`}
          contentContainerStyle={styles.listContent}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }
          ListHeaderComponent={
            <Text style={styles.listHeader}>最近 {viewers.length} 位访客</Text>
          }
        />
      )}

      <BottomNavBar />
    </View>
  );
}

function formatTimeAgo(timestamp: number): string {
  const now = Date.now() / 1000;
  const diff = now - timestamp;

  if (diff < 60) {
    return '刚刚';
  } else if (diff < 3600) {
    return `${Math.floor(diff / 60)} 分钟前`;
  } else if (diff < 86400) {
    return `${Math.floor(diff / 3600)} 小时前`;
  } else if (diff < 604800) {
    return `${Math.floor(diff / 86400)} 天前`;
  } else {
    return new Date(timestamp * 1000).toLocaleDateString();
  }
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
  quotaContainer: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    padding: 16,
    marginHorizontal: 16,
    marginTop: 16,
    borderRadius: 12,
    justifyContent: 'space-around',
    alignItems: 'center',
  },
  quotaItem: {
    alignItems: 'center',
  },
  quotaNumber: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#333',
    marginTop: 4,
  },
  quotaLabel: {
    fontSize: 11,
    color: '#999',
    marginTop: 2,
  },
  quotaDivider: {
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
  listHeader: {
    fontSize: 13,
    color: '#999',
    marginBottom: 12,
  },
  viewerItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 12,
    marginBottom: 10,
  },
  avatarContainer: {
    width: 50,
    height: 50,
    borderRadius: 25,
    overflow: 'hidden',
  },
  avatar: {
    width: '100%',
    height: '100%',
  },
  avatarPlaceholder: {
    width: '100%',
    height: '100%',
    backgroundColor: '#f0f0f0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  viewerInfo: {
    flex: 1,
    marginLeft: 12,
  },
  viewerHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  viewerName: {
    fontSize: 15,
    fontWeight: '600',
    color: '#333',
    marginRight: 6,
  },
  viewerDetail: {
    fontSize: 13,
    color: '#666',
    marginTop: 2,
  },
  viewerAddress: {
    fontSize: 14,
    color: '#333',
  },
  viewTime: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
});
