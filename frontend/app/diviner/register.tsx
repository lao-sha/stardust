/**
 * 占卜师注册页面
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { SpecialtySelector, DivinationTypeSelector } from '@/features/diviner';
import { Card, Button, Input } from '@/components/common';
import { useAsync } from '@/hooks';
import { divinationMarketService } from '@/services/divination-market.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';
const MIN_DEPOSIT = 100;
const DUST_DECIMALS = 12;

export default function DivinerRegisterPage() {
  const router = useRouter();
  const { execute, isLoading } = useAsync();
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');

  // 表单状态
  const [name, setName] = useState('');
  const [bio, setBio] = useState('');
  const [specialties, setSpecialties] = useState(0);
  const [supportedTypes, setSupportedTypes] = useState(0);

  // 验证
  const nameValid = name.length >= 1 && name.length <= 64;
  const bioValid = bio.length >= 1 && bio.length <= 256;
  const specialtiesValid = specialties > 0;
  const typesValid = supportedTypes > 0;
  const formValid = nameValid && bioValid && specialtiesValid && typesValid;

  const handleSubmit = async () => {
    if (!formValid) {
      Alert.alert('提示', '请完整填写所有必填项');
      return;
    }

    // 检查钱包是否解锁
    if (!isSignerUnlocked()) {
      setShowUnlockDialog(true);
      return;
    }

    await executeRegister();
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      await executeRegister();
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeRegister = async () => {
    setShowTxStatus(true);
    setTxStatus('正在提交注册申请...');

    await execute(async () => {
      // 将保证金转换为最小单位
      const depositBigInt = BigInt(MIN_DEPOSIT * Math.pow(10, DUST_DECIMALS));

      // 调用链上注册方法
      const providerId = await divinationMarketService.registerProvider(
        name,
        bio,
        specialties,
        supportedTypes,
        depositBigInt,
        (status) => {
          setTxStatus(status);
        }
      );

      setTxStatus('注册成功！');

      setTimeout(() => {
        setShowTxStatus(false);

        Alert.alert(
          '注册成功',
          `您的申请已提交，请等待审核通过\n解卦师ID: ${providerId}`,
          [{ text: '确定', onPress: () => router.push('/diviner/dashboard' as any) }]
        );
      }, 1500);
    }, {
      onError: (error) => {
        setTxStatus('注册失败');
        setTimeout(() => {
          setShowTxStatus(false);
          Alert.alert('注册失败', error.message || '请稍后重试');
        }, 1500);
      }
    });
  };

  return (
    <View style={styles.wrapper}>
      <PageHeader title="注册占卜师" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 保证金提示 */}
        <View style={styles.depositCard}>
          <Text style={styles.depositIcon}>💎</Text>
          <View style={styles.depositContent}>
            <Text style={styles.depositTitle}>保证金要求</Text>
            <Text style={styles.depositText}>
              注册需锁定 {MIN_DEPOSIT} DUST 作为保证金，注销时全额退还
            </Text>
          </View>
        </View>

        {/* 基本信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>基本信息</Text>
          
          <Card>
            <View style={styles.formItem}>
              <Text style={styles.label}>
                显示名称 <Text style={styles.required}>*</Text>
              </Text>
              <TextInput
                style={[styles.input, !nameValid && name.length > 0 && styles.inputError]}
                value={name}
                onChangeText={setName}
                placeholder="您的占卜师名称（1-64字符）"
                placeholderTextColor="#999"
                maxLength={64}
              />
              <Text style={styles.charCount}>{name.length}/64</Text>
            </View>

            <View style={styles.formItem}>
              <Text style={styles.label}>
                个人简介 <Text style={styles.required}>*</Text>
              </Text>
              <TextInput
                style={[styles.textArea, !bioValid && bio.length > 0 && styles.inputError]}
                value={bio}
                onChangeText={setBio}
                placeholder="介绍您的从业经历、擅长领域等（1-256字符）"
                placeholderTextColor="#999"
                multiline
                numberOfLines={4}
                maxLength={256}
                textAlignVertical="top"
              />
              <Text style={styles.charCount}>{bio.length}/256</Text>
            </View>
          </Card>
        </View>

        {/* 擅长领域 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>
            擅长领域 <Text style={styles.required}>*</Text>
          </Text>
          <Card>
            <SpecialtySelector value={specialties} onChange={setSpecialties} />
          </Card>
        </View>

        {/* 占卜类型 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>
            支持的占卜类型 <Text style={styles.required}>*</Text>
          </Text>
          <Card>
            <DivinationTypeSelector value={supportedTypes} onChange={setSupportedTypes} />
          </Card>
        </View>

        {/* 协议 */}
        <View style={styles.agreementSection}>
          <Text style={styles.agreementText}>
            点击"提交注册"即表示您同意
            <Text style={styles.agreementLink}>《占卜师服务协议》</Text>
          </Text>
        </View>

        {/* 提交按钮 */}
        <View style={styles.actionSection}>
          <Button
            title="提交注册"
            onPress={handleSubmit}
            loading={isLoading}
            disabled={!formValid || isLoading}
          />
        </View>
      </ScrollView>

      {/* 解锁钱包对话框 */}
      <UnlockWalletDialog
        visible={showUnlockDialog}
        onClose={() => setShowUnlockDialog(false)}
        onSuccess={handleWalletUnlocked}
      />

      {/* 交易状态对话框 */}
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
  depositCard: {
    flexDirection: 'row',
    backgroundColor: '#FFF9F0',
    margin: 16,
    padding: 16,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  depositIcon: {
    fontSize: 28,
    marginRight: 12,
  },
  depositContent: {
    flex: 1,
  },
  depositTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000',
    marginBottom: 4,
  },
  depositText: {
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
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
  required: {
    color: '#FF3B30',
  },
  formItem: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    color: '#333',
    marginBottom: 8,
    fontWeight: '500',
  },
  input: {
    height: 44,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FAFAFA',
  },
  inputError: {
    borderColor: '#FF3B30',
  },
  textArea: {
    height: 100,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FAFAFA',
  },
  charCount: {
    fontSize: 12,
    color: '#999',
    textAlign: 'right',
    marginTop: 4,
  },
  agreementSection: {
    paddingHorizontal: 16,
    marginBottom: 16,
  },
  agreementText: {
    fontSize: 12,
    color: '#999',
    textAlign: 'center',
  },
  agreementLink: {
    color: THEME_COLOR,
  },
  actionSection: {
    paddingHorizontal: 16,
  },
});
