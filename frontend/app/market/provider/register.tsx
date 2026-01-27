// frontend/app/market/provider/register.tsx

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  TouchableOpacity,
  SafeAreaView,
  StatusBar,
  Alert,
  Switch,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import { useWalletStore } from '@/stores/wallet.store';
import { useProvider, useChainTransaction } from '@/divination/market/hooks';
import { LoadingSpinner, TransactionStatus } from '@/divination/market/components';
import { Card, Button, Input } from '@/components/common';
import { useAsync, useWallet } from '@/hooks';
import { THEME, SHADOWS } from '@/divination/market/theme';
import {
  DIVINATION_TYPES,
  SPECIALTIES,
} from '@/divination/market/constants/market.constants';
import {
  validateName,
  validateBio,
  calculateBitmap,
} from '@/divination/market/utils/market.utils';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import type { DivinationType } from '@/divination/market/types';

export default function RegisterProviderScreen() {
  const router = useRouter();
  const { address, isLocked } = useWalletStore();
  const { isProvider, loading } = useProvider();
  const { registerProvider, txState, isProcessing, resetState } = useChainTransaction();
  const { execute, isLoading: submitting } = useAsync();
  const [showTxStatus, setShowTxStatus] = useState(false);

  // 表单状态
  const [name, setName] = useState('');
  const [bio, setBio] = useState('');
  const [selectedSpecialties, setSelectedSpecialties] = useState<number[]>([]);
  const [selectedTypes, setSelectedTypes] = useState<number[]>([]);
  const [acceptsUrgent, setAcceptsUrgent] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  // 错误状态
  const [nameError, setNameError] = useState('');
  const [bioError, setBioError] = useState('');

  const toggleSpecialty = (bit: number) => {
    setSelectedSpecialties((prev) =>
      prev.includes(bit) ? prev.filter((b) => b !== bit) : [...prev, bit]
    );
  };

  const toggleType = (id: number) => {
    setSelectedTypes((prev) =>
      prev.includes(id) ? prev.filter((t) => t !== id) : [...prev, id]
    );
  };

  const validateForm = (): boolean => {
    let valid = true;

    const nameValidation = validateName(name);
    if (!nameValidation.valid) {
      setNameError(nameValidation.message || '');
      valid = false;
    } else {
      setNameError('');
    }

    const bioValidation = validateBio(bio);
    if (!bioValidation.valid) {
      setBioError(bioValidation.message || '');
      valid = false;
    } else {
      setBioError('');
    }

    if (selectedSpecialties.length === 0) {
      Alert.alert('提示', '请至少选择一个擅长领域');
      valid = false;
    }

    if (selectedTypes.length === 0) {
      Alert.alert('提示', '请至少选择一种占卜类型');
      valid = false;
    }

    return valid;
  };

  handleSubmit = async () => {
    if (!address || isLocked) {
      Alert.alert('提示', '请先解锁钱包');
      return;
    }

    if (isProvider) {
      Alert.alert('提示', '您已经是解卦师了');
      router.push('/market/provider/dashboard');
      return;
    }

    if (!validateForm()) return;

    setShowTxStatus(true);

    await execute(async () => {
      // 调用链上交易
      const result = await registerProvider(
        {
          name,
          bio,
          divinationTypes: selectedTypes as DivinationType[],
          specialties: selectedSpecialties,
        },
        {
          onSuccess: () => {
            // 交易成功后跳转
            setTimeout(() => {
              setShowTxStatus(false);
              resetState();
              router.replace('/market/provider/dashboard');
            }, 1500);
          },
          onError: (error) => {
            console.error('Register provider error:', error);
          },
        }
      );

      if (!result) {
        // 用户取消了交易
        setShowTxStatus(false);
      }
    });
  };

  const handleTxStatusClose = () => {
    setShowTxStatus(false);
    resetState();
  };

  if (!address) {
    return (
      <View style={styles.container}>
        <PageHeader title="成为解卦师" />
        <View style={styles.centerContent}>
          <Ionicons name="wallet-outline" size={64} color={THEME.border} />
          <Text style={styles.emptyText}>请先创建或导入钱包</Text>
        </View>
        <BottomNavBar activeTab="market" />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor="#FFFFFF" />

      {/* 顶部导航 */}
      <PageHeader title="成为解卦师" />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 基本信息 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>基本信息</Text>

          <View style={styles.inputGroup}>
            <Text style={styles.label}>名称 *</Text>
            <TextInput
              style={[styles.input, nameError && styles.inputError]}
              placeholder="您的称号，如：云中子"
              placeholderTextColor={THEME.textTertiary}
              value={name}
              onChangeText={setName}
              maxLength={20}
            />
            {nameError && <Text style={styles.errorText}>{nameError}</Text>}
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.label}>简介 *</Text>
            <TextInput
              style={[styles.textArea, bioError && styles.inputError]}
              placeholder="介绍您的专业背景、擅长领域等（20-200字）"
              placeholderTextColor={THEME.textTertiary}
              value={bio}
              onChangeText={setBio}
              multiline
              numberOfLines={4}
              textAlignVertical="top"
              maxLength={200}
            />
            <Text style={styles.charCount}>{bio.length}/200</Text>
            {bioError && <Text style={styles.errorText}>{bioError}</Text>}
          </View>
        </Card>

        {/* 擅长领域 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>擅长领域 *</Text>
          <Text style={styles.sectionDesc}>选择您擅长解答的问题类型</Text>
          <View style={styles.tagsContainer}>
            {SPECIALTIES.map((specialty) => (
              <TouchableOpacity
                key={specialty.bit}
                style={[
                  styles.tag,
                  selectedSpecialties.includes(specialty.bit) && {
                    backgroundColor: specialty.color + '20',
                    borderColor: specialty.color,
                  },
                ]}
                onPress={() => toggleSpecialty(specialty.bit)}
              >
                <Ionicons
                  name={specialty.icon as any}
                  size={14}
                  color={
                    selectedSpecialties.includes(specialty.bit)
                      ? specialty.color
                      : THEME.textSecondary
                  }
                />
                <Text
                  style={[
                    styles.tagText,
                    selectedSpecialties.includes(specialty.bit) && {
                      color: specialty.color,
                    },
                  ]}
                >
                  {specialty.name}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </Card>

        {/* 占卜类型 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>占卜类型 *</Text>
          <Text style={styles.sectionDesc}>选择您擅长的占卜术</Text>
          <View style={styles.tagsContainer}>
            {DIVINATION_TYPES.map((type) => (
              <TouchableOpacity
                key={type.id}
                style={[
                  styles.tag,
                  selectedTypes.includes(type.id) && {
                    backgroundColor: type.color + '20',
                    borderColor: type.color,
                  },
                ]}
                onPress={() => toggleType(type.id)}
              >
                <Text
                  style={[
                    styles.tagText,
                    selectedTypes.includes(type.id) && {
                      color: type.color,
                    },
                  ]}
                >
                  {type.name}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </Card>

        {/* 其他设置 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>其他设置</Text>

          <View style={styles.switchRow}>
            <View style={styles.switchInfo}>
              <Text style={styles.switchLabel}>接受加急订单</Text>
              <Text style={styles.switchDesc}>开启后可接受客户的加急需求</Text>
            </View>
            <Switch
              value={acceptsUrgent}
              onValueChange={setAcceptsUrgent}
              trackColor={{ false: THEME.border, true: THEME.primary + '60' }}
              thumbColor={acceptsUrgent ? THEME.primary : THEME.textTertiary}
            />
          </View>
        </Card>

        {/* 提示 */}
        <View style={styles.notice}>
          <Ionicons name="information-circle-outline" size={18} color={THEME.info} />
          <Text style={styles.noticeText}>
            注册后需要创建服务套餐才能接收订单。平台将收取一定比例的服务费用，具体费率根据等级而定。
          </Text>
        </View>

        {/* 提交按钮 */}
        <Button
          title="提交申请"
          onPress={handleSubmit}
          loading={submitting}
          disabled={!formValid || submitting}
        />

        <View style={styles.bottomSpace} />
      </ScrollView>

      {/* 交易状态弹窗 */}
      <TransactionStatus
        visible={showTxStatus}
        state={txState}
        onClose={handleTxStatusClose}
        successMessage="注册成功！您已成为解卦师"
      />

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="market" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  centerContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    gap: 16,
  },
  emptyText: {
    fontSize: 15,
    color: THEME.textSecondary,
  },
  content: {
    flex: 1,
    padding: 16,
  },
  section: {
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 4,
  },
  sectionDesc: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginBottom: 12,
  },
  inputGroup: {
    marginTop: 12,
  },
  label: {
    fontSize: 13,
    color: THEME.textSecondary,
    marginBottom: 6,
  },
  input: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 15,
    color: THEME.text,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  inputError: {
    borderColor: THEME.error,
  },
  textArea: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 15,
    color: THEME.text,
    borderWidth: 1,
    borderColor: THEME.border,
    height: 100,
  },
  charCount: {
    fontSize: 11,
    color: THEME.textTertiary,
    textAlign: 'right',
    marginTop: 4,
  },
  errorText: {
    fontSize: 12,
    color: THEME.error,
    marginTop: 4,
  },
  tagsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 10,
  },
  tag: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 20,
    backgroundColor: THEME.background,
    borderWidth: 1,
    borderColor: THEME.border,
    gap: 4,
  },
  tagText: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  switchRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginTop: 8,
  },
  switchInfo: {
    flex: 1,
    marginRight: 16,
  },
  switchLabel: {
    fontSize: 14,
    color: THEME.text,
    fontWeight: '500',
  },
  switchDesc: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 2,
  },
  notice: {
    flexDirection: 'row',
    backgroundColor: THEME.info + '10',
    borderRadius: 8,
    padding: 12,
    gap: 8,
    marginBottom: 16,
  },
  noticeText: {
    flex: 1,
    fontSize: 12,
    color: THEME.info,
    lineHeight: 18,
  },
  submitBtn: {
    backgroundColor: THEME.primary,
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  submitBtnDisabled: {
    opacity: 0.6,
  },
  submitBtnText: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.textInverse,
  },
  bottomSpace: {
    height: 32,
  },
});
