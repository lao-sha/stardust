// frontend/app/market/review/create.tsx

import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  TouchableOpacity,
  SafeAreaView,
  StatusBar,
  Alert,
  Switch,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useWalletStore } from '@/stores/wallet.store';
import { useOrders, useReviews, useMarketApi, useChainTransaction } from '@/divination/market/hooks';
import {
  Avatar,
  TierBadge,
  RatingStars,
  DivinationTypeBadge,
  LoadingSpinner,
  TransactionStatus,
} from '@/divination/market/components';
import { Card, Button } from '@/components/common';
import { useAsync } from '@/hooks';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { Order, Provider } from '@/divination/market/types';
import { getIpfsUrl } from '@/divination/market/services/ipfs.service';

interface RatingItemProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
}

const RatingItem: React.FC<RatingItemProps> = ({ label, value, onChange }) => {
  return (
    <View style={styles.ratingItem}>
      <Text style={styles.ratingLabel}>{label}</Text>
      <View style={styles.ratingStars}>
        {[1, 2, 3, 4, 5].map((star) => (
          <TouchableOpacity
            key={star}
            onPress={() => onChange(star)}
            style={styles.starBtn}
          >
            <Ionicons
              name={star <= value ? 'star' : 'star-outline'}
              size={28}
              color={THEME.primary}
            />
          </TouchableOpacity>
        ))}
      </View>
    </View>
  );
};

export default function CreateReviewScreen() {
  const router = useRouter();
  const { orderId } = useLocalSearchParams<{ orderId: string }>();
  const { address } = useWalletStore();
  const { getOrder } = useOrders();
  const { getProvider } = useMarketApi();
  const { prepareSubmitReview, loading: reviewLoading } = useReviews();
  const { submitReview, txState, isProcessing, resetState } = useChainTransaction();
  const { execute, isLoading } = useAsync();

  const [order, setOrder] = useState<Order | null>(null);
  const [provider, setProvider] = useState<Provider | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);

  // 评分状态
  const [overallRating, setOverallRating] = useState(5);
  const [accuracyRating, setAccuracyRating] = useState(5);
  const [attitudeRating, setAttitudeRating] = useState(5);
  const [responseRating, setResponseRating] = useState(5);
  const [content, setContent] = useState('');
  const [isAnonymous, setIsAnonymous] = useState(false);

  const loadData = useCallback(async () => {
    if (!orderId) return;

    await execute(async () => {
      const orderData = await getOrder(parseInt(orderId, 10));
      setOrder(orderData);

      if (orderData?.provider) {
        const providerData = await getProvider(orderData.provider);
        setProvider(providerData);
      }
    });
  }, [orderId, getOrder, getProvider, execute]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleSubmit = async () => {
    if (!order || !address) {
      Alert.alert('提示', '请先连接钱包');
      return;
    }

    if (overallRating === 0) {
      Alert.alert('提示', '请给出总体评分');
      return;
    }

    setSubmitting(true);
    setShowTxStatus(true);

    try {
      // 准备评价数据
      const reviewData = await prepareSubmitReview({
        orderId: order.id,
        overallRating: overallRating * 100,
        accuracyRating: accuracyRating * 100,
        attitudeRating: attitudeRating * 100,
        responseRating: responseRating * 100,
        content: content.trim() || undefined,
        isAnonymous,
      });

      console.log('Submit review:', reviewData);

      // 调用链上交易提交评价
      const result = await submitReview(
        {
          orderId: order.id,
          ratings: {
            accuracy: accuracyRating,
            attitude: attitudeRating,
            speed: responseRating,
            value: overallRating,
          },
          contentCid: reviewData.contentCid,
          isAnonymous,
        },
        {
          onSuccess: () => {
            setTimeout(() => {
              setShowTxStatus(false);
              resetState();
              Alert.alert('成功', '评价提交成功', [
                { text: '确定', onPress: () => router.back() },
              ]);
            }, 1500);
          },
          onError: (error) => {
            console.error('Submit review error:', error);
          },
        }
      );

      if (!result) {
        setShowTxStatus(false);
      }
    } catch (err) {
      console.error('Submit review error:', err);
      Alert.alert('失败', '提交评价失败');
      setShowTxStatus(false);
    } finally {
      setSubmitting(false);
    }
  };

  const handleTxStatusClose = () => {
    setShowTxStatus(false);
    resetState();
  };

  if (isLoading) {
    return (
      <SafeAreaView style={styles.container}>
        <LoadingSpinner text="加载中..." fullScreen />
      </SafeAreaView>
    );
  }

  if (!order || !provider) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.header}>
          <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
            <Ionicons name="arrow-back" size={24} color={THEME.text} />
          </TouchableOpacity>
          <Text style={styles.headerTitle}>评价订单</Text>
          <View style={styles.backBtn} />
        </View>
        <View style={styles.centerContent}>
          <Ionicons name="alert-circle-outline" size={64} color={THEME.border} />
          <Text style={styles.emptyText}>订单不存在或无法评价</Text>
        </View>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.card} />

      {/* 顶部导航 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
          <Ionicons name="arrow-back" size={24} color={THEME.text} />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>评价订单</Text>
        <View style={styles.backBtn} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 解卦师信息 */}
        <Card style={styles.section}>
          <View style={styles.providerRow}>
            <Avatar
              uri={provider.avatarCid ? getIpfsUrl(provider.avatarCid) : undefined}
              name={provider.name}
              size={52}
            />
            <View style={styles.providerInfo}>
              <View style={styles.nameRow}>
                <Text style={styles.providerName}>{provider.name}</Text>
                <TierBadge tier={provider.tier} size="small" />
              </View>
              <View style={styles.orderInfo}>
                <DivinationTypeBadge type={order.divinationType} size="small" />
                <Text style={styles.orderId}>订单: {order.id}</Text>
              </View>
            </View>
          </View>
        </Card>

        {/* 评分 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>服务评分</Text>

          <RatingItem
            label="总体评价"
            value={overallRating}
            onChange={setOverallRating}
          />
          <RatingItem
            label="准确性"
            value={accuracyRating}
            onChange={setAccuracyRating}
          />
          <RatingItem
            label="服务态度"
            value={attitudeRating}
            onChange={setAttitudeRating}
          />
          <RatingItem
            label="响应速度"
            value={responseRating}
            onChange={setResponseRating}
          />
        </Card>

        {/* 评价内容 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>评价内容（选填）</Text>
          <TextInput
            style={styles.textArea}
            placeholder="分享您的体验，帮助其他用户做出选择..."
            placeholderTextColor={THEME.textTertiary}
            value={content}
            onChangeText={setContent}
            multiline
            numberOfLines={5}
            textAlignVertical="top"
            maxLength={500}
          />
          <Text style={styles.charCount}>{content.length}/500</Text>
        </Card>

        {/* 匿名选项 */}
        <Card style={styles.section}>
          <View style={styles.anonymousRow}>
            <View style={styles.anonymousInfo}>
              <Text style={styles.anonymousLabel}>匿名评价</Text>
              <Text style={styles.anonymousDesc}>
                开启后将隐藏您的地址
              </Text>
            </View>
            <Switch
              value={isAnonymous}
              onValueChange={setIsAnonymous}
              trackColor={{ false: THEME.border, true: THEME.primary + '60' }}
              thumbColor={isAnonymous ? THEME.primary : THEME.textTertiary}
            />
          </View>
        </Card>

        {/* 提交按钮 */}
        <Button
          title="提交评价"
          onPress={handleSubmit}
          loading={submitting}
          disabled={submitting}
        />

        <View style={styles.bottomSpace} />
      </ScrollView>

      {/* 交易状态弹窗 */}
      <TransactionStatus
        visible={showTxStatus}
        state={txState}
        onClose={handleTxStatusClose}
        successMessage="评价提交成功！"
      />
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
  centerContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    gap: 16,
  },
  emptyText: {
    fontSize: 15,
    color: THEME.textSecondary,
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
    marginBottom: 16,
  },
  providerRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  providerInfo: {
    flex: 1,
    marginLeft: 12,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  providerName: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.text,
  },
  orderInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
    marginTop: 6,
  },
  orderId: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  ratingItem: {
    marginBottom: 16,
  },
  ratingLabel: {
    fontSize: 14,
    color: THEME.textSecondary,
    marginBottom: 8,
  },
  ratingStars: {
    flexDirection: 'row',
    gap: 8,
  },
  starBtn: {
    padding: 4,
  },
  textArea: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
    color: THEME.text,
    height: 120,
  },
  charCount: {
    fontSize: 11,
    color: THEME.textTertiary,
    textAlign: 'right',
    marginTop: 4,
  },
  anonymousRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  anonymousInfo: {
    flex: 1,
    marginRight: 16,
  },
  anonymousLabel: {
    fontSize: 14,
    color: THEME.text,
    fontWeight: '500',
  },
  anonymousDesc: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 2,
  },
  submitBtn: {
    backgroundColor: THEME.primary,
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
    marginTop: 8,
  },
  submitBtnDisabled: {
    opacity: 0.6,
  },
  submitBtnText: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.textInverse,
  },
  bottomSpace: {
    height: 32,
  },
});
