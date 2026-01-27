/**
 * 扣除记录列表页面
 * 路径: /maker/penalties
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore } from '@/stores/maker.store';
import { MakerService, PenaltyRecord } from '@/services/maker.service';
import { PenaltyCard } from '@/features/maker/components';
import { PageHeader } from '@/components/PageHeader';
import { Card, LoadingSpinner, EmptyState } from '@/components/common';

type FilterType = 'all' | 'unappealed' | 'appealed';

export default function PenaltiesPage() {
  const router = useRouter();
  const {
    penalties,
    loadingPenalties,
    fetchPenalties,
  } = useMakerStore();

  const [refreshing, setRefreshing] = useState(false);
  const [filter, setFilter] = useState<FilterType>('all');

  useEffect(() => {
    fetchPenalties();
  }, []);

  const onRefresh = async () => {
    setRefreshing(true);
    await fetchPenalties();
    setRefreshing(false);
  };

  const filteredPenalties = penalties.filter((p) => {
    if (filter === 'unappealed') return !p.appealed;
    if (filter === 'appealed') return p.appealed;
    return true;
  });

  // 计算统计
  const totalDeducted = penalties.reduce((sum, p) => sum + p.usdValue, 0);
  const unappealedCount = penalties.filter((p) => !p.appealed).length;

  const handleViewDetail = (penaltyId: number) => {
    router.push(`/maker/penalties/${penaltyId}`);
  };

  const handleAppeal = (penaltyId: number) => {
    router.push(`/maker/penalties/${penaltyId}/appeal`);
  };

  if (loadingPenalties && penalties.length === 0) {
    return (
      <View style={styles.loadingContainer}>
        <LoadingSpinner text="加载中..." />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="扣除记录" showBack />

      <ScrollView
        style={styles.content}
        showsVerticalScrollIndicator={false}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
      >
        {/* 统计卡片 */}
        <Card style={styles.section}>
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{penalties.length}</Text>
            <Text style={styles.statLabel}>总记录</Text>
          </View>
          <View style={styles.statDivider} />
          <View style={styles.statItem}>
            <Text style={[styles.statValue, styles.statValueRed]}>
              ${MakerService.formatUsdAmount(totalDeducted)}
            </Text>
            <Text style={styles.statLabel}>累计扣除</Text>
          </View>
          <View style={styles.statDivider} />
          <View style={styles.statItem}>
            <Text style={[styles.statValue, unappealedCount > 0 && styles.statValueOrange]}>
              {unappealedCount}
            </Text>
            <Text style={styles.statLabel}>待申诉</Text>
          </View>
        </Card>

        {/* 筛选器 */}
        <View style={styles.filterContainer}>
          <TouchableOpacity
            style={[styles.filterButton, filter === 'all' && styles.filterButtonActive]}
            onPress={() => setFilter('all')}
          >
            <Text style={[styles.filterText, filter === 'all' && styles.filterTextActive]}>
              全部
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.filterButton, filter === 'unappealed' && styles.filterButtonActive]}
            onPress={() => setFilter('unappealed')}
          >
            <Text style={[styles.filterText, filter === 'unappealed' && styles.filterTextActive]}>
              未申诉
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.filterButton, filter === 'appealed' && styles.filterButtonActive]}
            onPress={() => setFilter('appealed')}
          >
            <Text style={[styles.filterText, filter === 'appealed' && styles.filterTextActive]}>
              已申诉
            </Text>
          </TouchableOpacity>
        </View>

        {/* 记录列表 */}
        {filteredPenalties.length === 0 ? (
          <EmptyState
            icon="document-text-outline"
            title="暂无扣除记录"
            description="您的扣除记录将显示在这里"
          />
        ) : (
          <View style={styles.list}>
            {filteredPenalties.map((penalty) => (
              <PenaltyCard
                key={penalty.id}
                penalty={penalty}
                onViewDetail={() => handleViewDetail(penalty.id)}
                onAppeal={() => handleAppeal(penalty.id)}
              />
            ))}
          </View>
        )}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  section: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 20,
    fontWeight: '700',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  statValueRed: {
    color: '#FF3B30',
  },
  statValueOrange: {
    color: '#FF9500',
  },
  statLabel: {
    fontSize: 12,
    color: '#8E8E93',
  },
  statDivider: {
    width: 1,
    backgroundColor: '#F2F2F7',
    marginHorizontal: 8,
  },
  filterContainer: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 16,
  },
  filterButton: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
    backgroundColor: '#FFFFFF',
  },
  filterButtonActive: {
    backgroundColor: '#B2955D',
  },
  filterText: {
    fontSize: 14,
    color: '#8E8E93',
  },
  filterTextActive: {
    color: '#FFFFFF',
    fontWeight: '500',
  },
  list: {
    gap: 0,
  },
});
