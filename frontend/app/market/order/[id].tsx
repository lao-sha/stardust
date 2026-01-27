// frontend/app/market/order/[id].tsx

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  RefreshControl,
  SafeAreaView,
  StatusBar,
  Alert,
  TextInput,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useWalletStore } from '@/stores/wallet.store';
import { useOrders, useMarketApi, useChainTransaction } from '@/divination/market/hooks';
import {
  Avatar,
  TierBadge,
  PriceDisplay,
  DivinationTypeBadge,
  OrderStatusBadge,
  OrderTimeline,
  LoadingSpinner,
  EmptyState,
  ActionButton,
} from '@/divination/market/components';
import { Card, Button, Input } from '@/components/common';
import { useAsync } from '@/hooks';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { Order, Provider, FollowUp } from '@/divination/market/types';
import { truncateAddress, formatDateTime } from '@/divination/market/utils/market.utils';
import { getIpfsUrl, uploadToIpfs } from '@/divination/market/services/ipfs.service';

export default function OrderDetailScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const { address } = useWalletStore();
  const { getOrder } = useOrders();
  const { getProvider } = useMarketApi();
  const {
    acceptOrder,
    rejectOrder,
    completeOrder,
    cancelOrder,
    requestRefund,
    submitFollowUp,
    replyFollowUp,
    isProcessing,
  } = useChainTransaction();
  const { execute, isLoading } = useAsync();

  const [order, setOrder] = useState<Order | null>(null);
  const [provider, setProvider] = useState<Provider | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [followUpQuestion, setFollowUpQuestion] = useState('');
  const [submittingFollowUp, setSubmittingFollowUp] = useState(false);
  const [answerContent, setAnswerContent] = useState('');

  const loadData = useCallback(async () => {
    if (!id) return;

    await execute(async () => {
      const orderData = await getOrder(parseInt(id, 10));
      setOrder(orderData);

      if (orderData?.provider) {
        const providerData = await getProvider(orderData.provider);
        setProvider(providerData);
      }
    });
  }, [id, getOrder, getProvider, execute]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  }, [loadData]);

  const isCustomer = order?.customer === address;
  const isProvider = order?.provider === address;
  
  const canFollowUp =
    order?.status === 'Completed' &&
    order.followUps &&
    order.followUps.length < (order as any).followUpCount;

  // --- 交易处理函数 ---

  const handleAcceptOrder = async () => {
    if (!order) return;
    await acceptOrder(order.id, {
      onSuccess: () => {
        Alert.alert('成功', '已接单');
        loadData();
      },
    });
  };

  const handleRejectOrder = async () => {
    if (!order) return;
    Alert.prompt('拒绝订单', '请输入拒绝原因', [
      { text: '取消', style: 'cancel' },
      {
        text: '确定',
        onPress: async (reason) => {
          await rejectOrder(order.id, reason, {
            onSuccess: () => {
              Alert.alert('已拒绝', '订单已拒绝并退款');
              loadData();
            },
          });
        },
      },
    ]);
  };

  const handleCancelOrder = async () => {
    if (!order) return;
    Alert.alert('取消订单', '确定要取消订单吗？', [
      { text: '返回', style: 'cancel' },
      {
        text: '确定取消',
        style: 'destructive',
        onPress: async () => {
          await cancelOrder(order.id, {
            onSuccess: () => {
              Alert.alert('已取消', '订单已成功取消');
              loadData();
            },
          });
        },
      },
    ]);
  };

  const handleDispute = async () => {
    if (!order) return;
    Alert.prompt('发起举报', '请简述举报原因，平台介入后将根据证据处理。注意：举报需要缴纳一定押金，恶意举报将没收押金并扣除信用分。', [
      { text: '取消', style: 'cancel' },
      {
        text: '提交举报',
        onPress: async (description) => {
          if (!description) return;
          // 这里应该调用新的 submitReport 方法
          // 暂时沿用 requestRefund 的结构，之后在 hook 中适配
          await requestRefund(order.id, description, {
            onSuccess: () => {
              Alert.alert('已提交', '举报已发起，请等待平台处理');
              loadData();
            },
          });
        },
      },
    ]);
  };

  const handleTip = async () => {
    if (!order) return;
    Alert.prompt('额外打赏', '输入打赏金额 (DUST)', [
      { text: '取消', style: 'cancel' },
      {
        text: '确认打赏',
        onPress: async (amountStr) => {
          const amount = parseFloat(amountStr || '0');
          if (isNaN(amount) || amount <= 0) {
            Alert.alert('错误', '请输入有效的打赏金额');
            return;
          }
          const amountBigInt = BigInt(Math.floor(amount * 1000000));
          await tip({
            providerId: order.provider,
            amount: amountBigInt,
            orderId: order.id
          }, {
            onSuccess: () => {
              Alert.alert('感谢', '打赏已发送，感谢您的支持！');
            }
          });
        }
      }
    ]);
  };

  const handleConfirmOrder = async () => {
    if (!order) return;
    Alert.alert('确认订单', '订单已完成，您可以对此次服务进行评价。如有问题，可以发起举报。', [
      { text: '返回', style: 'cancel' },
      {
        text: '去评价',
        onPress: handleReview,
      },
    ]);
  };

  const handleSubmitAnswer = async () => {
    if (!order || !answerContent.trim()) {
      Alert.alert('提示', '请输入解答内容');
      return;
    }

    try {
      setRefreshing(true);
      // 1. 上传到 IPFS
      const { cid } = await uploadToIpfs(answerContent);
      
      // 2. 提交到链上
      await completeOrder(
        { orderId: order.id, resultCid: cid },
        {
          onSuccess: () => {
            Alert.alert('成功', '解答已提交');
            setAnswerContent('');
            loadData();
          },
        }
      );
    } catch (err) {
      Alert.alert('失败', '提交失败: ' + (err instanceof Error ? err.message : '未知错误'));
    } finally {
      setRefreshing(false);
    }
  };

  const handleSubmitFollowUp = async () => {
    if (!followUpQuestion.trim() || !order) {
      Alert.alert('提示', '请输入追问内容');
      return;
    }

    setSubmittingFollowUp(true);
    try {
      const { cid } = await uploadToIpfs(followUpQuestion);
      await submitFollowUp(
        { orderId: order.id, questionCid: cid },
        {
          onSuccess: () => {
            Alert.alert('成功', '追问已提交');
            setFollowUpQuestion('');
            loadData();
          },
        }
      );
    } catch (err) {
      Alert.alert('失败', '提交追问失败');
    } finally {
      setSubmittingFollowUp(false);
    }
  };

  const handleReview = () => {
    router.push(`/market/review/create?orderId=${id}`);
  };

  if (isLoading && !order) {
    return (
      <SafeAreaView style={styles.container}>
        <LoadingSpinner text="加载中..." fullScreen />
      </SafeAreaView>
    );
  }

  if (!order) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.header}>
          <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
            <Ionicons name="arrow-back" size={24} color={THEME.text} />
          </TouchableOpacity>
          <Text style={styles.headerTitle}>订单详情</Text>
          <View style={styles.backBtn} />
        </View>
        <EmptyState
          icon="document-outline"
          title="订单不存在"
          actionText="返回"
          onAction={() => router.back()}
        />
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
        <Text style={styles.headerTitle}>订单详情</Text>
        <View style={styles.backBtn} />
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl
            refreshing={refreshing || isProcessing}
            onRefresh={onRefresh}
            colors={[THEME.primary]}
            tintColor={THEME.primary}
          />
        }
      >
        {/* 订单状态卡片 */}
        <Card style={styles.statusCard}>
          <View style={styles.statusHeader}>
            <OrderStatusBadge status={order.status} size="medium" />
            {order.isUrgent && (
              <View style={styles.urgentTag}>
                <Ionicons name="flash" size={12} color={THEME.warning} />
                <Text style={styles.urgentText}>加急</Text>
              </View>
            )}
          </View>
          <Text style={styles.orderId}>订单号: {order.id}</Text>
        </Card>

        {/* 解卦师/客户信息 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>
            {isCustomer ? '解卦师' : '客户'}
          </Text>
          <View style={styles.personRow}>
            {isCustomer && provider ? (
              <>
                <Avatar
                  uri={provider.avatarCid ? getIpfsUrl(provider.avatarCid) : undefined}
                  name={provider.name}
                  size={44}
                />
                <View style={styles.personInfo}>
                  <View style={styles.nameRow}>
                    <Text style={styles.personName}>{provider.name}</Text>
                    <TierBadge tier={provider.tier} size="small" />
                  </View>
                  <Text style={styles.personOrders}>
                    已完成 {provider.completedOrders} 单
                  </Text>
                </View>
              </>
            ) : (
              <>
                <Avatar name="客" size={44} />
                <View style={styles.personInfo}>
                  <Text style={styles.personName}>
                    {truncateAddress(order.customer)}
                  </Text>
                </View>
              </>
            )}
          </View>
        </Card>

        {/* 套餐信息 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>服务信息</Text>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>占卜类型</Text>
            <DivinationTypeBadge type={order.divinationType} />
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>订单金额</Text>
            <PriceDisplay amount={order.amount} size="small" />
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>创建时间</Text>
            <Text style={styles.infoValue}>{formatDateTime(order.createdAt)}</Text>
          </View>
        </Card>

        {/* 问题描述 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>问题描述</Text>
          <View style={styles.questionBox}>
            <Text style={styles.questionText}>
              {order.question || '问题内容加密存储在链上'}
            </Text>
          </View>
        </Card>

        {/* 提供者输入解答 (仅限进行中且我是提供者) */}
        {isProvider && order.status === 'Accepted' && (
          <Card style={styles.section}>
            <Text style={styles.sectionTitle}>提交解答</Text>
            <TextInput
              style={styles.answerInput}
              placeholder="请在这里输入您的详细解读..."
              placeholderTextColor={THEME.textTertiary}
              value={answerContent}
              onChangeText={setAnswerContent}
              multiline
              numberOfLines={6}
              textAlignVertical="top"
            />
            <TouchableOpacity
              style={[styles.submitBtn, (!answerContent.trim() || isProcessing) && styles.btnDisabled]}
              onPress={handleSubmitAnswer}
              disabled={!answerContent.trim() || isProcessing}
            >
              <Text style={styles.submitBtnText}>提交解答并完成订单</Text>
            </TouchableOpacity>
          </Card>
        )}

        {/* 解读结果 */}
        {order.answerCid && (
          <Card style={styles.section}>
            <Text style={styles.sectionTitle}>解读结果</Text>
            <View style={styles.answerBox}>
              <Text style={styles.answerText}>
                {order.answer || '解读结果已上链存储'}
              </Text>
            </View>
          </Card>
        )}

        {/* 追问列表 */}
        {order.followUps && order.followUps.length > 0 && (
          <Card style={styles.section}>
            <Text style={styles.sectionTitle}>
              追问记录 ({order.followUps.length})
            </Text>
            {order.followUps.map((followUp, index) => (
              <View key={index} style={styles.followUpItem}>
                <View style={styles.followUpQuestion}>
                  <Ionicons name="chatbubble-outline" size={14} color={THEME.info} />
                  <Text style={styles.followUpQuestionText}>
                    {followUp.question || '追问内容已上链'}
                  </Text>
                </View>
                {followUp.answerCid && (
                  <View style={styles.followUpAnswer}>
                    <Ionicons name="chatbubble" size={14} color={THEME.primary} />
                    <Text style={styles.followUpAnswerText}>
                      {followUp.answer || '追问解答已上链'}
                    </Text>
                  </View>
                )}
                {isProvider && !followUp.answerCid && (
                  <TouchableOpacity 
                    style={styles.replyBtn}
                    onPress={() => {
                      Alert.prompt('回复追问', '请输入您的解答', [
                        { text: '取消', style: 'cancel' },
                        {
                          text: '提交',
                          onPress: async (answer) => {
                            if (!answer) return;
                            const { cid } = await uploadToIpfs(answer);
                            await replyFollowUp({
                              orderId: order.id,
                              followUpIndex: index,
                              answerCid: cid
                            }, {
                              onSuccess: () => {
                                Alert.alert('成功', '追问已回复');
                                loadData();
                              }
                            });
                          }
                        }
                      ]);
                    }}
                  >
                    <Text style={styles.replyBtnText}>回复此追问</Text>
                  </TouchableOpacity>
                )}
              </View>
            ))}
          </Card>
        )}

        {/* 追问输入 */}
        {isCustomer && canFollowUp && (
          <Card style={styles.section}>
            <Text style={styles.sectionTitle}>追问</Text>
            <TextInput
              style={styles.followUpInput}
              placeholder="输入您的追问..."
              placeholderTextColor={THEME.textTertiary}
              value={followUpQuestion}
              onChangeText={setFollowUpQuestion}
              multiline
              numberOfLines={3}
              textAlignVertical="top"
            />
            <TouchableOpacity
              style={[styles.followUpBtn, (submittingFollowUp || isProcessing) && styles.btnDisabled]}
              onPress={handleSubmitFollowUp}
              disabled={submittingFollowUp || isProcessing}
            >
              <Text style={styles.followUpBtnText}>
                {submittingFollowUp ? '提交中...' : '提交追问'}
              </Text>
            </TouchableOpacity>
          </Card>
        )}

        {/* 订单时间线 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>订单进度</Text>
          <OrderTimeline order={order} />
        </Card>

        <View style={styles.bottomSpace} />
      </ScrollView>

      {/* 底部操作栏 - 根据身份和状态动态显示 */}
      <View style={styles.footer}>
        {/* 提供者操作 */}
        {isProvider && order.status === 'Paid' && (
          <View style={styles.actionRow}>
            <TouchableOpacity style={[styles.actionBtn, styles.rejectBtn]} onPress={handleRejectOrder} disabled={isProcessing}>
              <Text style={styles.rejectBtnText}>拒绝接单</Text>
            </TouchableOpacity>
            <TouchableOpacity style={[styles.actionBtn, styles.primaryBtn]} onPress={handleAcceptOrder} disabled={isProcessing}>
              <Text style={styles.primaryBtnText}>立即接单</Text>
            </TouchableOpacity>
          </View>
        )}

        {/* 客户操作 */}
        {isCustomer && (
          <View style={styles.actionRow}>
            {order.status === 'Paid' && (
              <TouchableOpacity style={[styles.actionBtn, styles.outlineBtn]} onPress={handleCancelOrder} disabled={isProcessing}>
                <Text style={styles.outlineBtnText}>取消订单</Text>
              </TouchableOpacity>
            )}
            {order.status === 'Accepted' && (
              <TouchableOpacity style={[styles.actionBtn, styles.outlineBtn]} onPress={handleDispute} disabled={isProcessing}>
                <Text style={styles.outlineBtnText}>发起争议</Text>
              </TouchableOpacity>
            )}
            {order.status === 'Completed' && (
              <>
                <TouchableOpacity style={[styles.actionBtn, styles.outlineBtn, { flex: 0.6 }]} onPress={handleDispute} disabled={isProcessing}>
                  <Text style={[styles.outlineBtnText, { fontSize: 14 }]}>举报</Text>
                </TouchableOpacity>
                <TouchableOpacity style={[styles.actionBtn, styles.outlineBtn, { flex: 0.6 }]} onPress={handleTip} disabled={isProcessing}>
                  <Text style={[styles.outlineBtnText, { fontSize: 14 }]}>打赏</Text>
                </TouchableOpacity>
                <TouchableOpacity style={[styles.actionBtn, styles.primaryBtn]} onPress={handleReview} disabled={isProcessing}>
                  <Ionicons name="star-outline" size={18} color={THEME.textInverse} />
                  <Text style={styles.primaryBtnText}>评价</Text>
                </TouchableOpacity>
              </>
            )}
          </View>
        )}
      </View>
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
  statusCard: {
    marginBottom: 16,
  },
  statusHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
  },
  urgentTag: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.warning + '15',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
    gap: 2,
  },
  urgentText: {
    fontSize: 11,
    color: THEME.warning,
    fontWeight: '500',
  },
  orderId: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 8,
  },
  section: {
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 12,
  },
  personRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  personInfo: {
    flex: 1,
    marginLeft: 12,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  personName: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
  },
  personOrders: {
    fontSize: 12,
    color: THEME.textSecondary,
    marginTop: 4,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.borderLight,
  },
  infoLabel: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  infoValue: {
    fontSize: 14,
    color: THEME.text,
  },
  questionBox: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    padding: 12,
  },
  questionText: {
    fontSize: 14,
    color: THEME.text,
    lineHeight: 20,
  },
  answerInput: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    padding: 12,
    fontSize: 14,
    color: THEME.text,
    minHeight: 120,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  submitBtn: {
    backgroundColor: THEME.success,
    borderRadius: 8,
    paddingVertical: 12,
    alignItems: 'center',
  },
  submitBtnText: {
    color: THEME.textInverse,
    fontSize: 15,
    fontWeight: '600',
  },
  answerBox: {
    backgroundColor: THEME.primary + '10',
    borderRadius: 8,
    padding: 12,
    borderLeftWidth: 3,
    borderLeftColor: THEME.primary,
  },
  answerText: {
    fontSize: 14,
    color: THEME.text,
    lineHeight: 22,
  },
  followUpItem: {
    marginBottom: 12,
    paddingBottom: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.borderLight,
  },
  followUpQuestion: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 8,
  },
  followUpQuestionText: {
    flex: 1,
    fontSize: 13,
    color: THEME.info,
    lineHeight: 18,
  },
  followUpAnswer: {
    flexDirection: 'row',
    gap: 8,
    marginLeft: 22,
    marginBottom: 8,
  },
  followUpAnswerText: {
    flex: 1,
    fontSize: 13,
    color: THEME.text,
    lineHeight: 18,
  },
  replyBtn: {
    alignSelf: 'flex-end',
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 14,
    backgroundColor: THEME.primary + '15',
  },
  replyBtnText: {
    fontSize: 12,
    color: THEME.primary,
    fontWeight: '500',
  },
  followUpInput: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
    color: THEME.text,
    height: 80,
    marginBottom: 12,
  },
  followUpBtn: {
    backgroundColor: THEME.primary,
    borderRadius: 8,
    paddingVertical: 10,
    alignItems: 'center',
  },
  btnDisabled: {
    opacity: 0.6,
  },
  followUpBtnText: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME.textInverse,
  },
  bottomSpace: {
    height: 100,
  },
  footer: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    backgroundColor: THEME.card,
    paddingHorizontal: 16,
    paddingVertical: 12,
    paddingBottom: 24,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: THEME.border,
    ...SHADOWS.medium,
  },
  actionRow: {
    flexDirection: 'row',
    gap: 12,
  },
  actionBtn: {
    flex: 1,
    flexDirection: 'row',
    height: 48,
    borderRadius: 10,
    justifyContent: 'center',
    alignItems: 'center',
    gap: 6,
  },
  primaryBtn: {
    backgroundColor: THEME.primary,
  },
  primaryBtnText: {
    color: THEME.textInverse,
    fontSize: 16,
    fontWeight: '600',
  },
  rejectBtn: {
    backgroundColor: THEME.error + '10',
    borderWidth: 1,
    borderColor: THEME.error,
  },
  rejectBtnText: {
    color: THEME.error,
    fontSize: 16,
    fontWeight: '600',
  },
  outlineBtn: {
    backgroundColor: THEME.card,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  outlineBtnText: {
    color: THEME.textSecondary,
    fontSize: 16,
    fontWeight: '600',
  },
});
