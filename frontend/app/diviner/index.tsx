/**
 * 成为占卜师 - 入口页面
 * 引导用户注册成为占卜师
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { TierBadge, TIER_CONFIG, ProviderTier } from '@/features/diviner';

const THEME_COLOR = '#B2955D';

export default function DivinerEntryPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(true);
  const [isProvider, setIsProvider] = useState(false);

  useEffect(() => {
    const checkProviderStatus = async () => {
      try {
        const { divinationMarketService } = await import('@/services/divination-market.service');
        const { useWalletStore } = await import('@/stores/wallet.store');
        const address = useWalletStore.getState().address;
        
        if (address) {
          // 检查当前用户是否已是占卜师
          const provider = await divinationMarketService.getProviderByAccount(address);
          setIsProvider(provider !== null && provider.status === 'Active');
        }
      } catch (error) {
        console.error('Check provider status error:', error);
        setIsProvider(false);
      } finally {
        setLoading(false);
      }
    };
    
    checkProviderStatus();
  }, []);

  const handleRegister = () => {
    router.push('/diviner/register' as any);
  };

  const handleGoDashboard = () => {
    router.push('/diviner/dashboard' as any);
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="成为占卜师" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  // 已是占卜师，跳转到仪表盘
  if (isProvider) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="占卜师中心" />
        <View style={styles.providerContainer}>
          <Text style={styles.providerTitle}>您已是占卜师</Text>
          <Pressable style={styles.dashboardBtn} onPress={handleGoDashboard}>
            <Text style={styles.dashboardBtnText}>进入仪表盘</Text>
          </Pressable>
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="成为占卜师" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 头部介绍 */}
        <View style={styles.heroSection}>
          <Text style={styles.heroEmoji}>🔮</Text>
          <Text style={styles.heroTitle}>成为星尘玄鉴占卜师</Text>
          <Text style={styles.heroSubtitle}>
            分享您的玄学智慧，帮助更多人解惑答疑
          </Text>
        </View>

        {/* 优势介绍 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>为什么加入我们？</Text>
          
          <View style={styles.benefitCard}>
            <Text style={styles.benefitIcon}>💰</Text>
            <View style={styles.benefitContent}>
              <Text style={styles.benefitTitle}>灵活收入</Text>
              <Text style={styles.benefitDesc}>自主定价，随时提现，收益透明</Text>
            </View>
          </View>

          <View style={styles.benefitCard}>
            <Text style={styles.benefitIcon}>🛡️</Text>
            <View style={styles.benefitContent}>
              <Text style={styles.benefitTitle}>安全保障</Text>
              <Text style={styles.benefitDesc}>链上交易，资金托管，仲裁保护</Text>
            </View>
          </View>

          <View style={styles.benefitCard}>
            <Text style={styles.benefitIcon}>📈</Text>
            <View style={styles.benefitContent}>
              <Text style={styles.benefitTitle}>等级成长</Text>
              <Text style={styles.benefitDesc}>完成订单提升等级，降低平台费率</Text>
            </View>
          </View>

          <View style={styles.benefitCard}>
            <Text style={styles.benefitIcon}>🌐</Text>
            <View style={styles.benefitContent}>
              <Text style={styles.benefitTitle}>多元服务</Text>
              <Text style={styles.benefitDesc}>支持文字、语音、视频多种服务形式</Text>
            </View>
          </View>
        </View>

        {/* 等级体系 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>等级体系</Text>
          <View style={styles.tierList}>
            {Object.entries(TIER_CONFIG).map(([tier, config]) => (
              <View key={tier} style={styles.tierItem}>
                <TierBadge tier={Number(tier) as ProviderTier} size="medium" />
                <Text style={styles.tierFee}>平台费 {config.feeRate}%</Text>
              </View>
            ))}
          </View>
          <Text style={styles.tierNote}>
            完成更多订单、获得更高评分，即可自动升级
          </Text>
        </View>

        {/* 注册要求 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>注册要求</Text>
          <View style={styles.requirementList}>
            <View style={styles.requirementItem}>
              <Text style={styles.checkIcon}>✓</Text>
              <Text style={styles.requirementText}>账户余额 ≥ 100 DUST（保证金）</Text>
            </View>
            <View style={styles.requirementItem}>
              <Text style={styles.checkIcon}>✓</Text>
              <Text style={styles.requirementText}>填写真实的个人简介</Text>
            </View>
            <View style={styles.requirementItem}>
              <Text style={styles.checkIcon}>✓</Text>
              <Text style={styles.requirementText}>选择至少一项擅长领域</Text>
            </View>
            <View style={styles.requirementItem}>
              <Text style={styles.checkIcon}>✓</Text>
              <Text style={styles.requirementText}>选择至少一种占卜类型</Text>
            </View>
          </View>
        </View>

        {/* 注册按钮 */}
        <View style={styles.actionSection}>
          <Pressable style={styles.registerBtn} onPress={handleRegister}>
            <Text style={styles.registerBtnText}>立即注册</Text>
          </Pressable>
          <Text style={styles.depositNote}>
            注册需锁定 100 DUST 保证金，注销时退还
          </Text>
        </View>
      </ScrollView>

      <BottomNavBar activeTab="profile" />
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  container: {
    flex: 1,
  },
  contentContainer: {
    paddingBottom: 100,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  providerContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  providerTitle: {
    fontSize: 18,
    color: '#333',
    marginBottom: 20,
  },
  dashboardBtn: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 32,
    paddingVertical: 14,
    borderRadius: 8,
  },
  dashboardBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
  heroSection: {
    alignItems: 'center',
    paddingVertical: 32,
    paddingHorizontal: 20,
    backgroundColor: '#FFF',
  },
  heroEmoji: {
    fontSize: 48,
    marginBottom: 16,
  },
  heroTitle: {
    fontSize: 22,
    fontWeight: '600',
    color: '#000',
    marginBottom: 8,
  },
  heroSubtitle: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
  },
  section: {
    padding: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000',
    marginBottom: 16,
  },
  benefitCard: {
    flexDirection: 'row',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    alignItems: 'center',
  },
  benefitIcon: {
    fontSize: 28,
    marginRight: 16,
  },
  benefitContent: {
    flex: 1,
  },
  benefitTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000',
    marginBottom: 4,
  },
  benefitDesc: {
    fontSize: 14,
    color: '#666',
  },
  tierList: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  tierItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 10,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  tierFee: {
    fontSize: 14,
    color: '#666',
  },
  tierNote: {
    fontSize: 12,
    color: '#999',
    textAlign: 'center',
    marginTop: 12,
  },
  requirementList: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  requirementItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 8,
  },
  checkIcon: {
    fontSize: 16,
    color: '#4CD964',
    marginRight: 12,
    fontWeight: '600',
  },
  requirementText: {
    fontSize: 14,
    color: '#333',
  },
  actionSection: {
    padding: 16,
    alignItems: 'center',
  },
  registerBtn: {
    width: '100%',
    height: 52,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 12,
  },
  registerBtnText: {
    fontSize: 18,
    color: '#FFF',
    fontWeight: '600',
  },
  depositNote: {
    fontSize: 12,
    color: '#999',
  },
});
