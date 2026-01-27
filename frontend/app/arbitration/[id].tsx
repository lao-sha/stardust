/**
 * 争议详情页面
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  Alert,
  ActivityIndicator,
  TextInput,
  RefreshControl,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import {
  arbitrationService,
  Dispute,
  Evidence,
  ArbitrationResult,
  DisputeStatus,
  DisputeType,
} from '@/services/arbitration.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

export default function DisputeDetailPage() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [dispute, setDispute] = useState<Dispute | null>(null);
  const [evidences, setEvidences] = useState<Evidence[]>([]);
  const [result, setResult] = useState<ArbitrationResult | null>(null);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [showEvidenceForm, setShowEvidenceForm] = useState(false);
  const [evidenceCid, setEvidenceCid] = useState('');
  const [evidenceDesc, setEvidenceDesc] = useState('');
  const [pendingAction, setPendingAction] = useState<'evidence' | 'appeal' | 'close' | null>(null);

  const disputeId = parseInt(id || '0');

  const loadData = useCallback(async () => {
    try {
      const [disputeData, evidenceData, resultData] = await Promise.all([
        arbitrationService.getDispute(disputeId),
        arbitrationService.getDisputeEvidences(disputeId),
        arbitrationService.getArbitrationResult(disputeId),
      ]);

      setDispute(disputeData);
      setEvidences(evidenceData);
      setResult(resultData);
    } catch (error) {
      console.error('Load dispute error:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [disputeId]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const onRefresh = () => {
    setRefreshing(true);
    loadData();
  };

  const handleAction = async (action: 'evidence' | 'appeal' | 'close') => {
    if (!isSignerUnlocked()) {
      setPendingAction(action);
      setShowUnlockDialog(true);
      return;
    }

    await executeAction(action);
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      if (pendingAction) {
        await executeAction(pendingAction);
        setPendingAction(null);
      }
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeAction = async (action: 'evidence' | 'appeal' | 'close') => {
    setShowTxStatus(true);

    try {
      if (action === 'evidence') {
        if (!evidenceCid || !evidenceDesc) {
          Alert.alert('提示', '请填写证据信息');
          setShowTxStatus(false);
          return;
        }
        setTxStatus('正在提交证据...');
        await arbitrationService.submitEvidence(
          disputeId,
          evidenceCid,
          evidenceDesc,
          (status) => setTxStatus(status)
        );
        setShowEvidenceForm(false);
        setEvidenceCid('');
        setEvidenceDesc('');
      } else if (action === 'appeal') {
        setTxStatus('正在提交申诉...');
        await arbitrationService.appeal(
          disputeId,
          '对仲裁结果有异议',
          undefined,
          (status) => setTxStatus(status)
        );
      } else if (action === 'close') {
        setTxStatus('正在关闭争议...');
        await arbitrationService.closeDispute(disputeId, (status) => setTxStatus(status));
      }

      setTxStatus('操作成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        loadData();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('操作失败', error.message || '请稍后重试');
    }
  };

  const getStatusText = (status: DisputeStatus) => {
    switch (status) {
      case DisputeStatus.Pending:
        return '待处理';
      case DisputeStatus.UnderReview:
        return '审核中';
      case DisputeStatus.Resolved:
        return '已解决';
      case DisputeStatus.Appealed:
        return '已申诉';
      case DisputeStatus.Closed:
        return '已关闭';
      default:
        return '未知';
    }
  };

  const getStatusColor = (status: DisputeStatus) => {
    switch (status) {
      case DisputeStatus.Pending:
        return '#FF9500';
      case DisputeStatus.UnderReview:
        return '#4A90D9';
      case DisputeStatus.Resolved:
        return '#4CD964';
      case DisputeStatus.Appealed:
        return '#FF6B6B';
      case DisputeStatus.Closed:
        return '#999';
      default:
        return '#999';
    }
  };

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="争议详情" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  if (!dispute) {
    return (
      <View style={styles.container}>
        <PageHeader title="争议详情" showBack />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>争议不存在</Text>
        </View>
        <BottomNavBar />
      </View>
    );
  }

  const isPlaintiff = dispute.plaintiff === address;
  const canSubmitEvidence = dispute.status === DisputeStatus.Pending || dispute.status === DisputeStatus.UnderReview;
  const canAppeal = dispute.status === DisputeStatus.Resolved && !isPlaintiff;
  const canClose = dispute.status === DisputeStatus.Pending && isPlaintiff;

  return (
    <View style={styles.container}>
      <PageHeader title="争议详情" showBack />

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
      >
        {/* 状态卡片 */}
        <View style={[styles.statusCard, { borderColor: getStatusColor(dispute.status) }]}>
          <View style={styles.statusHeader}>
            <Ionicons name="shield-checkmark" size={24} color={getStatusColor(dispute.status)} />
            <Text style={[styles.statusText, { color: getStatusColor(dispute.status) }]}>
              {getStatusText(dispute.status)}
            </Text>
          </View>
          <Text style={styles.disputeId}>争议 #{dispute.id}</Text>
        </View>

        {/* 基本信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>基本信息</Text>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>争议类型</Text>
            <Text style={styles.infoValue}>
              {dispute.disputeType === DisputeType.Order ? '订单争议' :
               dispute.disputeType === DisputeType.Swap ? '兑换争议' :
               dispute.disputeType === DisputeType.Service ? '服务争议' : '其他'}
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>相关ID</Text>
            <Text style={styles.infoValue}>{dispute.relatedId}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>原告</Text>
            <Text style={styles.infoValue}>
              {isPlaintiff ? '我' : `${dispute.plaintiff.slice(0, 8)}...${dispute.plaintiff.slice(-6)}`}
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>被告</Text>
            <Text style={styles.infoValue}>
              {!isPlaintiff ? '我' : `${dispute.defendant.slice(0, 8)}...${dispute.defendant.slice(-6)}`}
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>创建时间</Text>
            <Text style={styles.infoValue}>
              {new Date(dispute.createdAt * 1000).toLocaleString()}
            </Text>
          </View>
        </View>

        {/* 争议原因 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>争议原因</Text>
          <Text style={styles.reasonText}>{dispute.reason}</Text>
        </View>

        {/* 证据列表 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>证据材料</Text>
            {canSubmitEvidence && (
              <Pressable
                style={styles.addButton}
                onPress={() => setShowEvidenceForm(!showEvidenceForm)}
              >
                <Ionicons name={showEvidenceForm ? 'close' : 'add'} size={20} color={THEME_COLOR} />
              </Pressable>
            )}
          </View>

          {showEvidenceForm && (
            <View style={styles.evidenceForm}>
              <TextInput
                style={styles.input}
                value={evidenceCid}
                onChangeText={setEvidenceCid}
                placeholder="证据的 IPFS CID"
              />
              <TextInput
                style={[styles.input, styles.textArea]}
                value={evidenceDesc}
                onChangeText={setEvidenceDesc}
                placeholder="证据描述"
                multiline
                numberOfLines={3}
              />
              <Pressable
                style={styles.submitEvidenceButton}
                onPress={() => handleAction('evidence')}
              >
                <Text style={styles.submitEvidenceText}>提交证据</Text>
              </Pressable>
            </View>
          )}

          {evidences.length === 0 ? (
            <Text style={styles.noEvidence}>暂无证据</Text>
          ) : (
            evidences.map((evidence, index) => (
              <View key={evidence.id} style={styles.evidenceItem}>
                <View style={styles.evidenceHeader}>
                  <Text style={styles.evidenceIndex}>证据 {index + 1}</Text>
                  <Text style={styles.evidenceTime}>
                    {new Date(evidence.submittedAt * 1000).toLocaleString()}
                  </Text>
                </View>
                <Text style={styles.evidenceDesc}>{evidence.description}</Text>
                <Text style={styles.evidenceCid}>CID: {evidence.evidenceCid}</Text>
              </View>
            ))
          )}
        </View>

        {/* 仲裁结果 */}
        {result && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>仲裁结果</Text>
            <View style={styles.resultCard}>
              <View style={styles.resultRow}>
                <Text style={styles.resultLabel}>胜诉方</Text>
                <Text style={styles.resultValue}>
                  {result.winner === address ? '我' : `${result.winner.slice(0, 8)}...`}
                </Text>
              </View>
              <View style={styles.resultRow}>
                <Text style={styles.resultLabel}>赔偿金额</Text>
                <Text style={styles.resultValue}>
                  {(Number(result.compensation) / 1e12).toFixed(4)} DUST
                </Text>
              </View>
              <Text style={styles.resultReason}>{result.reason}</Text>
            </View>
          </View>
        )}

        {/* 操作按钮 */}
        <View style={styles.actions}>
          {canClose && (
            <Pressable
              style={[styles.actionButton, styles.closeButton]}
              onPress={() => handleAction('close')}
            >
              <Text style={styles.closeButtonText}>撤回争议</Text>
            </Pressable>
          )}
          {canAppeal && (
            <Pressable
              style={[styles.actionButton, styles.appealButton]}
              onPress={() => handleAction('appeal')}
            >
              <Text style={styles.appealButtonText}>提起申诉</Text>
            </Pressable>
          )}
        </View>

        <View style={{ height: 40 }} />
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

      <BottomNavBar />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  content: {
    flex: 1,
    padding: 16,
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
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    borderWidth: 2,
  },
  statusHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusText: {
    fontSize: 18,
    fontWeight: 'bold',
    marginLeft: 8,
  },
  disputeId: {
    fontSize: 14,
    color: '#999',
    marginTop: 8,
  },
  section: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
    marginBottom: 12,
  },
  addButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#f8f4e8',
    justifyContent: 'center',
    alignItems: 'center',
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  infoLabel: {
    fontSize: 14,
    color: '#666',
  },
  infoValue: {
    fontSize: 14,
    color: '#333',
  },
  reasonText: {
    fontSize: 14,
    color: '#333',
    lineHeight: 22,
  },
  evidenceForm: {
    backgroundColor: '#f8f8f8',
    borderRadius: 8,
    padding: 12,
    marginBottom: 12,
  },
  input: {
    backgroundColor: '#fff',
    borderRadius: 8,
    padding: 12,
    fontSize: 14,
    borderWidth: 1,
    borderColor: '#e0e0e0',
    marginBottom: 8,
  },
  textArea: {
    height: 80,
    textAlignVertical: 'top',
  },
  submitEvidenceButton: {
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    padding: 12,
    alignItems: 'center',
  },
  submitEvidenceText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  noEvidence: {
    fontSize: 14,
    color: '#999',
    textAlign: 'center',
    paddingVertical: 20,
  },
  evidenceItem: {
    backgroundColor: '#f8f8f8',
    borderRadius: 8,
    padding: 12,
    marginBottom: 8,
  },
  evidenceHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  evidenceIndex: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME_COLOR,
  },
  evidenceTime: {
    fontSize: 12,
    color: '#999',
  },
  evidenceDesc: {
    fontSize: 14,
    color: '#333',
    marginTop: 8,
  },
  evidenceCid: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
  },
  resultCard: {
    backgroundColor: '#f8f4e8',
    borderRadius: 8,
    padding: 12,
  },
  resultRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  resultLabel: {
    fontSize: 14,
    color: '#666',
  },
  resultValue: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME_COLOR,
  },
  resultReason: {
    fontSize: 14,
    color: '#333',
    marginTop: 8,
  },
  actions: {
    flexDirection: 'row',
    marginTop: 8,
  },
  actionButton: {
    flex: 1,
    paddingVertical: 14,
    borderRadius: 25,
    alignItems: 'center',
    marginHorizontal: 4,
  },
  closeButton: {
    backgroundColor: '#f5f5f5',
  },
  closeButtonText: {
    color: '#666',
    fontSize: 16,
    fontWeight: '500',
  },
  appealButton: {
    backgroundColor: THEME_COLOR,
  },
  appealButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '500',
  },
});
