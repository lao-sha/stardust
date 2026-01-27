// frontend/app/market/privacy-settings.tsx

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  SafeAreaView,
  StatusBar,
  Alert,
  Switch,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import storage from '@/lib/storage';
import { useWalletStore } from '@/stores/wallet.store';
import { Card } from '@/components/common';
import { useAsync } from '@/hooks';
import { THEME, SHADOWS } from '@/divination/market/theme';

const PRIVACY_SETTINGS_KEY = 'market_privacy_settings';

interface PrivacySettings {
  encryptQuestions: boolean;
  encryptReviews: boolean;
  hideAddressInReview: boolean;
  allowAnalytics: boolean;
  autoDeleteHistory: boolean;
  historyRetentionDays: number;
}

const DEFAULT_SETTINGS: PrivacySettings = {
  encryptQuestions: true,
  encryptReviews: false,
  hideAddressInReview: true,
  allowAnalytics: false,
  autoDeleteHistory: false,
  historyRetentionDays: 30,
};

export default function PrivacySettingsScreen() {
  const router = useRouter();
  const { address } = useWalletStore();
  const { execute, isLoading } = useAsync();
  const [settings, setSettings] = useState<PrivacySettings>(DEFAULT_SETTINGS);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    await execute(async () => {
      const data = await storage.getItem(PRIVACY_SETTINGS_KEY);
      if (data) {
        setSettings({ ...DEFAULT_SETTINGS, ...JSON.parse(data) });
      }
    });
  };

  const saveSettings = async (newSettings: PrivacySettings) => {
    try {
      await storage.setItem(PRIVACY_SETTINGS_KEY, JSON.stringify(newSettings));
      setSettings(newSettings);
    } catch (err) {
      console.error('Save privacy settings error:', err);
      Alert.alert('保存失败', '设置保存失败，请稍后重试');
    }
  };

  const handleToggle = (key: keyof PrivacySettings) => {
    const newSettings = {
      ...settings,
      [key]: !settings[key],
    };
    saveSettings(newSettings);
  };

  const handleClearHistory = () => {
    Alert.alert(
      '清除历史',
      '确定要清除所有本地历史记录吗？此操作不可恢复。',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定',
          style: 'destructive',
          onPress: async () => {
            try {
              await storage.multiRemove([
                'market_search_history',
                'market_viewed_providers',
                'market_draft_orders',
              ]);
              Alert.alert('成功', '历史记录已清除');
            } catch (err) {
              Alert.alert('失败', '清除失败');
            }
          },
        },
      ]
    );
  };

  const handleExportData = () => {
    Alert.alert('功能开发中', '数据导出功能即将推出');
  };

  const handleDeleteAccount = () => {
    Alert.alert(
      '注销账户',
      '确定要注销解卦师账户吗？注销后您的所有套餐和未完成订单将被取消。',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定注销',
          style: 'destructive',
          onPress: () => {
            // TODO: 调用链上交易注销账户
            Alert.alert('提示', '请在链上完成注销操作');
          },
        },
      ]
    );
  };

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.card} />

      {/* 顶部导航 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
          <Ionicons name="arrow-back" size={24} color={THEME.text} />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>隐私设置</Text>
        <View style={styles.backBtn} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 数据加密 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>数据加密</Text>
          <Text style={styles.sectionDesc}>
            选择哪些内容需要端到端加密保护
          </Text>

          <View style={styles.settingItem}>
            <View style={styles.settingInfo}>
              <Ionicons name="lock-closed-outline" size={20} color={THEME.primary} />
              <View style={styles.settingTextContainer}>
                <Text style={styles.settingLabel}>加密问题描述</Text>
                <Text style={styles.settingDesc}>
                  您的问题内容将被加密，仅解卦师可见
                </Text>
              </View>
            </View>
            <Switch
              value={settings.encryptQuestions}
              onValueChange={() => handleToggle('encryptQuestions')}
              trackColor={{ false: THEME.border, true: THEME.primary + '60' }}
              thumbColor={settings.encryptQuestions ? THEME.primary : THEME.textTertiary}
            />
          </View>

          <View style={styles.settingItem}>
            <View style={styles.settingInfo}>
              <Ionicons name="chatbubble-outline" size={20} color={THEME.primary} />
              <View style={styles.settingTextContainer}>
                <Text style={styles.settingLabel}>加密评价内容</Text>
                <Text style={styles.settingDesc}>
                  评价内容将被加密，仅相关方可见
                </Text>
              </View>
            </View>
            <Switch
              value={settings.encryptReviews}
              onValueChange={() => handleToggle('encryptReviews')}
              trackColor={{ false: THEME.border, true: THEME.primary + '60' }}
              thumbColor={settings.encryptReviews ? THEME.primary : THEME.textTertiary}
            />
          </View>
        </Card>

        {/* 身份保护 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>身份保护</Text>

          <View style={styles.settingItem}>
            <View style={styles.settingInfo}>
              <Ionicons name="eye-off-outline" size={20} color={THEME.primary} />
              <View style={styles.settingTextContainer}>
                <Text style={styles.settingLabel}>默认匿名评价</Text>
                <Text style={styles.settingDesc}>
                  发表评价时默认隐藏您的钱包地址
                </Text>
              </View>
            </View>
            <Switch
              value={settings.hideAddressInReview}
              onValueChange={() => handleToggle('hideAddressInReview')}
              trackColor={{ false: THEME.border, true: THEME.primary + '60' }}
              thumbColor={settings.hideAddressInReview ? THEME.primary : THEME.textTertiary}
            />
          </View>
        </Card>

        {/* 数据收集 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>数据收集</Text>

          <View style={styles.settingItem}>
            <View style={styles.settingInfo}>
              <Ionicons name="analytics-outline" size={20} color={THEME.primary} />
              <View style={styles.settingTextContainer}>
                <Text style={styles.settingLabel}>允许数据分析</Text>
                <Text style={styles.settingDesc}>
                  帮助我们改进产品体验（不包含敏感信息）
                </Text>
              </View>
            </View>
            <Switch
              value={settings.allowAnalytics}
              onValueChange={() => handleToggle('allowAnalytics')}
              trackColor={{ false: THEME.border, true: THEME.primary + '60' }}
              thumbColor={settings.allowAnalytics ? THEME.primary : THEME.textTertiary}
            />
          </View>

          <View style={styles.settingItem}>
            <View style={styles.settingInfo}>
              <Ionicons name="trash-outline" size={20} color={THEME.primary} />
              <View style={styles.settingTextContainer}>
                <Text style={styles.settingLabel}>自动清除历史</Text>
                <Text style={styles.settingDesc}>
                  定期自动清除本地浏览历史
                </Text>
              </View>
            </View>
            <Switch
              value={settings.autoDeleteHistory}
              onValueChange={() => handleToggle('autoDeleteHistory')}
              trackColor={{ false: THEME.border, true: THEME.primary + '60' }}
              thumbColor={settings.autoDeleteHistory ? THEME.primary : THEME.textTertiary}
            />
          </View>
        </Card>

        {/* 数据管理 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>数据管理</Text>

          <TouchableOpacity style={styles.actionItem} onPress={handleClearHistory}>
            <View style={styles.actionInfo}>
              <Ionicons name="time-outline" size={20} color={THEME.text} />
              <Text style={styles.actionLabel}>清除本地历史</Text>
            </View>
            <Ionicons name="chevron-forward" size={20} color={THEME.textTertiary} />
          </TouchableOpacity>

          <TouchableOpacity style={styles.actionItem} onPress={handleExportData}>
            <View style={styles.actionInfo}>
              <Ionicons name="download-outline" size={20} color={THEME.text} />
              <Text style={styles.actionLabel}>导出我的数据</Text>
            </View>
            <Ionicons name="chevron-forward" size={20} color={THEME.textTertiary} />
          </TouchableOpacity>
        </Card>

        {/* 危险操作 */}
        <Card style={styles.section}>
          <Text style={[styles.sectionTitle, { color: THEME.error }]}>
            危险操作
          </Text>

          <TouchableOpacity style={styles.actionItem} onPress={handleDeleteAccount}>
            <View style={styles.actionInfo}>
              <Ionicons name="person-remove-outline" size={20} color={THEME.error} />
              <Text style={[styles.actionLabel, { color: THEME.error }]}>
                注销解卦师账户
              </Text>
            </View>
            <Ionicons name="chevron-forward" size={20} color={THEME.error} />
          </TouchableOpacity>
        </Card>

        {/* 隐私说明 */}
        <View style={styles.notice}>
          <Ionicons name="shield-checkmark-outline" size={18} color={THEME.success} />
          <Text style={styles.noticeText}>
            您的隐私数据受到区块链和端到端加密的双重保护。
            我们不会出售或共享您的个人信息。
          </Text>
        </View>

        <View style={styles.bottomSpace} />
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: THEME.card,
    paddingHorizontal: 8,
    paddingVertical: 10,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  backBtn: {
    padding: 8,
    width: 40,
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME.text,
  },
  content: {
    flex: 1,
    padding: 16,
  },
  section: {
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 4,
  },
  sectionDesc: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginBottom: 12,
  },
  settingItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.borderLight,
  },
  settingInfo: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginRight: 16,
  },
  settingTextContainer: {
    flex: 1,
    marginLeft: 12,
  },
  settingLabel: {
    fontSize: 14,
    color: THEME.text,
    fontWeight: '500',
  },
  settingDesc: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 2,
    lineHeight: 16,
  },
  actionItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 14,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.borderLight,
  },
  actionInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  actionLabel: {
    fontSize: 14,
    color: THEME.text,
  },
  notice: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    backgroundColor: THEME.success + '10',
    borderRadius: 8,
    padding: 12,
    gap: 8,
  },
  noticeText: {
    flex: 1,
    fontSize: 12,
    color: THEME.success,
    lineHeight: 18,
  },
  bottomSpace: {
    height: 32,
  },
});
