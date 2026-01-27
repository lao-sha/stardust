/**
 * 婚恋模块首页
 * 显示用户资料状态和主要功能入口
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  Alert,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { matchmakingService, UserProfile, MatchmakingPrivacyMode } from '@/services/matchmaking.service';
import { useWalletStore } from '@/stores/wallet.store';

const THEME_COLOR = '#B2955D';

export default function MatchmakingIndexPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [matchCount, setMatchCount] = useState(0);

  const loadData = useCallback(async () => {
    if (!address) return;

    try {
      const userProfile = await matchmakingService.getProfile(address);
      setProfile(userProfile);

      if (userProfile) {
        const matches = await matchmakingService.getUserMatches(address);
        setMatchCount(matches.length);
      }
    } catch (error) {
      console.error('Load matchmaking data error:', error);
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

  const getPrivacyModeText = (mode: MatchmakingPrivacyMode) => {
    switch (mode) {
      case MatchmakingPrivacyMode.Public:
        return '公开';
      case MatchmakingPrivacyMode.MembersOnly:
        return '仅会员可见';
      case MatchmakingPrivacyMode.MatchedOnly:
        return '仅匹配后可见';
      default:
        return '未知';
    }
  };

  const isMembershipActive = profile?.membershipExpiry
    ? profile.membershipExpiry > Date.now() / 1000
    : false;

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="缘分天成" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="缘分天成" showBack />

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
      >
        {!profile ? (
          // 未创建资料
          <View style={styles.emptyContainer}>
            <Ionicons name="heart-outline" size={80} color="#ccc" />
            <Text style={styles.emptyTitle}>开启您的缘分之旅</Text>
            <Text style={styles.emptyText}>
              创建个人资料，寻找命中注定的另一半
            </Text>
            <Pressable
              style={styles.createButton}
              onPress={() => router.push('/matchmaking/create-profile' as any)}
            >
              <Text style={styles.createButtonText}>创建资料</Text>
            </Pressable>
          </View>
        ) : (
          // 已创建资料
          <>
            {/* 会员状态卡片 */}
            <View style={[styles.card, isMembershipActive ? styles.memberCard : styles.expiredCard]}>
              <View style={styles.cardHeader}>
                <Ionicons
                  name={isMembershipActive ? 'diamond' : 'diamond-outline'}
                  size={24}
                  color={isMembershipActive ? THEME_COLOR : '#999'}
                />
                <Text style={[styles.cardTitle, !isMembershipActive && styles.expiredText]}>
                  {isMembershipActive ? '会员有效' : '会员已过期'}
                </Text>
              </View>
              {isMembershipActive ? (
                <Text style={styles.memberExpiry}>
                  有效期至: {new Date(profile.membershipExpiry! * 1000).toLocaleDateString()}
                </Text>
              ) : (
                <Pressable
                  style={styles.renewButton}
                  onPress={() => router.push('/matchmaking/membership' as any)}
                >
                  <Text style={styles.renewButtonText}>续费会员</Text>
                </Pressable>
              )}
            </View>

            {/* 统计卡片 */}
            <View style={styles.statsRow}>
              <Pressable
                style={styles.statCard}
                onPress={() => router.push('/matchmaking/matches' as any)}
              >
                <Text style={styles.statNumber}>{matchCount}</Text>
                <Text style={styles.statLabel}>匹配成功</Text>
              </Pressable>
              <Pressable
                style={styles.statCard}
                onPress={() => router.push('/matchmaking/likes' as any)}
              >
                <Ionicons name="heart" size={24} color="#FF6B6B" />
                <Text style={styles.statLabel}>喜欢我的</Text>
              </Pressable>
              <Pressable
                style={styles.statCard}
                onPress={() => router.push('/matchmaking/requests' as any)}
              >
                <Ionicons name="git-merge" size={24} color={THEME_COLOR} />
                <Text style={styles.statLabel}>合婚请求</Text>
              </Pressable>
            </View>

            {/* 功能入口 */}
            <View style={styles.menuSection}>
              <Text style={styles.sectionTitle}>功能</Text>

              <Pressable
                style={styles.menuItem}
                onPress={() => router.push('/matchmaking/discover' as any)}
              >
                <View style={styles.menuIcon}>
                  <Ionicons name="compass" size={24} color={THEME_COLOR} />
                </View>
                <View style={styles.menuContent}>
                  <Text style={styles.menuTitle}>发现</Text>
                  <Text style={styles.menuDesc}>浏览推荐的优质对象</Text>
                </View>
                <Ionicons name="chevron-forward" size={20} color="#999" />
              </Pressable>

              <Pressable
                style={styles.menuItem}
                onPress={() => router.push('/matchmaking/profile' as any)}
              >
                <View style={styles.menuIcon}>
                  <Ionicons name="person" size={24} color={THEME_COLOR} />
                </View>
                <View style={styles.menuContent}>
                  <Text style={styles.menuTitle}>我的资料</Text>
                  <Text style={styles.menuDesc}>编辑个人信息和照片</Text>
                </View>
                <Ionicons name="chevron-forward" size={20} color="#999" />
              </Pressable>

              <Pressable
                style={styles.menuItem}
                onPress={() => router.push('/matchmaking/preferences' as any)}
              >
                <View style={styles.menuIcon}>
                  <Ionicons name="options" size={24} color={THEME_COLOR} />
                </View>
                <View style={styles.menuContent}>
                  <Text style={styles.menuTitle}>择偶条件</Text>
                  <Text style={styles.menuDesc}>设置理想对象的条件</Text>
                </View>
                <Ionicons name="chevron-forward" size={20} color="#999" />
              </Pressable>

              <Pressable
                style={styles.menuItem}
                onPress={() => router.push('/matchmaking/bazi-matching' as any)}
              >
                <View style={styles.menuIcon}>
                  <Ionicons name="git-compare" size={24} color={THEME_COLOR} />
                </View>
                <View style={styles.menuContent}>
                  <Text style={styles.menuTitle}>八字合婚</Text>
                  <Text style={styles.menuDesc}>基于八字的深度匹配分析</Text>
                </View>
                <Ionicons name="chevron-forward" size={20} color="#999" />
              </Pressable>

              <Pressable
                style={styles.menuItem}
                onPress={() => router.push('/matchmaking/settings' as any)}
              >
                <View style={styles.menuIcon}>
                  <Ionicons name="settings" size={24} color={THEME_COLOR} />
                </View>
                <View style={styles.menuContent}>
                  <Text style={styles.menuTitle}>隐私设置</Text>
                  <Text style={styles.menuDesc}>当前: {getPrivacyModeText(profile.privacyMode)}</Text>
                </View>
                <Ionicons name="chevron-forward" size={20} color="#999" />
              </Pressable>
            </View>
          </>
        )}
      </ScrollView>

      <BottomNavBar />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 12,
    color: '#666',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 80,
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
    paddingHorizontal: 32,
    paddingVertical: 14,
    borderRadius: 25,
    marginTop: 24,
  },
  createButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: 'bold',
  },
  card: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  memberCard: {
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  expiredCard: {
    borderWidth: 1,
    borderColor: '#ddd',
  },
  cardHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    marginLeft: 8,
    color: THEME_COLOR,
  },
  expiredText: {
    color: '#999',
  },
  memberExpiry: {
    fontSize: 14,
    color: '#666',
    marginTop: 8,
  },
  renewButton: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
    marginTop: 12,
    alignSelf: 'flex-start',
  },
  renewButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  statsRow: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  statCard: {
    flex: 1,
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
    marginHorizontal: 4,
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
  menuSection: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
    marginBottom: 12,
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  menuIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#f8f4e8',
    justifyContent: 'center',
    alignItems: 'center',
  },
  menuContent: {
    flex: 1,
    marginLeft: 12,
  },
  menuTitle: {
    fontSize: 15,
    fontWeight: '500',
    color: '#333',
  },
  menuDesc: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
});
