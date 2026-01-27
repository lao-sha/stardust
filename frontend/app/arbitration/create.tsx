/**
 * 创建争议页面
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  Pressable,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { arbitrationService, DisputeType } from '@/services/arbitration.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

export default function CreateDisputePage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');

  // 表单状态
  const [disputeType, setDisputeType] = useState<DisputeType>(DisputeType.Order);
  const [relatedId, setRelatedId] = useState('');
  const [defendant, setDefendant] = useState('');
  const [reason, setReason] = useState('');

  // 验证
  const relatedIdValid = /^\d+$/.test(relatedId);
  const defendantValid = defendant.length >= 40 && defendant.length <= 50;
  const reasonValid = reason.length >= 10 && reason.length <= 1000;
  const formValid = relatedIdValid && defendantValid && reasonValid;

  const handleSubmit = async () => {
    if (!formValid) {
      Alert.alert('提示', '请完整填写所有必填项');
      return;
    }

    if (!isSignerUnlocked()) {
      setShowUnlockDialog(true);
      return;
    }

    await executeCreate();
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      await executeCreate();
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeCreate = async () => {
    setLoading(true);
    setShowTxStatus(true);
    setTxStatus('正在创建争议...');

    try {
      const disputeId = await arbitrationService.createDispute(
        disputeType,
        parseInt(relatedId),
        defendant,
        reason,
        undefined,
        (status) => setTxStatus(status)
      );

      setTxStatus('创建成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        router.replace(`/arbitration/${disputeId}` as any);
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('创建失败', error.message || '请稍后重试');
    } finally {
      setLoading(false);
    }
  };

  const disputeTypes = [
    { type: DisputeType.Order, label: '订单争议', icon: 'receipt' },
    { type: DisputeType.Swap, label: '兑换争议', icon: 'swap-horizontal' },
    { type: DisputeType.Service, label: '服务争议', icon: 'construct' },
    { type: DisputeType.Other, label: '其他', icon: 'ellipsis-horizontal' },
  ];

  return (
    <View style={styles.container}>
      <PageHeader title="发起争议" showBack />

      <ScrollView style={styles.content}>
        {/* 争议类型 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>争议类型 *</Text>
          <View style={styles.typeGrid}>
            {disputeTypes.map((item) => (
              <Pressable
                key={item.type}
                style={[
                  styles.typeButton,
                  disputeType === item.type && styles.typeButtonActive,
                ]}
                onPress={() => setDisputeType(item.type)}
              >
                <Ionicons
                  name={item.icon as any}
                  size={24}
                  color={disputeType === item.type ? '#fff' : THEME_COLOR}
                />
                <Text
                  style={[
                    styles.typeText,
                    disputeType === item.type && styles.typeTextActive,
                  ]}
                >
                  {item.label}
                </Text>
              </Pressable>
            ))}
          </View>
        </View>

        {/* 相关ID */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>
            {disputeType === DisputeType.Order ? '订单ID' : disputeType === DisputeType.Swap ? '兑换ID' : '相关ID'} *
          </Text>
          <TextInput
            style={[styles.input, !relatedIdValid && relatedId.length > 0 && styles.inputError]}
            value={relatedId}
            onChangeText={setRelatedId}
            placeholder="请输入相关记录的ID"
            keyboardType="numeric"
          />
        </View>

        {/* 被告地址 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>对方地址 *</Text>
          <TextInput
            style={[styles.input, !defendantValid && defendant.length > 0 && styles.inputError]}
            value={defendant}
            onChangeText={setDefendant}
            placeholder="请输入对方的钱包地址"
            autoCapitalize="none"
          />
          <Text style={styles.hint}>以 5 开头的 Substrate 地址</Text>
        </View>

        {/* 争议原因 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>争议原因 *</Text>
          <TextInput
            style={[styles.input, styles.textArea, !reasonValid && reason.length > 0 && styles.inputError]}
            value={reason}
            onChangeText={setReason}
            placeholder="请详细描述您遇到的问题（至少10个字）"
            multiline
            numberOfLines={6}
            maxLength={1000}
          />
          <Text style={styles.charCount}>{reason.length}/1000</Text>
        </View>

        {/* 提示信息 */}
        <View style={styles.tipCard}>
          <Ionicons name="information-circle" size={20} color={THEME_COLOR} />
          <View style={styles.tipContent}>
            <Text style={styles.tipTitle}>温馨提示</Text>
            <Text style={styles.tipText}>
              • 请确保提供真实准确的信息{'\n'}
              • 创建争议后可以继续补充证据{'\n'}
              • 恶意争议可能导致账户受限
            </Text>
          </View>
        </View>

        {/* 提交按钮 */}
        <Pressable
          style={[styles.submitButton, !formValid && styles.submitButtonDisabled]}
          onPress={handleSubmit}
          disabled={loading || !formValid}
        >
          {loading ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text style={styles.submitButtonText}>提交争议</Text>
          )}
        </Pressable>

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
  formGroup: {
    marginBottom: 20,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#fff',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    borderWidth: 1,
    borderColor: '#e0e0e0',
  },
  inputError: {
    borderColor: '#FF6B6B',
  },
  textArea: {
    height: 150,
    textAlignVertical: 'top',
  },
  charCount: {
    fontSize: 12,
    color: '#999',
    textAlign: 'right',
    marginTop: 4,
  },
  hint: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
  typeGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  typeButton: {
    width: '48%',
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#fff',
    borderRadius: 8,
    padding: 12,
    marginRight: '4%',
    marginBottom: 8,
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  typeButtonActive: {
    backgroundColor: THEME_COLOR,
  },
  typeText: {
    fontSize: 14,
    color: THEME_COLOR,
    marginLeft: 8,
  },
  typeTextActive: {
    color: '#fff',
  },
  tipCard: {
    flexDirection: 'row',
    backgroundColor: '#f8f4e8',
    borderRadius: 8,
    padding: 12,
    marginBottom: 20,
  },
  tipContent: {
    flex: 1,
    marginLeft: 8,
  },
  tipTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME_COLOR,
  },
  tipText: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
    lineHeight: 18,
  },
  submitButton: {
    backgroundColor: THEME_COLOR,
    borderRadius: 25,
    padding: 16,
    alignItems: 'center',
  },
  submitButtonDisabled: {
    backgroundColor: '#ccc',
  },
  submitButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: 'bold',
  },
});
