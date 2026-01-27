/**
 * 订单详情页面
 */

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  TextInput,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import {
  Order,
  OrderStatus,
  ORDER_STATUS_CONFIG,
  DIVINATION_TYPE_CONFIG,
  SERVICE_TYPE_CONFIG,
  DivinationType,
  ServiceType,
} from '@/features/diviner';
import { divinationMarketService } from '@/services/divination-market.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

export default function OrderDetailPage() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [order, setOrder] = useState<Order | null>(null);
  const [answerText, setAnswerText] = useState('');
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingAction, setPendingAction] = useState<'accept' | 'reject' | 'submit' | null>(null);

  const orderId = Number(id);

  const loadOrder = useCallback(async () => {
    try {
      const orderData = await divinationMarketService.getOrder(orderId);
      if (orderData) {
        // 转换为前端格式
        setOrder({
          id: orderData.id,
          customer: orderData.customer,
          provider: '', // 需要从 provider 查询
          packageId: orderData.packageId,
          divinationType: DivinationType.Meihua, // 需要从 package 查询
          questionCid: orderData.questionCid,
          answerCid: orderData.answerCid,
          totalAmount: orderData.amount,
          platformFee: orderData.amount / 10n, // 假设 10% 手续费
          providerEarnings: orderData.amount * 9n / 10n,
          isUrgent: false,
          status: orderData.status as unknown as OrderStatus,
          createdAt: orderData.createdAt,
          completedAt: orderData.completedAt,
          followUpsUsed: 0,
          followUpsTotal: 3,
        });
      }
    } catch (error) {
      console.error('Load order error:', error);
    } finally {
      setLoading(false);
    }
  }, [orderId]);

  useEffect(() => {
    loadOrder();
  }, [loadOrder]);

  const handleAccept = () => {
    if (!isSignerUnlocked()) {
      setPendingAction('accept');
      setShowUnlockDialog(true);
      return;
    }
    executeAccept();
  };

  const executeAccept = async () => {
    setShowTxStatus(true);
    setTxStatus('正在接受订单...');

    try {
      await divinationMarketService.acceptOrder(orderId, (status) => setTxStatus(status));
      setTxStatus('接单成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        loadOrder();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('接单失败', error.message || '请稍后重试');
    }
  };

  const handleReject = () => {
    Alert.alert('确认拒绝', '拒绝后订单将取消并退款给客户', [
      { text: '取消', style: 'cancel' },
      {
        text: '确认拒绝',
        style: 'destructive',
        onPress: () => {
          if (!isSignerUnlocked()) {
            setPendingAction('reject');
            setShowUnlockDialog(true);
            return;
          }
          executeReject();
        },
      },
    ]);
  };

  const executeReject = async () => {
    setShowTxStatus(true);
    setTxStatus('正在拒绝订单...');

    try {
      await divinationMarketService.rejectOrder(orderId, '解卦师拒绝', (status) => setTxStatus(status));
      setTxStatus('拒绝成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        loadOrder();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('拒绝失败', error.message || '请稍后重试');
    }
  };

  const handleSubmitAnswer = () => {
    if (!answerText.trim()) {
      Alert.alert('提示', '请输入解读内容');
      return;
    }

    if (!isSignerUnlocked()) {
      setPendingAction('submit');
      setShowUnlockDialog(true);
      return;
    }
    executeSubmitAnswer();
  };

  const executeSubmitAnswer = async () => {
    setShowTxStatus(true);
    setTxStatus('正在上传解读内容...');

    try {
      // TODO: 先上传到 IPFS 获取 CID
      const answerCid = 'Qm' + Date.now().toString(36); // 临时 mock

      setTxStatus('正在提交解卦结果...');
      await divinationMarketService.submitInterpretation(orderId, answerCid, (status) => setTxStatus(status));

      setTxStatus('提交成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        setAnswerText('');
        loadOrder();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('提交失败', error.message || '请稍后重试');
    }
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      if (pendingAction === 'accept') {
        await executeAccept();
      } else if (pendingAction === 'reject') {
        await executeReject();
      } else if (pendingAction === 'submit') {
        await executeSubmitAnswer();
      }
      setPendingAction(null);
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="订单详情" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  if (!order) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="订单详情" />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>订单不存在</Text>
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  const statusConfig = ORDER_STATUS_CONFIG[order.status];
  const divType = DIVINATION_TYPE_CONFIG[order.divinationType];
  const priceDisplay = (Number(order.totalAmount) / 1e10).toFixed(2);
  const feeDisplay = (Number(order.platformFee) / 1e10).toFixed(2);
  const earningsDisplay = (Number(order.providerEarnings) / 1e10).toFixed(2);

  const formatTime = (timestamp?: number) => {
    if (!timestamp) return '-';
    const date = new Date(timestamp);
    return `${date.getFullYear()}/${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`;
  };

  return (
    <View style={styles.wrapper}>
      <PageHeader title="订单详情" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 状态卡片 */}
        <View style={[styles.statusCard, { backgroundColor: `${statusConfig.color}10` }]}>
          <Text style={[styles.statusText, { color: statusConfig.color }]}>{statusConfig.label}</Text>
          {order.isUrgent && <Text style={styles.urgentTag}>⚡ 加急订单</Text>}
        </View>

        {/* 订单信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>订单信息</Text>
          <View style={styles.infoCard}>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>订单编号</Text>
              <Text style={styles.infoValue}>#{order.id}</Text>
            </View>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>占卜类型</Text>
              <Text style={styles.infoValue}>{divType?.icon} {divType?.label}</Text>
            </View>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>创建时间</Text>
              <Text style={styles.infoValue}>{formatTime(order.createdAt)}</Text>
            </View>
            {order.acceptedAt && (
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>接单时间</Text>
                <Text style={styles.infoValue}>{formatTime(order.acceptedAt)}</Text>
              </View>
            )}
            {order.completedAt && (
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>完成时间</Text>
                <Text style={styles.infoValue}>{formatTime(order.completedAt)}</Text>
              </View>
            )}
          </View>
        </View>

        {/* 费用明细 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>费用明细</Text>
          <View style={styles.infoCard}>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>订单金额</Text>
              <Text style={styles.infoValue}>{priceDisplay} DUST</Text>
            </View>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>平台手续费</Text>
              <Text style={styles.infoValue}>-{feeDisplay} DUST</Text>
            </View>
            <View style={[styles.infoRow, styles.earningsRow]}>
              <Text style={styles.earningsLabel}>您的收益</Text>
              <Text style={styles.earningsValue}>{earningsDisplay} DUST</Text>
            </View>
          </View>
        </View>

        {/* 客户问题 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>客户问题</Text>
          <View style={styles.questionCard}>
            <Text style={styles.questionText}>问题内容加载中... (CID: {order.questionCid})</Text>
          </View>
        </View>

        {/* 追问信息 */}
        {order.followUpsTotal > 0 && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>追问</Text>
            <View style={styles.infoCard}>
              <Text style={styles.followUpText}>
                已使用 {order.followUpsUsed}/{order.followUpsTotal} 次追问
              </Text>
            </View>
          </View>
        )}

        {/* 提交解读 */}
        {order.status === OrderStatus.Accepted && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>提交解读</Text>
            <View style={styles.answerCard}>
              <TextInput
                style={styles.answerInput}
                value={answerText}
                onChangeText={setAnswerText}
                placeholder="请输入您的解读内容..."
                placeholderTextColor="#999"
                multiline
                numberOfLines={8}
                textAlignVertical="top"
              />
              <Pressable
                style={[styles.submitBtn, !answerText.trim() && styles.submitBtnDisabled]}
                onPress={handleSubmitAnswer}
                disabled={!answerText.trim() || submitting}
              >
                {submitting ? (
                  <ActivityIndicator color="#FFF" />
                ) : (
                  <Text style={styles.submitBtnText}>提交解读</Text>
                )}
              </Pressable>
            </View>
          </View>
        )}

        {/* 已完成的解读 */}
        {order.answerCid && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>解读内容</Text>
            <View style={styles.answerCard}>
              <Text style={styles.answerText}>解读内容加载中... (CID: {order.answerCid})</Text>
            </View>
          </View>
        )}

        {/* 操作按钮 */}
        {order.status === OrderStatus.Paid && (
          <View style={styles.actionSection}>
            <Pressable style={styles.rejectBtn} onPress={handleReject} disabled={submitting}>
              <Text style={styles.rejectBtnText}>拒绝订单</Text>
            </Pressable>
            <Pressable style={styles.acceptBtn} onPress={handleAccept} disabled={submitting}>
              {submitting ? (
                <ActivityIndicator color="#FFF" />
              ) : (
                <Text style={styles.acceptBtnText}>接受订单</Text>
              )}
            </Pressable>
          </View>
        )}
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
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
  },
  statusCard: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 16,
    margin: 16,
    borderRadius: 12,
    gap: 12,
  },
  statusText: {
    fontSize: 18,
    fontWeight: '600',
  },
  urgentTag: {
    fontSize: 14,
    color: '#FF9500',
  },
  section: {
    paddingHorizontal: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000',
    marginBottom: 12,
  },
  infoCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  infoLabel: {
    fontSize: 14,
    color: '#666',
  },
  infoValue: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  earningsRow: {
    borderBottomWidth: 0,
    paddingTop: 12,
  },
  earningsLabel: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  earningsValue: {
    fontSize: 16,
    color: THEME_COLOR,
    fontWeight: '600',
  },
  questionCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  questionText: {
    fontSize: 14,
    color: '#333',
    lineHeight: 22,
  },
  followUpText: {
    fontSize: 14,
    color: '#666',
  },
  answerCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  answerInput: {
    height: 160,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    padding: 12,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FAFAFA',
    marginBottom: 12,
  },
  answerText: {
    fontSize: 14,
    color: '#333',
    lineHeight: 22,
  },
  submitBtn: {
    height: 44,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  submitBtnDisabled: {
    opacity: 0.5,
  },
  submitBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
  actionSection: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
  },
  rejectBtn: {
    flex: 1,
    height: 48,
    borderWidth: 1,
    borderColor: '#FF3B30',
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  rejectBtnText: {
    fontSize: 16,
    color: '#FF3B30',
  },
  acceptBtn: {
    flex: 1,
    height: 48,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  acceptBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
});
