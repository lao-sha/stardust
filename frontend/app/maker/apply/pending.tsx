/**
 * 等待审核页面
 * 路径: /maker/apply/pending
 */

import React, { useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore } from '@/stores/maker.store';
import { ApplicationStatus } from '@/services/maker.service';
import { PageHeader } from '@/components/PageHeader';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { Card, Button } from '@/components/common';
import { useAsync } from '@/hooks';

export default function PendingPage() {
  const router = useRouter();
  const {
    makerApp,
    fetchMakerInfo,
    cancelApplication,
    txStatus,
    error,
    clearError,
  } = useMakerStore();
  const { execute, isLoading } = useAsync();

  const [showTxDialog, setShowTxDialog] = React.useState(false);

  useEffect(() => {
    fetchMakerInfo();

    // 定期刷新状态
    const interval = setInterval(fetchMakerInfo, 30000);
    return () => clearInterval(interval);
  }, []);

  // 如果审核通过，跳转到控制台
  useEffect(() => {
    if (makerApp?.status === ApplicationStatus.Active) {
      Alert.alert('审核通过', '恭喜您成为做市商！', [
        { text: '进入控制台', onPress: () => router.replace('/maker/dashboard') },
      ]);
    } else if (makerApp?.status === ApplicationStatus.Rejected) {
      Alert.alert('审核驳回', '您的申请已被驳回，押金将退还', [
        { text: '确定', onPress: () => router.replace('/maker') },
      ]);
    }
  }, [makerApp?.status]);

  const handleCancel = () => {
    Alert.alert(
      '取消申请',
      '确定要取消做市商申请吗？押金将退还到您的账户。',
      [
        { text: '再想想', style: 'cancel' },
        {
          text: '确定取消',
          style: 'destructive',
          onPress: async () => {
            setShowTxDialog(true);
            await execute(async () => {
              await cancelApplication();
              setTimeout(() => {
                setShowTxDialog(false);
                router.replace('/maker');
              }, 1500);
            });
          },
        },
      ]
    );
  };

  const handleCloseTxDialog = () => {
    setShowTxDialog(false);
    clearError();
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  if (!makerApp) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#B2955D" />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="申请做市商 (3/3)" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 状态图标 */}
        <View style={styles.statusContainer}>
          <Text style={styles.statusIcon}>⏳</Text>
          <Text style={styles.statusTitle}>等待审核中</Text>
        </View>

        {/* 申请信息 */}
        <Card style={styles.section}>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>申请编号</Text>
            <Text style={styles.infoValue}>#{makerApp.id}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>提交时间</Text>
            <Text style={styles.infoValue}>{formatDate(makerApp.createdAt)}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>预计审核</Text>
            <Text style={styles.infoValue}>24 小时内</Text>
          </View>
        </Card>

        {/* 审核流程 */}
        <Card style={styles.section}>
          <Text style={styles.cardTitle}>审核流程</Text>

          <View style={styles.timeline}>
            <View style={styles.timelineItem}>
              <View style={[styles.timelineDot, styles.dotCompleted]} />
              <Text style={styles.timelineText}>✅ 押金已锁定</Text>
            </View>

            <View style={styles.timelineLine} />

            <View style={styles.timelineItem}>
              <View style={[styles.timelineDot, styles.dotCompleted]} />
              <Text style={styles.timelineText}>✅ 资料已提交</Text>
            </View>

            <View style={styles.timelineLine} />

            <View style={styles.timelineItem}>
              <View style={[styles.timelineDot, styles.dotActive]} />
              <Text style={styles.timelineText}>⏳ 平台审核中</Text>
            </View>

            <View style={styles.timelineLine} />

            <View style={styles.timelineItem}>
              <View style={[styles.timelineDot, styles.dotPending]} />
              <Text style={[styles.timelineText, styles.textPending]}>○ 审核通过</Text>
            </View>
          </View>
        </Card>

        {/* 审核说明 */}
        <Card style={[styles.section, styles.infoCard]}>
          <Text style={styles.infoIcon}>💡</Text>
          <Text style={styles.infoTitle}>审核说明</Text>
          <View style={styles.infoList}>
            <Text style={styles.infoItem}>• 审核通过后即可开始服务</Text>
            <Text style={styles.infoItem}>• 审核驳回将退还押金</Text>
            <Text style={styles.infoItem}>• 如需取消可点击下方按钮</Text>
          </View>
        </Card>

        {/* 取消按钮 */}
        <Button
          title="取消申请"
          onPress={handleCancel}
          loading={isLoading}
          disabled={isLoading}
          variant="outline"
        />
      </ScrollView>

      {/* 交易状态弹窗 */}
      <TransactionStatusDialog
        visible={showTxDialog}
        status={txStatus || ''}
        error={error}
        onClose={handleCloseTxDialog}
      />
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
  statusContainer: {
    alignItems: 'center',
    marginVertical: 24,
  },
  statusIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  statusTitle: {
    fontSize: 22,
    fontWeight: '600',
    color: '#1C1C1E',
  },
  section: {
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 16,
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
  },
  timeline: {
    paddingLeft: 8,
  },
  timelineItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  timelineDot: {
    width: 12,
    height: 12,
    borderRadius: 6,
    marginRight: 12,
  },
  dotCompleted: {
    backgroundColor: '#4CD964',
  },
  dotActive: {
    backgroundColor: '#007AFF',
  },
  dotPending: {
    backgroundColor: '#E5E5EA',
  },
  timelineLine: {
    width: 2,
    height: 20,
    backgroundColor: '#E5E5EA',
    marginLeft: 5,
    marginVertical: 4,
  },
  timelineText: {
    fontSize: 14,
    color: '#1C1C1E',
  },
  textPending: {
    color: '#8E8E93',
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
    marginBottom: 12,
  },
  infoList: {
    gap: 6,
  },
  infoItem: {
    fontSize: 14,
    color: '#666666',
    lineHeight: 20,
  },
});
