/**
 * 通用占卜页面模板
 * 减少重复代码，统一UI风格
 */

import React, { ReactNode } from 'react';
import { View, Text, StyleSheet, ScrollView, Alert } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import { Card, Button, LoadingSpinner } from '@/components/common';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { useWallet, useAsync } from '@/hooks';

const THEME_COLOR = '#B2955D';

export interface DivinationTemplateProps {
  // 页面信息
  title: string;
  subtitle?: string;
  icon?: keyof typeof Ionicons.glyphMap;

  // 表单内容
  renderForm: () => ReactNode;

  // 结果内容
  result: any | null;
  renderResult: () => ReactNode;

  // 操作
  onCalculate: () => Promise<void>;
  onSaveToChain?: () => Promise<void>;
  onReset: () => void;

  // 状态
  isCalculating?: boolean;
  isSaving?: boolean;

  // 对话框
  showUnlockDialog?: boolean;
  showTxStatus?: boolean;
  txStatus?: string;
  onUnlockSuccess?: () => void;
  onCloseUnlock?: () => void;
  onCloseTxStatus?: () => void;

  // 底部导航
  activeTab?: 'index' | 'divination' | 'chat' | 'market' | 'profile';
}

export const DivinationTemplate: React.FC<DivinationTemplateProps> = ({
  title,
  subtitle,
  icon = 'calendar-outline',
  renderForm,
  result,
  renderResult,
  onCalculate,
  onSaveToChain,
  onReset,
  isCalculating = false,
  isSaving = false,
  showUnlockDialog = false,
  showTxStatus = false,
  txStatus = '处理中...',
  onUnlockSuccess,
  onCloseUnlock,
  onCloseTxStatus,
  activeTab = 'divination',
}) => {
  const router = useRouter();

  return (
    <View style={styles.container}>
      <PageHeader title={title} />

      <ScrollView
        style={styles.scrollView}
        contentContainerStyle={styles.content}
        showsVerticalScrollIndicator={false}
      >
        {/* 标题区域 */}
        <Card style={styles.headerCard}>
          <View style={styles.headerContent}>
            <Ionicons name={icon} size={32} color={THEME_COLOR} />
            <View style={styles.headerText}>
              <Text style={styles.headerTitle}>{title}</Text>
              {subtitle && <Text style={styles.headerSubtitle}>{subtitle}</Text>}
            </View>
          </View>
        </Card>

        {/* 表单或结果 */}
        {!result ? (
          <>
            {/* 输入表单 */}
            <Card>{renderForm()}</Card>

            {/* 计算按钮 */}
            <Button
              title={isCalculating ? '计算中...' : '开始占卜'}
              onPress={onCalculate}
              loading={isCalculating}
              disabled={isCalculating}
              style={styles.calculateButton}
            />

            {/* 提示 */}
            <View style={styles.tipCard}>
              <Ionicons name="information-circle-outline" size={20} color="#666" />
              <Text style={styles.tipText}>
                占卜结果将保存到区块链，永久可查
              </Text>
            </View>
          </>
        ) : (
          <>
            {/* 占卜结果 */}
            {renderResult()}

            {/* 操作按钮 */}
            <View style={styles.actionButtons}>
              <Button
                title="重新占卜"
                onPress={onReset}
                variant="outline"
                style={styles.actionButton}
              />
              {onSaveToChain && (
                <Button
                  title={isSaving ? '保存中...' : '保存到链上'}
                  onPress={onSaveToChain}
                  loading={isSaving}
                  disabled={isSaving}
                  style={styles.actionButton}
                />
              )}
            </View>

            {/* AI解读按钮（占位） */}
            <Button
              title="AI智能解读"
              onPress={() => Alert.alert('提示', 'AI解读功能即将上线')}
              variant="secondary"
              style={styles.aiButton}
            />

            {/* 查看历史 */}
            <Button
              title="查看历史记录"
              onPress={() => router.push('/divination/history' as any)}
              variant="text"
              style={styles.historyButton}
            />
          </>
        )}
      </ScrollView>

      <BottomNavBar activeTab={activeTab} />

      {/* 解锁钱包对话框 */}
      {onUnlockSuccess && onCloseUnlock && (
        <UnlockWalletDialog
          visible={showUnlockDialog}
          onUnlock={onUnlockSuccess}
          onCancel={onCloseUnlock}
        />
      )}

      {/* 交易状态对话框 */}
      {onCloseTxStatus && (
        <TransactionStatusDialog
          visible={showTxStatus}
          status={txStatus}
          title="处理中"
        />
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  scrollView: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  headerCard: {
    marginBottom: 16,
  },
  headerContent: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 16,
  },
  headerText: {
    flex: 1,
  },
  headerTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  headerSubtitle: {
    fontSize: 14,
    color: '#666',
  },
  calculateButton: {
    marginTop: 16,
  },
  tipCard: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginTop: 16,
    padding: 12,
    backgroundColor: '#FFF9F0',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: THEME_COLOR + '30',
  },
  tipText: {
    flex: 1,
    fontSize: 13,
    color: '#666',
    lineHeight: 18,
  },
  actionButtons: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 16,
  },
  actionButton: {
    flex: 1,
  },
  aiButton: {
    marginTop: 12,
  },
  historyButton: {
    marginTop: 8,
  },
});
