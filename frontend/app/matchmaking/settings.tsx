/**
 * 隐私设置页面
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
  Switch,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { matchmakingService, MatchmakingPrivacyMode, UserProfile } from '@/services/matchmaking.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

export default function SettingsPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [privacyMode, setPrivacyMode] = useState<MatchmakingPrivacyMode>(MatchmakingPrivacyMode.Public);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingMode, setPendingMode] = useState<MatchmakingPrivacyMode | null>(null);

  const loadProfile = useCallback(async () => {
    if (!address) return;

    try {
      const userProfile = await matchmakingService.getProfile(address);
      if (userProfile) {
        setProfile(userProfile);
        setPrivacyMode(userProfile.privacyMode);
      }
    } catch (error) {
      console.error('Load profile error:', error);
    } finally {
      setLoading(false);
    }
  }, [address]);

  useEffect(() => {
    loadProfile();
  }, [loadProfile]);

  const handlePrivacyModeChange = async (mode: MatchmakingPrivacyMode) => {
    if (mode === privacyMode) return;

    if (!isSignerUnlocked()) {
      setPendingMode(mode);
      setShowUnlockDialog(true);
      return;
    }

    await executePrivacyModeChange(mode);
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      if (pendingMode !== null) {
        await executePrivacyModeChange(pendingMode);
        setPendingMode(null);
      }
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executePrivacyModeChange = async (mode: MatchmakingPrivacyMode) => {
    setShowTxStatus(true);
    setTxStatus('正在更新隐私设置...');

    try {
      await matchmakingService.updatePrivacyMode(mode, (status) => setTxStatus(status));

      setPrivacyMode(mode);
      setTxStatus('设置已更新！');
      setTimeout(() => {
        setShowTxStatus(false);
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('操作失败', error.message || '请稍后重试');
    }
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

  const getPrivacyModeDesc = (mode: MatchmakingPrivacyMode) => {
    switch (mode) {
      case MatchmakingPrivacyMode.Public:
        return '所有用户都可以查看您的资料';
      case MatchmakingPrivacyMode.MembersOnly:
        return '只有付费会员可以查看您的资料';
      case MatchmakingPrivacyMode.MatchedOnly:
        return '只有与您匹配成功的用户可以查看完整资料';
      default:
        return '';
    }
  };

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="隐私设置" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  if (!profile) {
    return (
      <View style={styles.container}>
        <PageHeader title="隐私设置" showBack />
        <View style={styles.emptyContainer}>
          <Ionicons name="person-outline" size={80} color="#ccc" />
          <Text style={styles.emptyTitle}>尚未创建资料</Text>
          <Text style={styles.emptyText}>请先创建您的婚恋资料</Text>
          <Pressable
            style={styles.createButton}
            onPress={() => router.push('/matchmaking/create-profile' as any)}
          >
            <Text style={styles.createButtonText}>创建资料</Text>
          </Pressable>
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="隐私设置" showBack />

      <ScrollView style={styles.content}>
        {/* 隐私模式设置 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>资料可见性</Text>
          <Text style={styles.sectionDesc}>设置谁可以查看您的资料</Text>

          {[
            MatchmakingPrivacyMode.Public,
            MatchmakingPrivacyMode.MembersOnly,
            MatchmakingPrivacyMode.MatchedOnly,
          ].map((mode) => (
            <Pressable
              key={mode}
              style={[
                styles.optionItem,
                privacyMode === mode && styles.optionItemActive,
              ]}
              onPress={() => handlePrivacyModeChange(mode)}
            >
              <View style={styles.optionContent}>
                <Text style={[
                  styles.optionTitle,
                  privacyMode === mode && styles.optionTitleActive,
                ]}>
                  {getPrivacyModeText(mode)}
                </Text>
                <Text style={styles.optionDesc}>
                  {getPrivacyModeDesc(mode)}
                </Text>
              </View>
              {privacyMode === mode && (
                <Ionicons name="checkmark-circle" size={24} color={THEME_COLOR} />
              )}
            </Pressable>
          ))}
        </View>

        {/* 会员状态 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>会员状态</Text>
          
          <View style={styles.membershipCard}>
            {profile.membershipExpiry ? (
              <>
                <View style={styles.membershipBadge}>
                  <Ionicons name="diamond" size={20} color={THEME_COLOR} />
                  <Text style={styles.membershipBadgeText}>VIP会员</Text>
                </View>
                <Text style={styles.membershipExpiry}>
                  有效期至: {new Date(profile.membershipExpiry * 1000).toLocaleDateString()}
                </Text>
                <Pressable
                  style={styles.renewButton}
                  onPress={() => router.push('/matchmaking/membership' as any)}
                >
                  <Text style={styles.renewButtonText}>续费</Text>
                </Pressable>
              </>
            ) : (
              <>
                <View style={styles.membershipBadge}>
                  <Ionicons name="person" size={20} color="#999" />
                  <Text style={[styles.membershipBadgeText, { color: '#999' }]}>免费用户</Text>
                </View>
                <Text style={styles.membershipDesc}>
                  升级会员享受更多特权
                </Text>
                <Pressable
                  style={styles.upgradeButton}
                  onPress={() => router.push('/matchmaking/membership' as any)}
                >
                  <Text style={styles.upgradeButtonText}>升级会员</Text>
                </Pressable>
              </>
            )}
          </View>
        </View>

        {/* 其他设置 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>其他</Text>

          <Pressable
            style={styles.menuItem}
            onPress={() => router.push('/matchmaking/preferences' as any)}
          >
            <View style={styles.menuItemLeft}>
              <Ionicons name="options" size={22} color="#666" />
              <Text style={styles.menuItemText}>择偶条件</Text>
            </View>
            <Ionicons name="chevron-forward" size={20} color="#ccc" />
          </Pressable>

          <Pressable
            style={styles.menuItem}
            onPress={() => Alert.alert('提示', '此功能即将上线')}
          >
            <View style={styles.menuItemLeft}>
              <Ionicons name="shield-checkmark" size={22} color="#666" />
              <Text style={styles.menuItemText}>实名认证</Text>
            </View>
            <Ionicons name="chevron-forward" size={20} color="#ccc" />
          </Pressable>

          <Pressable
            style={[styles.menuItem, styles.dangerItem]}
            onPress={() => {
              Alert.alert(
                '确认删除',
                '删除资料后将无法恢复，保证金将退还。确定要删除吗？',
                [
                  { text: '取消', style: 'cancel' },
                  {
                    text: '删除',
                    style: 'destructive',
                    onPress: async () => {
                      if (!isSignerUnlocked()) {
                        setShowUnlockDialog(true);
                        return;
                      }
                      setShowTxStatus(true);
                      setTxStatus('正在删除资料...');
                      try {
                        await matchmakingService.deleteProfile((status) => setTxStatus(status));
                        setTxStatus('资料已删除');
                        setTimeout(() => {
                          setShowTxStatus(false);
                          router.replace('/matchmaking' as any);
                        }, 1500);
                      } catch (error: any) {
                        setShowTxStatus(false);
                        Alert.alert('删除失败', error.message || '请稍后重试');
                      }
                    },
                  },
                ]
              );
            }}
          >
            <View style={styles.menuItemLeft}>
              <Ionicons name="trash" size={22} color="#FF3B30" />
              <Text style={[styles.menuItemText, { color: '#FF3B30' }]}>删除资料</Text>
            </View>
            <Ionicons name="chevron-forward" size={20} color="#ccc" />
          </Pressable>
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
  content: {
    flex: 1,
  },
  section: {
    backgroundColor: '#fff',
    marginTop: 16,
    marginHorizontal: 16,
    borderRadius: 12,
    padding: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
  },
  sectionDesc: {
    fontSize: 13,
    color: '#999',
    marginTop: 4,
    marginBottom: 12,
  },
  optionItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    borderRadius: 8,
    marginTop: 8,
    backgroundColor: '#f8f8f8',
  },
  optionItemActive: {
    backgroundColor: '#f8f4e8',
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  optionContent: {
    flex: 1,
  },
  optionTitle: {
    fontSize: 15,
    fontWeight: '500',
    color: '#333',
  },
  optionTitleActive: {
    color: THEME_COLOR,
  },
  optionDesc: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  membershipCard: {
    backgroundColor: '#f8f8f8',
    borderRadius: 12,
    padding: 16,
    marginTop: 12,
    alignItems: 'center',
  },
  membershipBadge: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  membershipBadgeText: {
    fontSize: 16,
    fontWeight: 'bold',
    color: THEME_COLOR,
    marginLeft: 6,
  },
  membershipExpiry: {
    fontSize: 13,
    color: '#666',
    marginTop: 8,
  },
  membershipDesc: {
    fontSize: 13,
    color: '#666',
    marginTop: 8,
  },
  renewButton: {
    backgroundColor: '#f8f4e8',
    paddingHorizontal: 20,
    paddingVertical: 8,
    borderRadius: 16,
    marginTop: 12,
  },
  renewButtonText: {
    color: THEME_COLOR,
    fontSize: 13,
    fontWeight: '500',
  },
  upgradeButton: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 20,
    paddingVertical: 8,
    borderRadius: 16,
    marginTop: 12,
  },
  upgradeButtonText: {
    color: '#fff',
    fontSize: 13,
    fontWeight: '500',
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 14,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  menuItemLeft: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  menuItemText: {
    fontSize: 15,
    color: '#333',
    marginLeft: 12,
  },
  dangerItem: {
    borderBottomWidth: 0,
  },
});
