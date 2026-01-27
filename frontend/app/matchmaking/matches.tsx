/**
 * 匹配成功列表页面
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
  Image,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { matchmakingService, UserProfile, Gender } from '@/services/matchmaking.service';
import { useWalletStore } from '@/stores/wallet.store';

const THEME_COLOR = '#B2955D';

interface MatchItem {
  matchId: number;
  profile: UserProfile;
  matchedAt: number;
}

export default function MatchesPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [matches, setMatches] = useState<MatchItem[]>([]);

  const loadMatches = useCallback(async () => {
    if (!address) return;

    try {
      const matchAddresses = await matchmakingService.getUserMatches(address);
      const matchList: MatchItem[] = [];

      for (const matchAddress of matchAddresses) {
        const profile = await matchmakingService.getProfile(matchAddress);
        if (profile) {
          matchList.push({
            matchId: matchList.length,
            profile,
            matchedAt: profile.updatedAt, // 使用更新时间作为匹配时间的近似值
          });
        }
      }

      setMatches(matchList);
    } catch (error) {
      console.error('Load matches error:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [address]);

  useEffect(() => {
    loadMatches();
  }, [loadMatches]);

  const onRefresh = () => {
    setRefreshing(true);
    loadMatches();
  };

  const renderMatchItem = ({ item }: { item: MatchItem }) => (
    <Pressable
      style={styles.matchItem}
      onPress={() => router.push(`/matchmaking/chat/${item.profile.owner}` as any)}
    >
      <View style={styles.avatarContainer}>
        {item.profile.photoCids && item.profile.photoCids.length > 0 ? (
          <Image
            source={{ uri: `https://ipfs.io/ipfs/${item.profile.photoCids[0]}` }}
            style={styles.avatar}
          />
        ) : (
          <View style={styles.avatarPlaceholder}>
            <Ionicons name="person" size={32} color="#ccc" />
          </View>
        )}
      </View>

      <View style={styles.matchInfo}>
        <View style={styles.matchHeader}>
          <Text style={styles.matchName}>{item.profile.nickname}</Text>
          <Ionicons
            name={item.profile.gender === Gender.Male ? 'male' : 'female'}
            size={16}
            color={item.profile.gender === Gender.Male ? '#4A90D9' : '#FF6B9D'}
          />
        </View>
        <Text style={styles.matchDetail}>
          {new Date().getFullYear() - item.profile.birthYear}岁 · {item.profile.location}
        </Text>
        <Text style={styles.matchTime}>
          匹配于 {new Date(item.matchedAt * 1000).toLocaleDateString()}
        </Text>
      </View>

      <View style={styles.matchActions}>
        <Pressable
          style={styles.chatButton}
          onPress={() => router.push(`/matchmaking/chat/${item.profile.owner}` as any)}
        >
          <Ionicons name="chatbubble" size={20} color={THEME_COLOR} />
        </Pressable>
      </View>
    </Pressable>
  );

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="匹配成功" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="匹配成功" showBack />

      {matches.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Ionicons name="heart-outline" size={80} color="#ccc" />
          <Text style={styles.emptyTitle}>暂无匹配</Text>
          <Text style={styles.emptyText}>
            去发现页面寻找心仪的对象吧
          </Text>
          <Pressable
            style={styles.discoverButton}
            onPress={() => router.push('/matchmaking/discover' as any)}
          >
            <Text style={styles.discoverButtonText}>去发现</Text>
          </Pressable>
        </View>
      ) : (
        <FlatList
          data={matches}
          renderItem={renderMatchItem}
          keyExtractor={(item) => item.matchId.toString()}
          contentContainerStyle={styles.listContent}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }
        />
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
  },
  discoverButton: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 20,
    marginTop: 20,
  },
  discoverButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  listContent: {
    padding: 16,
  },
  matchItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 12,
    marginBottom: 12,
  },
  avatarContainer: {
    width: 60,
    height: 60,
    borderRadius: 30,
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
  matchInfo: {
    flex: 1,
    marginLeft: 12,
  },
  matchHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  matchName: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
    marginRight: 6,
  },
  matchDetail: {
    fontSize: 13,
    color: '#666',
    marginTop: 4,
  },
  matchTime: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  matchActions: {
    marginLeft: 8,
  },
  chatButton: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#f8f4e8',
    justifyContent: 'center',
    alignItems: 'center',
  },
});
