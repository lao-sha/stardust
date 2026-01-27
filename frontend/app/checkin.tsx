/**
 * 星尘玄鉴 - 每日签到页面
 * 连续签到可获得 DUST 代币奖励
 * 主题色：金棕色 #B2955D
 */

import { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { BottomNavBar } from '@/components/BottomNavBar';
import { Card, Button } from '@/components/common';
import { useAsync } from '@/hooks';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_BG = '#F5F5F7';

// 签到奖励配置
const CHECKIN_REWARDS = [
  { day: 1, reward: 1, label: '第1天' },
  { day: 2, reward: 2, label: '第2天' },
  { day: 3, reward: 3, label: '第3天' },
  { day: 4, reward: 5, label: '第4天' },
  { day: 5, reward: 8, label: '第5天' },
  { day: 6, reward: 10, label: '第6天' },
  { day: 7, reward: 20, label: '第7天' },
];

export default function CheckinPage() {
  const router = useRouter();
  const { execute, isLoading } = useAsync();
  const [checkedDays, setCheckedDays] = useState(0);
  const [todayChecked, setTodayChecked] = useState(false);
  const [totalReward, setTotalReward] = useState(0);

  // 处理签到
  const handleCheckin = async () => {
    if (todayChecked) {
      Alert.alert('提示', '今日已签到，明天再来吧！');
      return;
    }

    await execute(async () => {
      const nextDay = (checkedDays % 7) + 1;
      const reward = CHECKIN_REWARDS.find(r => r.day === nextDay)?.reward || 1;

      setCheckedDays(prev => prev + 1);
      setTodayChecked(true);
      setTotalReward(prev => prev + reward);

      Alert.alert(
        '签到成功！',
        `恭喜获得 ${reward} DUST 代币\n\n连续签到 ${nextDay} 天`,
        [{ text: '太棒了', style: 'default' }]
      );
    });
  };

  // 获取当前周期的天数
  const currentCycleDay = (checkedDays % 7) + (todayChecked ? 0 : 1);

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>每日签到</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 签到状态卡片 */}
        <Card style={styles.statusCard}>
          <View style={styles.statusHeader}>
            <View style={styles.statusIcon}>
              <Ionicons name="gift" size={32} color="#FFF" />
            </View>
            <View style={styles.statusInfo}>
              <Text style={styles.statusTitle}>
                {todayChecked ? '今日已签到' : '今日未签到'}
              </Text>
              <Text style={styles.statusSubtitle}>
                累计签到 {checkedDays} 天 · 获得 {totalReward} DUST
              </Text>
            </View>
          </View>

          <Button
            title={todayChecked ? '已签到' : '立即签到'}
            onPress={handleCheckin}
            loading={isLoading}
            disabled={todayChecked || isLoading}
            icon={todayChecked ? 'checkmark-circle' : 'gift-outline'}
          />
        </Card>

        {/* 7天签到奖励 */}
        <View style={styles.rewardSection}>
          <Text style={styles.sectionTitle}>7天签到奖励</Text>
          <View style={styles.rewardGrid}>
            {CHECKIN_REWARDS.map((item, index) => {
              const isChecked = todayChecked
                ? index < currentCycleDay
                : index < currentCycleDay - 1;
              const isToday = !todayChecked && index === currentCycleDay - 1;

              return (
                <View
                  key={item.day}
                  style={[
                    styles.rewardItem,
                    isChecked && styles.rewardItemChecked,
                    isToday && styles.rewardItemToday,
                  ]}
                >
                  <View style={[
                    styles.rewardIconBox,
                    isChecked && styles.rewardIconBoxChecked,
                  ]}>
                    {isChecked ? (
                      <Ionicons name="checkmark" size={20} color="#FFF" />
                    ) : (
                      <Ionicons name="gift-outline" size={20} color={isToday ? THEME_COLOR : '#999'} />
                    )}
                  </View>
                  <Text style={[styles.rewardDay, isChecked && styles.rewardDayChecked]}>
                    {item.label}
                  </Text>
                  <Text style={[styles.rewardAmount, isChecked && styles.rewardAmountChecked]}>
                    +{item.reward} DUST
                  </Text>
                </View>
              );
            })}
          </View>
        </View>

        {/* 签到规则 */}
        <View style={styles.rulesSection}>
          <Text style={styles.sectionTitle}>签到规则</Text>
          <Card>
            <View style={styles.ruleItem}>
              <Ionicons name="checkmark-circle-outline" size={18} color={THEME_COLOR} />
              <Text style={styles.ruleText}>每日可签到一次，领取 DUST 代币奖励</Text>
            </View>
            <View style={styles.ruleItem}>
              <Ionicons name="checkmark-circle-outline" size={18} color={THEME_COLOR} />
              <Text style={styles.ruleText}>连续签到7天，奖励递增，第7天可获20 DUST</Text>
            </View>
            <View style={styles.ruleItem}>
              <Ionicons name="checkmark-circle-outline" size={18} color={THEME_COLOR} />
              <Text style={styles.ruleText}>中断签到后，从第1天重新开始计算</Text>
            </View>
            <View style={styles.ruleItem}>
              <Ionicons name="checkmark-circle-outline" size={18} color={THEME_COLOR} />
              <Text style={styles.ruleText}>签到奖励将自动存入您的钱包账户</Text>
            </View>
          </Card>
        </View>

        {/* 底部提示 */}
        <View style={styles.footer}>
          <Text style={styles.footerText}>签到数据将上链存证，确保公平公正</Text>
        </View>

        {/* 底部间距 */}
        <View style={{ height: 80 }} />
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="index" />
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
  navBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backButton: {
    padding: 4,
  },
  navTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  placeholder: {
    width: 32,
  },
  content: {
    flex: 1,
    padding: 16,
  },
  statusCard: {
    backgroundColor: THEME_COLOR,
    marginBottom: 20,
  },
  statusHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 20,
  },
  statusIcon: {
    width: 60,
    height: 60,
    borderRadius: 30,
    backgroundColor: 'rgba(255,255,255,0.2)',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 16,
  },
  statusInfo: {
    flex: 1,
  },
  statusTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#FFF',
    marginBottom: 4,
  },
  statusSubtitle: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
  },
  rewardSection: {
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  rewardGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 10,
  },
  rewardItem: {
    width: '31%',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 12,
    alignItems: 'center',
    borderWidth: 2,
    borderColor: 'transparent',
  },
  rewardItemChecked: {
    backgroundColor: '#F0F8F0',
    borderColor: '#52c41a',
  },
  rewardItemToday: {
    borderColor: THEME_COLOR,
  },
  rewardIconBox: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#F5F5F5',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 8,
  },
  rewardIconBoxChecked: {
    backgroundColor: '#52c41a',
  },
  rewardDay: {
    fontSize: 13,
    color: '#666',
    marginBottom: 4,
  },
  rewardDayChecked: {
    color: '#52c41a',
  },
  rewardAmount: {
    fontSize: 12,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  rewardAmountChecked: {
    color: '#52c41a',
  },
  rulesSection: {
    marginBottom: 20,
  },
  ruleItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: 10,
  },
  ruleText: {
    flex: 1,
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
  },
  footer: {
    alignItems: 'center',
    paddingVertical: 20,
  },
  footerText: {
    fontSize: 12,
    color: '#999',
  },
});
