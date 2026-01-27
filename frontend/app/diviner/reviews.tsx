/**
 * 评价管理页面
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  ActivityIndicator,
  RefreshControl,
  Alert,
} from 'react-native';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { ReviewCard, Review } from '@/features/diviner';

const THEME_COLOR = '#B2955D';

// Mock 数据
const mockReviews: Review[] = [
  {
    orderId: 1001,
    customer: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    overallRating: 5,
    accuracyRating: 5,
    attitudeRating: 5,
    responseRating: 4,
    contentCid: 'QmReview1...',
    isAnonymous: false,
    createdAt: Date.now() - 86400000,
  },
  {
    orderId: 1002,
    customer: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
    provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    overallRating: 4,
    accuracyRating: 4,
    attitudeRating: 5,
    responseRating: 4,
    isAnonymous: true,
    replyCid: 'QmReply1...',
    createdAt: Date.now() - 172800000,
  },
  {
    orderId: 1003,
    customer: '5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw',
    provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    overallRating: 5,
    accuracyRating: 5,
    attitudeRating: 5,
    responseRating: 5,
    contentCid: 'QmReview3...',
    isAnonymous: false,
    createdAt: Date.now() - 259200000,
  },
];

export default function ReviewsPage() {
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [reviews, setReviews] = useState<Review[]>([]);

  // 统计数据
  const avgRating = reviews.length > 0
    ? (reviews.reduce((sum, r) => sum + (r.overallRating + r.accuracyRating + r.attitudeRating + r.responseRating) / 4, 0) / reviews.length).toFixed(1)
    : '0.0';

  const loadData = async () => {
    await new Promise(resolve => setTimeout(resolve, 500));
    setReviews(mockReviews);
  };

  useEffect(() => {
    loadData().finally(() => setLoading(false));
  }, []);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const handleReply = async (orderId: number) => {
    Alert.prompt(
      '回复评价',
      '请输入您的回复内容',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '提交',
          onPress: async (text) => {
            if (text?.trim()) {
              try {
                const { divinationMarketService } = await import('@/services/divination-market.service');
                // 找到对应的评价 ID（这里假设 orderId 就是 reviewId）
                await divinationMarketService.replyReview(orderId, text.trim(), (status) => {
                  console.log('Reply status:', status);
                });
                // 更新本地状态
                setReviews(prev =>
                  prev.map(r => (r.orderId === orderId ? { ...r, replyCid: 'QmNewReply...' } : r))
                );
                Alert.alert('成功', '回复已提交');
              } catch (error: any) {
                Alert.alert('回复失败', error.message || '请稍后重试');
              }
            }
          },
        },
      ],
      'plain-text'
    );
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="评价管理" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="评价管理" />

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={THEME_COLOR} />}
      >
        {/* 统计卡片 */}
        <View style={styles.statsCard}>
          <View style={styles.statMain}>
            <Text style={styles.statValue}>{avgRating}</Text>
            <Text style={styles.statLabel}>平均评分</Text>
          </View>
          <View style={styles.statDivider} />
          <View style={styles.statItem}>
            <Text style={styles.statItemValue}>{reviews.length}</Text>
            <Text style={styles.statItemLabel}>总评价数</Text>
          </View>
          <View style={styles.statDivider} />
          <View style={styles.statItem}>
            <Text style={styles.statItemValue}>{reviews.filter(r => !r.replyCid).length}</Text>
            <Text style={styles.statItemLabel}>待回复</Text>
          </View>
        </View>

        {/* 评价列表 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>全部评价</Text>
          {reviews.length === 0 ? (
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyIcon}>⭐</Text>
              <Text style={styles.emptyText}>暂无评价</Text>
            </View>
          ) : (
            reviews.map(review => (
              <ReviewCard
                key={review.orderId}
                review={review}
                showReplyButton
                onReply={() => handleReply(review.orderId)}
              />
            ))
          )}
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
  statsCard: {
    flexDirection: 'row',
    backgroundColor: '#FFF',
    margin: 16,
    borderRadius: 12,
    padding: 20,
    alignItems: 'center',
  },
  statMain: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 36,
    fontWeight: '700',
    color: THEME_COLOR,
  },
  statLabel: {
    fontSize: 14,
    color: '#666',
    marginTop: 4,
  },
  statDivider: {
    width: 1,
    height: 40,
    backgroundColor: '#F0F0F0',
    marginHorizontal: 16,
  },
  statItem: {
    alignItems: 'center',
  },
  statItemValue: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
  },
  statItemLabel: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
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
  emptyContainer: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 48,
    alignItems: 'center',
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
  },
});
