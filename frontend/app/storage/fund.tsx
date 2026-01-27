/**
 * IPFS 存储账户充值页面
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
import { ipfsStorageService } from '@/services/ipfs-storage.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';
const DUST_DECIMALS = 12;

export default function FundStoragePage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');

  // 表单状态
  const [subjectType, setSubjectType] = useState('bazi');
  const [subjectId, setSubjectId] = useState('');
  const [amount, setAmount] = useState('');

  // 验证
  const subjectIdValid = subjectId.length > 0;
  const amountValid = /^\d+(\.\d{1,4})?$/.test(amount) && parseFloat(amount) > 0;
  const formValid = subjectIdValid && amountValid;

  const handleSubmit = async () => {
    if (!formValid) {
      Alert.alert('提示', '请完整填写所有必填项');
      return;
    }

    if (!isSignerUnlocked()) {
      setShowUnlockDialog(true);
      return;
    }

    await executeFund();
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      await executeFund();
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeFund = async () => {
    setLoading(true);
    setShowTxStatus(true);
    setTxStatus('正在充值...');

    try {
      const amountBigInt = BigInt(Math.floor(parseFloat(amount) * Math.pow(10, DUST_DECIMALS)));

      await ipfsStorageService.fundSubjectAccount(
        subjectType,
        subjectId,
        amountBigInt,
        (status) => setTxStatus(status)
      );

      setTxStatus('充值成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        router.back();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('充值失败', error.message || '请稍后重试');
    } finally {
      setLoading(false);
    }
  };

  const subjectTypes = [
    { type: 'bazi', label: '八字', icon: 'calendar' },
    { type: 'chat', label: '聊天', icon: 'chatbubbles' },
    { type: 'matchmaking', label: '婚恋', icon: 'heart' },
    { type: 'divination', label: '占卜', icon: 'sparkles' },
  ];

  return (
    <View style={styles.container}>
      <PageHeader title="充值存储账户" showBack />

      <ScrollView style={styles.content}>
        {/* 主题类型 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>存储类型 *</Text>
          <View style={styles.typeGrid}>
            {subjectTypes.map((item) => (
              <Pressable
                key={item.type}
                style={[
                  styles.typeButton,
                  subjectType === item.type && styles.typeButtonActive,
                ]}
                onPress={() => setSubjectType(item.type)}
              >
                <Ionicons
                  name={item.icon as any}
                  size={20}
                  color={subjectType === item.type ? '#fff' : THEME_COLOR}
                />
                <Text
                  style={[
                    styles.typeText,
                    subjectType === item.type && styles.typeTextActive,
                  ]}
                >
                  {item.label}
                </Text>
              </Pressable>
            ))}
          </View>
        </View>

        {/* 主题ID */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>主题ID *</Text>
          <TextInput
            style={[styles.input, !subjectIdValid && subjectId.length > 0 && styles.inputError]}
            value={subjectId}
            onChangeText={setSubjectId}
            placeholder="如: 命盘ID、会话ID等"
          />
          <Text style={styles.hint}>
            输入您要充值的{subjectType === 'bazi' ? '命盘' : subjectType === 'chat' ? '会话' : '记录'}ID
          </Text>
        </View>

        {/* 充值金额 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>充值金额 (DUST) *</Text>
          <TextInput
            style={[styles.input, !amountValid && amount.length > 0 && styles.inputError]}
            value={amount}
            onChangeText={setAmount}
            placeholder="如: 10.0000"
            keyboardType="decimal-pad"
          />
          <Text style={styles.hint}>最小充值 0.0001 DUST</Text>
        </View>

        {/* 快捷金额 */}
        <View style={styles.quickAmounts}>
          {['1', '5', '10', '50'].map((val) => (
            <Pressable
              key={val}
              style={[styles.quickButton, amount === val && styles.quickButtonActive]}
              onPress={() => setAmount(val)}
            >
              <Text style={[styles.quickText, amount === val && styles.quickTextActive]}>
                {val} DUST
              </Text>
            </Pressable>
          ))}
        </View>

        {/* 费用说明 */}
        <View style={styles.infoCard}>
          <Ionicons name="information-circle" size={20} color={THEME_COLOR} />
          <View style={styles.infoContent}>
            <Text style={styles.infoTitle}>存储费用说明</Text>
            <Text style={styles.infoText}>
              • 存储费用按文件大小和存储时长计算{'\n'}
              • 账户余额不足时，内容可能被取消固定{'\n'}
              • 建议保持充足余额以确保数据安全
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
            <Text style={styles.submitButtonText}>确认充值</Text>
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
  quickAmounts: {
    flexDirection: 'row',
    marginBottom: 20,
  },
  quickButton: {
    flex: 1,
    backgroundColor: '#fff',
    borderRadius: 8,
    padding: 10,
    marginRight: 8,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#e0e0e0',
  },
  quickButtonActive: {
    borderColor: THEME_COLOR,
    backgroundColor: '#f8f4e8',
  },
  quickText: {
    fontSize: 13,
    color: '#666',
  },
  quickTextActive: {
    color: THEME_COLOR,
    fontWeight: '500',
  },
  infoCard: {
    flexDirection: 'row',
    backgroundColor: '#f8f4e8',
    borderRadius: 8,
    padding: 12,
    marginBottom: 20,
  },
  infoContent: {
    flex: 1,
    marginLeft: 8,
  },
  infoTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME_COLOR,
  },
  infoText: {
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
