/**
 * 占卜历史记录总览页面
 * 显示用户所有类型的占卜记录
 */

import React, { useState, useEffect } from 'react';
import { View, StyleSheet, Pressable, Text, ScrollView, Alert } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { DivinationHistory } from '@/components/DivinationHistory';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import { LoadingSpinner, EmptyState } from '@/components/common';
import {
  DivinationType,
  DivinationRecord,
  divinationService,
} from '@/services/divination.service';
import { useWallet, useAsync } from '@/hooks';

const THEME_COLOR = '#B2955D';

export default function DivinationHistoryPage() {
  const router = useRouter();
  const { address } = useWallet();
  const { execute, isLoading } = useAsync();

  const [selectedType, setSelectedType] = useState<DivinationType | undefined>(undefined);
  const [stats, setStats] = useState<{ total: number; byType: Record<DivinationType, number> } | null>(null);

  useEffect(() => {
    if (address) {
      loadStats();
    }
  }, [address]);

  const loadStats = async () => {
    try {
      await execute(async () => {
        const statistics = await divinationService.getDivinationStats(address!);
        setStats(statistics);
      });
    } catch (error) {
      console.error('Load stats error:', error);
      Alert.alert('错误', '加载统计数据失败');
    }
  };

  const handleRecordPress = (record: DivinationRecord) => {
    // TODO: 导航到详情页
    console.log('查看占卜记录:', record);
  };

  const typeFilters = [
    { type: undefined, label: '全部', icon: 'apps-outline' },
    { type: DivinationType.Bazi, label: '八字', icon: 'calendar-outline' },
    { type: DivinationType.Ziwei, label: '紫微', icon: 'star-outline' },
    { type: DivinationType.Qimen, label: '奇门', icon: 'grid-outline' },
    { type: DivinationType.Liuyao, label: '六爻', icon: 'layers-outline' },
    { type: DivinationType.Meihua, label: '梅花', icon: 'flower-outline' },
    { type: DivinationType.Tarot, label: '塔罗', icon: 'card-outline' },
    { type: DivinationType.Daliuren, label: '大六壬', icon: 'compass-outline' },
    { type: DivinationType.Xiaoliuren, label: '小六壬', icon: 'hand-left-outline' },
  ];

  const getTypeCount = (type: DivinationType | undefined) => {
    if (!stats) return 0;
    if (type === undefined) return stats.total;
    return stats.byType[type] || 0;
  };

  return (
    <View style={styles.container}>
      <PageHeader title="占卜记录" />

      {/* 类型筛选 */}
      <ScrollView
        horizontal
        showsHorizontalScrollIndicator={false}
        style={styles.filterBar}
        contentContainerStyle={styles.filterContent}
      >
        {typeFilters.map((filter) => {
          const isActive = selectedType === filter.type;
          const count = getTypeCount(filter.type);

          return (
            <Pressable
              key={filter.label}
              style={[styles.filterChip, isActive && styles.filterChipActive]}
              onPress={() => setSelectedType(filter.type)}
            >
              <Ionicons
                name={filter.icon as any}
                size={16}
                color={isActive ? '#FFF' : THEME_COLOR}
              />
              <Text style={[styles.filterLabel, isActive && styles.filterLabelActive]}>
                {filter.label}
              </Text>
              {count > 0 && (
                <View style={[styles.countBadge, isActive && styles.countBadgeActive]}>
                  <Text style={[styles.countText, isActive && styles.countTextActive]}>
                    {count}
                  </Text>
                </View>
              )}
            </Pressable>
          );
        })}
      </ScrollView>

      {/* 历史记录列表 */}
      <DivinationHistory
        divinationType={selectedType}
        onRecordPress={handleRecordPress}
        emptyMessage={
          selectedType
            ? `暂无${typeFilters.find((f) => f.type === selectedType)?.label}记录`
            : '暂无占卜记录'
        }
      />

      <BottomNavBar activeTab="divination" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  filterBar: {
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  filterContent: {
    paddingHorizontal: 16,
    paddingVertical: 12,
    gap: 8,
  },
  filterChip: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 20,
    borderWidth: 1,
    borderColor: THEME_COLOR,
    backgroundColor: '#FFF',
    gap: 6,
  },
  filterChipActive: {
    backgroundColor: THEME_COLOR,
    borderColor: THEME_COLOR,
  },
  filterLabel: {
    fontSize: 13,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  filterLabelActive: {
    color: '#FFF',
  },
  countBadge: {
    backgroundColor: THEME_COLOR + '20',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 10,
    minWidth: 20,
    alignItems: 'center',
  },
  countBadgeActive: {
    backgroundColor: '#FFF',
  },
  countText: {
    fontSize: 11,
    color: THEME_COLOR,
    fontWeight: '600',
  },
  countTextActive: {
    color: THEME_COLOR,
  },
});
