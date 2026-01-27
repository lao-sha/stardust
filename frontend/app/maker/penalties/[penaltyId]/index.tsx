/**
 * 扣除详情页面
 * 路径: /maker/penalties/[penaltyId]
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useMakerStore } from '@/stores/maker.store';
import { MakerService, PenaltyRecord } from '@/services/maker.service';
import { PageHeader } from '@/components/PageHeader';
import { Card, LoadingSpinner, Button } from '@/components/common';

export default function PenaltyDetailPage() {
  const router = useRouter();
  const { penaltyId } = useLocalSearchParams<{ penaltyId: string }>();
  const { penalties, fetchPenalties } = useMakerStore();

  const [penalty, setPenalty] = useState<PenaltyRecord | null>(null);

  useEffect(() => {
    fetchPenalties();
  }, []);

  useEffect(() => {
    if (penaltyId && penalties.length > 0) {
      const found = penalties.find((p) => p.id === parseInt(penaltyId));
      setPenalty(found || null);
    }
  }, [penaltyId, penalties]);

  if (!penalty) {
    return (
      <View style={styles.loadingContainer}>
        <LoadingSpinner text="加载中..." />
      </View>
    );
  }

  const typeText = MakerService.getPenaltyTypeText(penalty.penaltyType);

  // 计算申诉截止时间
  const appealDeadline = new Date((penalty.deductedAt + 7 * 24 * 3600) * 1000);
  const now = new Date();
  const canAppeal = !penalty.appealed && now < appealDeadline;
  const daysLeft = Math.ceil((appealDeadline.getTime() - now.getTime()) / (24 * 3600 * 1000));

  const getAppealStatus = () => {
    if (!penalty.appealed) {
      return { text: '未申诉', color: '#8E8E93', bgColor: '#8E8E9320' };
    }
    if (penalty.appealResult === undefined) {
      return { text: '申诉中', color: '#007AFF', bgColor: '#007AFF20' };
    }
    if (penalty.appealResult) {
      return { text: '申诉成功', color: '#4CD964', bgColor: '#4CD96420' };
    }
    return { text: '申诉驳回', color: '#FF3B30', bgColor: '#FF3B3020' };
  };

  const appealStatus = getAppealStatus();

  // 获取扣除原因详情
  const getReasonDetail = () => {
    switch (penalty.penaltyType.type) {
      case 'OtcTimeout':
        return `买家已付款超过 ${penalty.penaltyType.timeoutHours} 小时，做市商未及时释放 DUST，触发超时扣除机制。`;
      case 'BridgeTimeout':
        return `Bridge 兑换请求超过 ${penalty.penaltyType.timeoutHours} 小时未处理，触发超时扣除机制。`;
      case 'ArbitrationLoss':
        return `争议仲裁案件 #${penalty.penaltyType.caseId} 判定做市商败诉，扣除相应金额作为赔偿。`;
      case 'LowCreditScore':
        return `信用分连续 ${penalty.penaltyType.daysBelowThreshold} 天低于阈值 (当前: ${penalty.penaltyType.currentScore})，触发信用扣除机制。`;
      case 'MaliciousBehavior':
        return `检测到恶意行为 (类型: ${penalty.penaltyType.behaviorType})，根据平台规则进行扣除。`;
      default:
        return '未知原因';
    }
  };

  return (
    <View style={styles.container}>
      <PageHeader title="扣除详情" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 基本信息 */}
        <Card style={styles.section}>
          <View style={styles.headerRow}>
            <View>
              <Text style={styles.penaltyId}>扣除编号: #P{penalty.id}</Text>
              <Text style={styles.penaltyType}>类型: {typeText}</Text>
            </View>
            <View style={[styles.statusBadge, { backgroundColor: appealStatus.bgColor }]}>
              <Text style={[styles.statusText, { color: appealStatus.color }]}>
                {appealStatus.text}
              </Text>
            </View>
          </View>
        </Card>

        {/* 扣除金额 */}
        <Card style={styles.section}>
          <Text style={styles.cardTitle}>扣除金额</Text>
          <Text style={styles.amountDust}>
            {MakerService.formatDustAmount(penalty.deductedAmount)} DUST
          </Text>
          <Text style={styles.amountUsd}>
            ≈ ${MakerService.formatUsdAmount(penalty.usdValue)} USD
          </Text>
          <View style={styles.timeRow}>
            <Text style={styles.timeLabel}>扣除时间</Text>
            <Text style={styles.timeValue}>
              {new Date(penalty.deductedAt * 1000).toLocaleString('zh-CN')}
            </Text>
          </View>
        </Card>

        {/* 关联信息 */}
        <Card style={styles.section}>
          <Text style={styles.cardTitle}>关联信息</Text>
          {penalty.penaltyType.type === 'OtcTimeout' && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>订单号</Text>
              <Text style={styles.infoValue}>#{penalty.penaltyType.orderId}</Text>
            </View>
          )}
          {penalty.penaltyType.type === 'BridgeTimeout' && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>兑换号</Text>
              <Text style={styles.infoValue}>#{penalty.penaltyType.swapId}</Text>
            </View>
          )}
          {penalty.penaltyType.type === 'ArbitrationLoss' && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>案件号</Text>
              <Text style={styles.infoValue}>#{penalty.penaltyType.caseId}</Text>
            </View>
          )}
          {penalty.beneficiary && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>受益人</Text>
              <Text style={styles.infoValue} numberOfLines={1}>
                {penalty.beneficiary.slice(0, 10)}...{penalty.beneficiary.slice(-8)}
              </Text>
            </View>
          )}
        </Card>

        {/* 扣除原因 */}
        <Card style={[styles.section, styles.infoCard]}>
          <Text style={styles.infoIcon}>💡</Text>
          <Text style={styles.infoTitle}>扣除原因</Text>
          <Text style={styles.infoDesc}>{getReasonDetail()}</Text>
        </Card>

        {/* 申诉信息 */}
        {canAppeal && (
          <View style={styles.appealInfo}>
            <Text style={styles.appealDeadline}>
              申诉截止: {appealDeadline.toLocaleString('zh-CN')} ({daysLeft}天后)
            </Text>
            <Button
              title="发起申诉"
              onPress={() => router.push(`/maker/penalties/${penalty.id}/appeal`)}
            />
          </View>
        )}

        {penalty.appealed && penalty.appealResult === undefined && (
          <View style={styles.appealingCard}>
            <Text style={styles.appealingIcon}>⏳</Text>
            <Text style={styles.appealingText}>申诉审核中，请耐心等待</Text>
          </View>
        )}

        {penalty.appealed && penalty.appealResult === true && (
          <View style={styles.appealSuccessCard}>
            <Text style={styles.appealSuccessIcon}>✅</Text>
            <Text style={styles.appealSuccessText}>
              申诉成功，扣除金额已退还
            </Text>
          </View>
        )}

        {penalty.appealed && penalty.appealResult === false && (
          <View style={styles.appealFailCard}>
            <Text style={styles.appealFailIcon}>❌</Text>
            <Text style={styles.appealFailText}>
              申诉被驳回，扣除维持原判
            </Text>
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
    marginBottom: 16,
  },
  headerRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
  },
  penaltyId: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 4,
  },
  penaltyType: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1C1C1E',
  },
  statusBadge: {
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 6,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  cardTitle: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 12,
  },
  amountDust: {
    fontSize: 28,
    fontWeight: '700',
    color: '#FF3B30',
    marginBottom: 4,
  },
  amountUsd: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 16,
  },
  timeRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#F2F2F7',
  },
  timeLabel: {
    fontSize: 14,
    color: '#8E8E93',
  },
  timeValue: {
    fontSize: 14,
    color: '#1C1C1E',
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  infoLabel: {
    fontSize: 14,
    color: '#8E8E93',
  },
  infoValue: {
    fontSize: 14,
    color: '#1C1C1E',
    fontWeight: '500',
    maxWidth: 200,
  },
  infoCard: {
    backgroundColor: '#FFF9E6',
  },
  infoIcon: {
    fontSize: 20,
    marginBottom: 8,
  },
  infoTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 8,
  },
  infoDesc: {
    fontSize: 14,
    color: '#666666',
    lineHeight: 20,
  },
  appealInfo: {
    marginBottom: 32,
  },
  appealDeadline: {
    fontSize: 14,
    color: '#FF9500',
    textAlign: 'center',
    marginBottom: 12,
  },
  appealingCard: {
    backgroundColor: '#007AFF20',
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 32,
  },
  appealingIcon: {
    fontSize: 24,
    marginRight: 12,
  },
  appealingText: {
    fontSize: 14,
    color: '#007AFF',
  },
  appealSuccessCard: {
    backgroundColor: '#4CD96420',
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 32,
  },
  appealSuccessIcon: {
    fontSize: 24,
    marginRight: 12,
  },
  appealSuccessText: {
    fontSize: 14,
    color: '#4CD964',
  },
  appealFailCard: {
    backgroundColor: '#FF3B3020',
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 32,
  },
  appealFailIcon: {
    fontSize: 24,
    marginRight: 12,
  },
  appealFailText: {
    fontSize: 14,
    color: '#FF3B30',
  },
});
