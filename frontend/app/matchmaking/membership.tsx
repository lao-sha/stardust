/**
 * 会员续费页面
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
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { matchmakingService, UserProfile } from '@/services/matchmaking.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

interface PlanOption {
  months: number;
  price: number;
  originalPrice?: number;
  label: string;
  popular?: boolean;
}

const PLANS: PlanOption[] = [
  { months: 1, price: 2, label: '1个月' },
  { months: 3, price: 5, originalPrice: 6, label: '3个月', popular: true },
  { months: 6, price: 9, originalPrice: 12, label: '6个月' },
  { months: 12, price: 15, originalPrice: 24, label: '12个月' },
];

const MEMBER_BENEFITS = [
  { icon: 'eye', title: '无限查看', desc: '每日查看资料无限制' },
  { icon: 'chatbubbles', title: '更多聊天', desc: '每日发起聊天数量提升' },
  { icon: 'star', title: '超级喜欢', desc: '每日更多超级喜欢次数' },
  { icon: 'sparkles', title: '优先推荐', desc: '资料优先展示给其他用户' },
  { icon: 'shield-checkmark', title: '专属标识', desc: '会员专属身份标识' },
  { icon: 'git-compare', title: '八字合婚', desc: '免费使用八字合婚功能' },
];

export default function MembershipPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [selectedPlan, setSelectedPlan] = useState<PlanOption>(PLANS[1]);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');

  const loadProfile = useCallback(async () => {
    if (!address) return;

    try {
      const userProfile = await matchmakingService.getProfile(address);
      setProfile(userProfile);
    } catch (error) {
      console.error('Load profile error:', error);
    } finally {
      setLoading(false);
    }
  }, [address]);

  useEffect(() => {
    loadProfile();
  }, [loadProfile]);

  const handlePurchase = async () => {
    if (!isSignerUnlocked()) {
      setShowUnlockDialog(true);
      return;
    }

    await executePurchase();
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      await executePurchase();
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executePurchase = async () => {
    setShowTxStatus(true);
    setTxStatus('正在处理支付...');

    try {
      await matchmakingService.payMonthlyFee(
        selectedPlan.months,
        undefined,
        (status) => setTxStatus(status)
      );

      setTxStatus('支付成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        loadProfile();
        Alert.alert('恭喜', `您已成功开通${selectedPlan.label}会员！`);
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('支付失败', error.message || '请稍后重试');
    }
  };

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="会员中心" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  const isMember = profile?.membershipExpiry && profile.membershipExpiry * 1000 > Date.now();

  return (
    <View style={styles.container}>
      <PageHeader title="会员中心" showBack />

      <ScrollView style={styles.content}>
        {/* 当前状态 */}
        <View style={styles.statusCard}>
          <View style={styles.statusIcon}>
            <Ionicons
              name={isMember ? 'diamond' : 'person'}
              size={32}
              color={isMember ? THEME_COLOR : '#999'}
            />
          </View>
          <Text style={styles.statusTitle}>
            {isMember ? 'VIP会员' : '免费用户'}
          </Text>
          {isMember && profile?.membershipExpiry && (
            <Text style={styles.statusExpiry}>
              有效期至: {new Date(profile.membershipExpiry * 1000).toLocaleDateString()}
            </Text>
          )}
        </View>

        {/* 会员权益 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>会员专属权益</Text>
          <View style={styles.benefitsGrid}>
            {MEMBER_BENEFITS.map((benefit, index) => (
              <View key={index} style={styles.benefitItem}>
                <View style={styles.benefitIcon}>
                  <Ionicons name={benefit.icon as any} size={24} color={THEME_COLOR} />
                </View>
                <Text style={styles.benefitTitle}>{benefit.title}</Text>
                <Text style={styles.benefitDesc}>{benefit.desc}</Text>
              </View>
            ))}
          </View>
        </View>

        {/* 套餐选择 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>选择套餐</Text>
          <View style={styles.plansContainer}>
            {PLANS.map((plan) => (
              <Pressable
                key={plan.months}
                style={[
                  styles.planItem,
                  selectedPlan.months === plan.months && styles.planItemSelected,
                ]}
                onPress={() => setSelectedPlan(plan)}
              >
                {plan.popular && (
                  <View style={styles.popularBadge}>
                    <Text style={styles.popularText}>推荐</Text>
                  </View>
                )}
                <Text style={styles.planLabel}>{plan.label}</Text>
                <View style={styles.planPriceRow}>
                  <Text style={styles.planPrice}>${plan.price}</Text>
                  {plan.originalPrice && (
                    <Text style={styles.planOriginalPrice}>${plan.originalPrice}</Text>
                  )}
                </View>
                {plan.originalPrice && (
                  <Text style={styles.planDiscount}>
                    省 ${plan.originalPrice - plan.price}
                  </Text>
                )}
              </Pressable>
            ))}
          </View>
        </View>

        {/* 支付按钮 */}
        <Pressable style={styles.purchaseButton} onPress={handlePurchase}>
          <Text style={styles.purchaseButtonText}>
            {isMember ? '续费' : '立即开通'} - ${selectedPlan.price} USDT
          </Text>
        </Pressable>

        {/* 说明 */}
        <View style={styles.noteContainer}>
          <Text style={styles.noteTitle}>说明</Text>
          <Text style={styles.noteText}>• 会员费用以 DUST 代币支付（按 USDT 等值计算）</Text>
          <Text style={styles.noteText}>• 续费时间将在当前会员期限基础上累加</Text>
          <Text style={styles.noteText}>• 会员权益即时生效</Text>
        </View>
      </ScrollView>

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
  content: {
    flex: 1,
  },
  statusCard: {
    backgroundColor: '#fff',
    margin: 16,
    borderRadius: 16,
    padding: 24,
    alignItems: 'center',
  },
  statusIcon: {
    width: 64,
    height: 64,
    borderRadius: 32,
    backgroundColor: '#f8f4e8',
    justifyContent: 'center',
    alignItems: 'center',
  },
  statusTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#333',
    marginTop: 12,
  },
  statusExpiry: {
    fontSize: 14,
    color: '#666',
    marginTop: 4,
  },
  section: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginBottom: 16,
    borderRadius: 12,
    padding: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
    marginBottom: 16,
  },
  benefitsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginHorizontal: -8,
  },
  benefitItem: {
    width: '33.33%',
    paddingHorizontal: 8,
    marginBottom: 16,
    alignItems: 'center',
  },
  benefitIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: '#f8f4e8',
    justifyContent: 'center',
    alignItems: 'center',
  },
  benefitTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#333',
    marginTop: 8,
  },
  benefitDesc: {
    fontSize: 11,
    color: '#999',
    marginTop: 2,
    textAlign: 'center',
  },
  plansContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginHorizontal: -6,
  },
  planItem: {
    width: '50%',
    paddingHorizontal: 6,
    marginBottom: 12,
  },
  planItemSelected: {
    transform: [{ scale: 1 }],
  },
  popularBadge: {
    position: 'absolute',
    top: 0,
    right: 6,
    backgroundColor: '#FF6B6B',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderTopRightRadius: 8,
    borderBottomLeftRadius: 8,
    zIndex: 1,
  },
  popularText: {
    fontSize: 10,
    color: '#fff',
    fontWeight: 'bold',
  },
  planLabel: {
    fontSize: 14,
    color: '#333',
    textAlign: 'center',
    marginTop: 12,
  },
  planPriceRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
    justifyContent: 'center',
    marginTop: 8,
  },
  planPrice: {
    fontSize: 24,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  planOriginalPrice: {
    fontSize: 14,
    color: '#999',
    textDecorationLine: 'line-through',
    marginLeft: 6,
  },
  planDiscount: {
    fontSize: 12,
    color: '#FF6B6B',
    textAlign: 'center',
    marginTop: 4,
  },
  purchaseButton: {
    backgroundColor: THEME_COLOR,
    marginHorizontal: 16,
    marginBottom: 16,
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  purchaseButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: 'bold',
  },
  noteContainer: {
    marginHorizontal: 16,
    marginBottom: 32,
    padding: 16,
  },
  noteTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666',
    marginBottom: 8,
  },
  noteText: {
    fontSize: 12,
    color: '#999',
    lineHeight: 20,
  },
});
