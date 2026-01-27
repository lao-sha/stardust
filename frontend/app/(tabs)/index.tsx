/**
 * 星尘玄鉴 - 首页
 * 展示钱包概览、快捷功能入口、最近动态
 * 主题色：金棕色 #B2955D
 */

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  RefreshControl,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { useWalletStore } from '@/stores/wallet.store';
import { useChatStore } from '@/stores/chat.store';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

// 快捷功能配置
const QUICK_ACTIONS = [
  {
    id: 'divination',
    name: '占卜',
    icon: 'compass-outline' as const,
    route: '/(tabs)/divination',
    color: '#E74C3C',
  },
  {
    id: 'market',
    name: '市场',
    icon: 'storefront-outline' as const,
    route: '/(tabs)/market',
    color: '#3498DB',
  },
  {
    id: 'calendar',
    name: '万年历',
    icon: 'calendar-outline' as const,
    route: '/calendar',
    color: '#9B59B6',
  },
  {
    id: 'bridge',
    name: '跨链桥',
    icon: 'swap-horizontal-outline' as const,
    route: '/bridge',
    color: '#1ABC9C',
  },
];

// 功能入口配置
const FEATURE_ENTRIES = [
  {
    id: 'bazi',
    name: '八字排盘',
    desc: '四柱推命，命运格局',
    icon: 'calendar' as const,
    route: '/divination/bazi',
    color: '#E74C3C',
  },
  {
    id: 'ziwei',
    name: '紫微斗数',
    desc: '十四主星，人生领域',
    icon: 'star' as const,
    route: '/divination/ziwei',
    color: '#9B59B6',
  },
  {
    id: 'qimen',
    name: '奇门遁甲',
    desc: '帝王之术，预测决策',
    icon: 'grid' as const,
    route: '/divination/qimen',
    color: '#3498DB',
  },
  {
    id: 'liuyao',
    name: '六爻占卜',
    desc: '纳甲筮法，断事精准',
    icon: 'layers' as const,
    route: '/divination/liuyao',
    color: '#F39C12',
  },
];

export default function HomePage() {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const { address, hasWallet, isLocked, initialize } = useWalletStore();
  const { totalUnread } = useChatStore();
  
  const [refreshing, setRefreshing] = useState(false);
  const [greeting, setGreeting] = useState('');

  // 设置问候语
  useEffect(() => {
    const hour = new Date().getHours();
    if (hour < 6) setGreeting('夜深了');
    else if (hour < 9) setGreeting('早上好');
    else if (hour < 12) setGreeting('上午好');
    else if (hour < 14) setGreeting('中午好');
    else if (hour < 18) setGreeting('下午好');
    else if (hour < 22) setGreeting('晚上好');
    else setGreeting('夜深了');
  }, []);

  // 下拉刷新
  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    try {
      await initialize();
    } finally {
      setRefreshing(false);
    }
  }, [initialize]);

  // 格式化地址显示
  const formatAddress = (addr: string | null) => {
    if (!addr) return '未连接';
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  };

  // 处理钱包点击
  const handleWalletPress = () => {
    if (!hasWallet) {
      router.push('/auth/create');
    } else if (isLocked) {
      router.push('/auth/unlock');
    } else {
      router.push('/wallet/manage');
    }
  };

  return (
    <View style={[styles.container, { paddingTop: insets.top }]}>
      {/* 顶部区域 */}
      <View style={styles.header}>
        <View style={styles.headerLeft}>
          <Text style={styles.greeting}>{greeting}</Text>
          <Text style={styles.appName}>星尘玄鉴</Text>
        </View>
        <View style={styles.headerRight}>
          <Pressable
            style={styles.headerIcon}
            onPress={() => router.push('/(tabs)/chat')}
          >
            <Ionicons name="chatbubble-outline" size={24} color="#333" />
            {totalUnread > 0 && (
              <View style={styles.badge}>
                <Text style={styles.badgeText}>
                  {totalUnread > 99 ? '99+' : totalUnread}
                </Text>
              </View>
            )}
          </Pressable>
          <Pressable
            style={styles.headerIcon}
            onPress={() => router.push('/checkin')}
          >
            <Ionicons name="gift-outline" size={24} color="#333" />
          </Pressable>
        </View>
      </View>

      <ScrollView
        style={styles.scrollView}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={onRefresh}
            tintColor={THEME_COLOR}
          />
        }
      >
        {/* 钱包卡片 */}
        <Pressable style={styles.walletCard} onPress={handleWalletPress}>
          <View style={styles.walletHeader}>
            <View style={styles.walletInfo}>
              <View style={styles.walletIcon}>
                <Ionicons name="wallet-outline" size={24} color="#FFF" />
              </View>
              <View>
                <Text style={styles.walletLabel}>我的钱包</Text>
                <Text style={styles.walletAddress}>
                  {hasWallet ? formatAddress(address) : '点击创建钱包'}
                </Text>
              </View>
            </View>
            <Ionicons name="chevron-forward" size={20} color="rgba(255,255,255,0.7)" />
          </View>
          {hasWallet && !isLocked && (
            <View style={styles.walletActions}>
              <Pressable
                style={styles.walletAction}
                onPress={() => router.push('/wallet/transfer')}
              >
                <Ionicons name="arrow-up-outline" size={20} color="#FFF" />
                <Text style={styles.walletActionText}>转账</Text>
              </Pressable>
              <View style={styles.walletDivider} />
              <Pressable
                style={styles.walletAction}
                onPress={() => router.push('/wallet/buy-dust' as any)}
              >
                <Ionicons name="add-circle-outline" size={20} color="#FFF" />
                <Text style={styles.walletActionText}>购买</Text>
              </Pressable>
              <View style={styles.walletDivider} />
              <Pressable
                style={styles.walletAction}
                onPress={() => router.push('/wallet/transactions')}
              >
                <Ionicons name="list-outline" size={20} color="#FFF" />
                <Text style={styles.walletActionText}>记录</Text>
              </Pressable>
            </View>
          )}
        </Pressable>

        {/* 快捷功能 */}
        <View style={styles.quickSection}>
          <View style={styles.quickGrid}>
            {QUICK_ACTIONS.map((action) => (
              <Pressable
                key={action.id}
                style={styles.quickItem}
                onPress={() => router.push(action.route as any)}
              >
                <View style={[styles.quickIcon, { backgroundColor: action.color + '15' }]}>
                  <Ionicons name={action.icon} size={24} color={action.color} />
                </View>
                <Text style={styles.quickName}>{action.name}</Text>
              </Pressable>
            ))}
          </View>
        </View>

        {/* 功能入口 */}
        <View style={styles.featureSection}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>热门占卜</Text>
            <Pressable onPress={() => router.push('/(tabs)/divination')}>
              <Text style={styles.sectionMore}>查看全部</Text>
            </Pressable>
          </View>
          <View style={styles.featureGrid}>
            {FEATURE_ENTRIES.map((feature) => (
              <Pressable
                key={feature.id}
                style={styles.featureCard}
                onPress={() => router.push(feature.route as any)}
              >
                <View style={[styles.featureIcon, { backgroundColor: feature.color + '15' }]}>
                  <Ionicons name={feature.icon} size={22} color={feature.color} />
                </View>
                <View style={styles.featureInfo}>
                  <Text style={styles.featureName}>{feature.name}</Text>
                  <Text style={styles.featureDesc}>{feature.desc}</Text>
                </View>
                <Ionicons name="chevron-forward" size={18} color="#CCC" />
              </Pressable>
            ))}
          </View>
        </View>

        {/* 服务入口 */}
        <View style={styles.serviceSection}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>更多服务</Text>
          </View>
          <View style={styles.serviceGrid}>
            <Pressable
              style={styles.serviceCard}
              onPress={() => router.push('/matchmaking' as any)}
            >
              <Ionicons name="heart-outline" size={28} color="#E91E63" />
              <Text style={styles.serviceName}>婚恋匹配</Text>
              <Text style={styles.serviceDesc}>命理合婚</Text>
            </Pressable>
            <Pressable
              style={styles.serviceCard}
              onPress={() => router.push('/diviner' as any)}
            >
              <Ionicons name="person-outline" size={28} color="#673AB7" />
              <Text style={styles.serviceName}>成为大师</Text>
              <Text style={styles.serviceDesc}>入驻平台</Text>
            </Pressable>
            <Pressable
              style={styles.serviceCard}
              onPress={() => router.push('/maker' as any)}
            >
              <Ionicons name="cash-outline" size={28} color="#00BCD4" />
              <Text style={styles.serviceName}>做市商</Text>
              <Text style={styles.serviceDesc}>赚取收益</Text>
            </Pressable>
          </View>
        </View>

        {/* 底部说明 */}
        <View style={styles.footer}>
          <Text style={styles.footerText}>传统术数 · 链上存证 · 隐私加密</Text>
          <Text style={styles.footerVersion}>星尘玄鉴 v1.0.0</Text>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingVertical: 16,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  headerLeft: {},
  greeting: {
    fontSize: 14,
    color: '#999',
  },
  appName: {
    fontSize: 24,
    fontWeight: '600',
    color: '#333',
    marginTop: 2,
  },
  headerRight: {
    flexDirection: 'row',
    gap: 16,
  },
  headerIcon: {
    position: 'relative',
    padding: 4,
  },
  badge: {
    position: 'absolute',
    top: 0,
    right: 0,
    backgroundColor: '#FF3B30',
    borderRadius: 10,
    minWidth: 18,
    height: 18,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 4,
  },
  badgeText: {
    color: '#FFF',
    fontSize: 10,
    fontWeight: '600',
  },
  scrollView: {
    flex: 1,
  },
  scrollContent: {
    padding: 16,
    paddingBottom: 100,
  },
  // 钱包卡片
  walletCard: {
    backgroundColor: THEME_COLOR,
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
    shadowColor: THEME_COLOR,
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.3,
    shadowRadius: 8,
    elevation: 4,
  },
  walletHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  walletInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  walletIcon: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: 'rgba(255,255,255,0.2)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  walletLabel: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
  },
  walletAddress: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFF',
    marginTop: 2,
  },
  walletActions: {
    flexDirection: 'row',
    marginTop: 20,
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: 'rgba(255,255,255,0.2)',
  },
  walletAction: {
    flex: 1,
    alignItems: 'center',
    gap: 6,
  },
  walletActionText: {
    fontSize: 13,
    color: 'rgba(255,255,255,0.9)',
  },
  walletDivider: {
    width: 1,
    height: 36,
    backgroundColor: 'rgba(255,255,255,0.2)',
  },
  // 快捷功能
  quickSection: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  quickGrid: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  quickItem: {
    alignItems: 'center',
    gap: 8,
  },
  quickIcon: {
    width: 52,
    height: 52,
    borderRadius: 14,
    justifyContent: 'center',
    alignItems: 'center',
  },
  quickName: {
    fontSize: 13,
    color: '#333',
    fontWeight: '500',
  },
  // 功能入口
  featureSection: {
    marginBottom: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
    paddingHorizontal: 4,
  },
  sectionTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  sectionMore: {
    fontSize: 14,
    color: THEME_COLOR,
  },
  featureGrid: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    overflow: 'hidden',
  },
  featureCard: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#F5F5F5',
  },
  featureIcon: {
    width: 44,
    height: 44,
    borderRadius: 12,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  featureInfo: {
    flex: 1,
  },
  featureName: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
    marginBottom: 2,
  },
  featureDesc: {
    fontSize: 13,
    color: '#999',
  },
  // 服务入口
  serviceSection: {
    marginBottom: 16,
  },
  serviceGrid: {
    flexDirection: 'row',
    gap: 12,
  },
  serviceCard: {
    flex: 1,
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
    gap: 8,
  },
  serviceName: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
  },
  serviceDesc: {
    fontSize: 12,
    color: '#999',
  },
  // 底部
  footer: {
    alignItems: 'center',
    paddingVertical: 24,
  },
  footerText: {
    fontSize: 13,
    color: '#BBB',
    marginBottom: 4,
  },
  footerVersion: {
    fontSize: 12,
    color: '#CCC',
  },
});
