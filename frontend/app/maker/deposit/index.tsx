/**
 * 押金管理页面
 * 路径: /maker/deposit
 */

import React, { useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore, selectHasPendingWithdrawal } from '@/stores/maker.store';
import { MakerService } from '@/services/maker.service';
import { DepositStatus, WithdrawalProgress } from '@/features/maker/components';
import { PageHeader } from '@/components/PageHeader';
import { LoadingSpinner, EmptyState } from '@/components/common';

export default function DepositManagePage() {
  const router = useRouter();
  const {
    makerApp,
    depositUsdValue,
    withdrawalRequest,
    penalties,
    isLoading,
    refreshAll,
    fetchWithdrawalRequest,
    fetchPenalties,
  } = useMakerStore();

  const hasPendingWithdrawal = useMakerStore(selectHasPendingWithdrawal);

  const [refreshing, setRefreshing] = React.useState(false);

  useEffect(() => {
    refreshAll();
  }, []);

  const onRefresh = async () => {
    setRefreshing(true);
    await refreshAll();
    setRefreshing(false);
  };

  if (isLoading && !makerApp) {
    return (
      <View style={styles.loadingContainer}>
        <LoadingSpinner text="加载中..." />
      </View>
    );
  }

  if (!makerApp) {
    return (
      <View style={styles.container}>
        <PageHeader title="押金管理" showBack />
        <EmptyState
          icon="alert-circle-outline"
          title="未找到做市商信息"
          description="请先申请成为做市商"
        />
      </View>
    );
  }

  // 获取最近的押金变动记录（从扣除记录中）
  const recentPenalties = penalties.slice(0, 5);

  return (
    <View style={styles.container}>
      <PageHeader title="押金管理" showBack />

      <ScrollView
        style={styles.content}
        showsVerticalScrollIndicator={false}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
      >
        {/* 押金概览 */}
        <DepositStatus
          depositAmount={makerApp.deposit}
          depositUsdValue={depositUsdValue}
          showProgress
        />

        {/* 操作按钮 */}
        <View style={styles.actionButtons}>
          <TouchableOpacity
            style={styles.actionButton}
            onPress={() => router.push('/maker/deposit/replenish')}
          >
            <Text style={styles.actionButtonText}>补充押金</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.actionButton, styles.actionButtonOutline]}
            onPress={() => router.push('/maker/deposit/withdraw')}
            disabled={hasPendingWithdrawal}
          >
            <Text style={[styles.actionButtonText, styles.actionButtonTextOutline]}>
              {hasPendingWithdrawal ? '提现处理中' : '申请提现'}
            </Text>
          </TouchableOpacity>
        </View>

        {/* 提现进度 */}
        {withdrawalRequest && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>提现进度</Text>
            <TouchableOpacity onPress={() => router.push('/maker/deposit/withdraw/status')}>
              <WithdrawalProgress request={withdrawalRequest} />
            </TouchableOpacity>
          </View>
        )}

        {/* 押金变动记录 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>押金变动记录</Text>
            {penalties.length > 0 && (
              <TouchableOpacity onPress={() => router.push('/maker/penalties')}>
                <Text style={styles.sectionAction}>查看全部 →</Text>
              </TouchableOpacity>
            )}
          </View>

          {recentPenalties.length === 0 ? (
            <View style={styles.emptyRecords}>
              <Text style={styles.emptyRecordsText}>暂无变动记录</Text>
            </View>
          ) : (
            <View style={styles.recordsList}>
              {recentPenalties.map((penalty) => (
                <TouchableOpacity
                  key={penalty.id}
                  style={styles.recordItem}
                  onPress={() => router.push(`/maker/penalties/${penalty.id}`)}
                >
                  <View style={styles.recordIcon}>
                    <Text style={styles.recordIconText}>📤</Text>
                  </View>
                  <View style={styles.recordContent}>
                    <Text style={styles.recordTitle}>
                      扣除 - {MakerService.getPenaltyTypeText(penalty.penaltyType)}
                    </Text>
                    <Text style={styles.recordTime}>
                      {new Date(penalty.deductedAt * 1000).toLocaleString('zh-CN', {
                        month: '2-digit',
                        day: '2-digit',
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </Text>
                  </View>
                  <View style={styles.recordAmount}>
                    <Text style={styles.recordAmountText}>
                      -{MakerService.formatDustAmount(penalty.deductedAmount)} DUST
                    </Text>
                  </View>
                </TouchableOpacity>
              ))}
            </View>
          )}
        </View>
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
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#8E8E93',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  actionButtons: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 16,
  },
  actionButton: {
    flex: 1,
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  actionButtonOutline: {
    backgroundColor: '#FFFFFF',
    borderWidth: 1,
    borderColor: '#B2955D',
  },
  actionButtonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  actionButtonTextOutline: {
    color: '#B2955D',
  },
  section: {
    marginTop: 24,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 12,
  },
  sectionAction: {
    fontSize: 14,
    color: '#B2955D',
  },
  emptyRecords: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 24,
    alignItems: 'center',
  },
  emptyRecordsText: {
    fontSize: 14,
    color: '#8E8E93',
  },
  recordsList: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    overflow: 'hidden',
  },
  recordItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#F2F2F7',
  },
  recordIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#FF3B3020',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  recordIconText: {
    fontSize: 18,
  },
  recordContent: {
    flex: 1,
  },
  recordTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  recordTime: {
    fontSize: 12,
    color: '#8E8E93',
  },
  recordAmount: {
    alignItems: 'flex-end',
  },
  recordAmountText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#FF3B30',
  },
});
