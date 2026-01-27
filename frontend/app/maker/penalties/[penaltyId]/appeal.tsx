/**
 * 申诉页面
 * 路径: /maker/penalties/[penaltyId]/appeal
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  TouchableOpacity,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useMakerStore } from '@/stores/maker.store';
import { MakerService, PenaltyRecord } from '@/services/maker.service';
import { PageHeader } from '@/components/PageHeader';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { Card, Button, LoadingSpinner } from '@/components/common';
import { useAsync } from '@/hooks';

export default function AppealPage() {
  const router = useRouter();
  const { penaltyId } = useLocalSearchParams<{ penaltyId: string }>();
  const {
    penalties,
    appealPenalty,
    txStatus,
    error,
    clearError,
    fetchPenalties,
  } = useMakerStore();
  const { execute, isLoading } = useAsync();

  const [penalty, setPenalty] = useState<PenaltyRecord | null>(null);
  const [reason, setReason] = useState('');
  const [evidenceCid, setEvidenceCid] = useState('');
  const [showTxDialog, setShowTxDialog] = useState(false);

  useEffect(() => {
    fetchPenalties();
  }, []);

  useEffect(() => {
    if (penaltyId && penalties.length > 0) {
      const found = penalties.find((p) => p.id === parseInt(penaltyId));
      setPenalty(found || null);
    }
  }, [penaltyId, penalties]);

  const handleSubmit = async () => {
    if (!reason.trim()) {
      Alert.alert('请填写申诉理由');
      return;
    }

    if (!penalty) return;

    // 生成证据 CID（实际应该上传到 IPFS）
    const cid = evidenceCid || `appeal_${penalty.id}_${Date.now()}`;

    setShowTxDialog(true);
    await execute(async () => {
      await appealPenalty(penalty.id, cid);
      setTimeout(() => {
        setShowTxDialog(false);
        Alert.alert('申诉已提交', '您的申诉已提交，请等待审核结果', [
          { text: '确定', onPress: () => router.back() },
        ]);
      }, 1500);
    });
  };

  const handleCloseTxDialog = () => {
    setShowTxDialog(false);
    clearError();
  };

  if (!penalty) {
    return (
      <View style={styles.loadingContainer}>
        <LoadingSpinner text="加载中..." />
      </View>
    );
  }

  // 检查是否可以申诉
  const appealDeadline = new Date((penalty.deductedAt + 7 * 24 * 3600) * 1000);
  const now = new Date();
  const canAppeal = !penalty.appealed && now < appealDeadline;

  if (!canAppeal) {
    return (
      <View style={styles.container}>
        <PageHeader title="发起申诉" showBack />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>❌</Text>
          <Text style={styles.emptyText}>
            {penalty.appealed ? '该记录已申诉' : '申诉期限已过'}
          </Text>
        </View>
      </View>
    );
  }

  const typeText = MakerService.getPenaltyTypeText(penalty.penaltyType);

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
    >
      <PageHeader title="发起申诉" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 扣除信息 */}
        <Card style={styles.section}>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>扣除编号</Text>
            <Text style={styles.infoValue}>#P{penalty.id}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>扣除金额</Text>
            <Text style={[styles.infoValue, styles.infoValueRed]}>
              {MakerService.formatDustAmount(penalty.deductedAmount)} DUST (${MakerService.formatUsdAmount(penalty.usdValue)})
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>扣除原因</Text>
            <Text style={styles.infoValue}>{typeText}</Text>
          </View>
        </Card>

        {/* 申诉理由 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>申诉理由</Text>
          <TextInput
            style={styles.textArea}
            placeholder="请详细说明申诉理由..."
            value={reason}
            onChangeText={setReason}
            multiline
            numberOfLines={6}
            textAlignVertical="top"
          />
        </View>

        {/* 上传证据 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>上传证据</Text>
          <TouchableOpacity style={styles.uploadButton}>
            <Text style={styles.uploadIcon}>📎</Text>
            <Text style={styles.uploadText}>点击上传证据文件</Text>
            <Text style={styles.uploadHint}>支持图片、PDF (最大10MB)</Text>
          </TouchableOpacity>

          {/* 已上传文件列表（示例） */}
          {evidenceCid && (
            <View style={styles.fileList}>
              <View style={styles.fileItem}>
                <Text style={styles.fileIcon}>📄</Text>
                <Text style={styles.fileName}>证据文件</Text>
                <TouchableOpacity onPress={() => setEvidenceCid('')}>
                  <Text style={styles.fileDelete}>删除</Text>
                </TouchableOpacity>
              </View>
            </View>
          )}
        </View>

        {/* 申诉须知 */}
        <Card style={[styles.section, styles.infoCard]}>
          <Text style={styles.infoCardIcon}>⚠️</Text>
          <Text style={styles.infoCardTitle}>申诉须知</Text>
          <View style={styles.infoList}>
            <Text style={styles.infoItem}>• 申诉将由平台仲裁员审核</Text>
            <Text style={styles.infoItem}>• 审核周期约 3-7 个工作日</Text>
            <Text style={styles.infoItem}>• 申诉成功将退还扣除金额</Text>
            <Text style={styles.infoItem}>• 恶意申诉将加重处罚</Text>
          </View>
        </Card>

        {/* 提交按钮 */}
        <Button
          title="提交申诉"
          onPress={handleSubmit}
          loading={isLoading}
          disabled={!reason.trim() || isLoading}
        />
      </ScrollView>

      {/* 交易状态弹窗 */}
      <TransactionStatusDialog
        visible={showTxDialog}
        status={txStatus || ''}
        error={error}
        onClose={handleCloseTxDialog}
      />
    </KeyboardAvoidingView>
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
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#8E8E93',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  section: {
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
    maxWidth: 200,
    textAlign: 'right',
  },
  infoValueRed: {
    color: '#FF3B30',
  },
  section: {
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 8,
  },
  textArea: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    fontSize: 15,
    color: '#1C1C1E',
    minHeight: 120,
  },
  uploadButton: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 24,
    alignItems: 'center',
    borderWidth: 2,
    borderColor: '#E5E5EA',
    borderStyle: 'dashed',
  },
  uploadIcon: {
    fontSize: 32,
    marginBottom: 8,
  },
  uploadText: {
    fontSize: 15,
    color: '#1C1C1E',
    marginBottom: 4,
  },
  uploadHint: {
    fontSize: 12,
    color: '#8E8E93',
  },
  fileList: {
    marginTop: 12,
  },
  fileItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFFFFF',
    borderRadius: 8,
    padding: 12,
  },
  fileIcon: {
    fontSize: 20,
    marginRight: 8,
  },
  fileName: {
    flex: 1,
    fontSize: 14,
    color: '#1C1C1E',
  },
  fileDelete: {
    fontSize: 14,
    color: '#FF3B30',
  },
  infoCard: {
    backgroundColor: '#FFF9E6',
  },
  infoCardIcon: {
    fontSize: 20,
    marginBottom: 8,
  },
  infoCardTitle: {
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
