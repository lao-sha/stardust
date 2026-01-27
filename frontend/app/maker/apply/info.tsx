/**
 * 提交资料页面
 * 路径: /maker/apply/info
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore } from '@/stores/maker.store';
import { MakerService, MakerInfoInput } from '@/services/maker.service';
import { PageHeader } from '@/components/PageHeader';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { Card, Button } from '@/components/common';
import { useAsync } from '@/hooks';

export default function InfoPage() {
  const router = useRouter();
  const { makerApp, submitInfo, txStatus, error, clearError, fetchMakerInfo } = useMakerStore();
  const { execute, isLoading } = useAsync();

  const [showTxDialog, setShowTxDialog] = useState(false);
  const [formData, setFormData] = useState<MakerInfoInput>({
    realName: '',
    idCardNumber: '',
    birthday: '',
    tronAddress: '',
    wechatId: '',
    epayNo: '',
    epayKey: '',
  });
  const [errors, setErrors] = useState<Partial<Record<keyof MakerInfoInput, string>>>({});

  useEffect(() => {
    fetchMakerInfo();
  }, []);

  // 计算剩余时间
  const [timeLeft, setTimeLeft] = useState('');

  useEffect(() => {
    if (!makerApp?.infoDeadline) return;

    const updateTime = () => {
      const now = Math.floor(Date.now() / 1000);
      const remaining = makerApp.infoDeadline - now;

      if (remaining <= 0) {
        setTimeLeft('已超时');
        return;
      }

      const minutes = Math.floor(remaining / 60);
      const seconds = remaining % 60;
      setTimeLeft(`${minutes}:${seconds.toString().padStart(2, '0')}`);
    };

    updateTime();
    const interval = setInterval(updateTime, 1000);
    return () => clearInterval(interval);
  }, [makerApp?.infoDeadline]);

  const validateForm = (): boolean => {
    const newErrors: Partial<Record<keyof MakerInfoInput, string>> = {};

    if (!formData.realName.trim()) {
      newErrors.realName = '请输入真实姓名';
    }

    if (!formData.idCardNumber.trim()) {
      newErrors.idCardNumber = '请输入身份证号';
    } else if (!MakerService.isValidIdCard(formData.idCardNumber)) {
      newErrors.idCardNumber = '身份证号格式不正确';
    }

    if (!formData.birthday.trim()) {
      newErrors.birthday = '请输入出生日期';
    } else if (!/^\d{4}-\d{2}-\d{2}$/.test(formData.birthday)) {
      newErrors.birthday = '日期格式应为 YYYY-MM-DD';
    }

    if (!formData.tronAddress.trim()) {
      newErrors.tronAddress = '请输入 TRON 地址';
    } else if (!MakerService.isValidTronAddress(formData.tronAddress)) {
      newErrors.tronAddress = 'TRON 地址格式不正确';
    }

    if (!formData.wechatId.trim()) {
      newErrors.wechatId = '请输入微信号';
    }

    // EPAY 配置可选，但如果填了商户号就必须填密钥
    if (formData.epayNo && !formData.epayKey) {
      newErrors.epayKey = '请输入 EPAY 密钥';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async () => {
    if (!validateForm()) {
      return;
    }

    setShowTxDialog(true);
    await execute(async () => {
      await submitInfo(formData);
      // 成功后跳转到等待审核页面
      setTimeout(() => {
        setShowTxDialog(false);
        router.replace('/maker/apply/pending');
      }, 1500);
    });
  };

  const handleCloseTxDialog = () => {
    setShowTxDialog(false);
    clearError();
  };

  const updateField = (field: keyof MakerInfoInput, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    if (errors[field]) {
      setErrors((prev) => ({ ...prev, [field]: undefined }));
    }
  };

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
    >
      <PageHeader title="申请做市商 (2/3)" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <Text style={styles.stepTitle}>第二步：提交资料</Text>

        {timeLeft && (
          <View style={styles.timerContainer}>
            <Text style={styles.timerIcon}>⏱️</Text>
            <Text style={styles.timerText}>请在 {timeLeft} 内完成提交</Text>
          </View>
        )}

        {/* 实名信息 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>实名信息</Text>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>真实姓名</Text>
            <TextInput
              style={[styles.input, errors.realName && styles.inputError]}
              placeholder="请输入真实姓名"
              value={formData.realName}
              onChangeText={(v) => updateField('realName', v)}
            />
            {errors.realName && <Text style={styles.errorText}>{errors.realName}</Text>}
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>身份证号</Text>
            <TextInput
              style={[styles.input, errors.idCardNumber && styles.inputError]}
              placeholder="请输入18位身份证号"
              value={formData.idCardNumber}
              onChangeText={(v) => updateField('idCardNumber', v)}
              maxLength={18}
              autoCapitalize="characters"
            />
            {errors.idCardNumber && <Text style={styles.errorText}>{errors.idCardNumber}</Text>}
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>出生日期</Text>
            <TextInput
              style={[styles.input, errors.birthday && styles.inputError]}
              placeholder="YYYY-MM-DD"
              value={formData.birthday}
              onChangeText={(v) => updateField('birthday', v)}
              maxLength={10}
            />
            {errors.birthday && <Text style={styles.errorText}>{errors.birthday}</Text>}
          </View>
        </Card>

        {/* 收款信息 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>收款信息</Text>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>TRON 地址 (TRC20)</Text>
            <TextInput
              style={[styles.input, errors.tronAddress && styles.inputError]}
              placeholder="T..."
              value={formData.tronAddress}
              onChangeText={(v) => updateField('tronAddress', v)}
              autoCapitalize="none"
            />
            {errors.tronAddress && <Text style={styles.errorText}>{errors.tronAddress}</Text>}
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>微信号</Text>
            <TextInput
              style={[styles.input, errors.wechatId && styles.inputError]}
              placeholder="请输入微信号"
              value={formData.wechatId}
              onChangeText={(v) => updateField('wechatId', v)}
              autoCapitalize="none"
            />
            {errors.wechatId && <Text style={styles.errorText}>{errors.wechatId}</Text>}
          </View>
        </Card>

        {/* EPAY 配置 */}
        <Card style={styles.section}>
          <Text style={styles.sectionTitle}>EPAY 配置 (可选)</Text>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>商户号</Text>
            <TextInput
              style={styles.input}
              placeholder="请输入 EPAY 商户号"
              value={formData.epayNo}
              onChangeText={(v) => updateField('epayNo', v)}
            />
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>密钥</Text>
            <TextInput
              style={[styles.input, errors.epayKey && styles.inputError]}
              placeholder="请输入 EPAY 密钥"
              value={formData.epayKey}
              onChangeText={(v) => updateField('epayKey', v)}
              secureTextEntry
            />
            {errors.epayKey && <Text style={styles.errorText}>{errors.epayKey}</Text>}
          </View>
        </Card>

        {/* 提交按钮 */}
        <Button
          title="提交资料"
          onPress={handleSubmit}
          loading={isLoading}
          disabled={isLoading}
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
  content: {
    flex: 1,
    padding: 16,
  },
  stepTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 12,
  },
  timerContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FF950020',
    padding: 12,
    borderRadius: 8,
    marginBottom: 20,
  },
  timerIcon: {
    fontSize: 16,
    marginRight: 8,
  },
  timerText: {
    fontSize: 14,
    color: '#FF9500',
    fontWeight: '500',
  },
  section: {
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 16,
  },
  inputGroup: {
    marginBottom: 16,
  },
  inputLabel: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 15,
    color: '#1C1C1E',
  },
  inputError: {
    borderWidth: 1,
    borderColor: '#FF3B30',
  },
  errorText: {
    fontSize: 12,
    color: '#FF3B30',
    marginTop: 4,
  },
});
