/**
 * 创建婚恋资料页面
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
import { matchmakingService, Gender, MatchmakingPrivacyMode } from '@/services/matchmaking.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

export default function CreateProfilePage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');

  // 表单状态
  const [nickname, setNickname] = useState('');
  const [gender, setGender] = useState<Gender | null>(null);
  const [birthYear, setBirthYear] = useState('');
  const [height, setHeight] = useState('');
  const [location, setLocation] = useState('');
  const [bio, setBio] = useState('');
  const [privacyMode, setPrivacyMode] = useState<MatchmakingPrivacyMode>(MatchmakingPrivacyMode.MembersOnly);

  // 验证
  const nicknameValid = nickname.length >= 2 && nickname.length <= 32;
  const genderValid = gender !== null;
  const birthYearValid = /^\d{4}$/.test(birthYear) && parseInt(birthYear) >= 1950 && parseInt(birthYear) <= 2010;
  const heightValid = /^\d{2,3}$/.test(height) && parseInt(height) >= 140 && parseInt(height) <= 220;
  const locationValid = location.length >= 2 && location.length <= 64;
  const bioValid = bio.length <= 500;
  const formValid = nicknameValid && genderValid && birthYearValid && heightValid && locationValid && bioValid;

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
    setTxStatus('正在创建资料...');

    try {
      await matchmakingService.createProfile(
        gender!,
        parseInt(birthYear),
        location,
        parseInt(height),
        0, // education
        '', // occupation
        0, // income
        0, // housingStatus
        (status: string) => setTxStatus(status)
      );

      setTxStatus('创建成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        router.replace('/matchmaking' as any);
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('创建失败', error.message || '请稍后重试');
    } finally {
      setLoading(false);
    }
  };

  return (
    <View style={styles.container}>
      <PageHeader title="创建资料" showBack />

      <ScrollView style={styles.content}>
        {/* 昵称 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>昵称 *</Text>
          <TextInput
            style={[styles.input, !nicknameValid && nickname.length > 0 && styles.inputError]}
            value={nickname}
            onChangeText={setNickname}
            placeholder="2-32个字符"
            maxLength={32}
          />
        </View>

        {/* 性别 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>性别 *</Text>
          <View style={styles.genderRow}>
            <Pressable
              style={[styles.genderButton, gender === Gender.Male && styles.genderButtonActive]}
              onPress={() => setGender(Gender.Male)}
            >
              <Ionicons
                name="male"
                size={24}
                color={gender === Gender.Male ? '#fff' : THEME_COLOR}
              />
              <Text style={[styles.genderText, gender === Gender.Male && styles.genderTextActive]}>
                男
              </Text>
            </Pressable>
            <Pressable
              style={[styles.genderButton, gender === Gender.Female && styles.genderButtonActive]}
              onPress={() => setGender(Gender.Female)}
            >
              <Ionicons
                name="female"
                size={24}
                color={gender === Gender.Female ? '#fff' : THEME_COLOR}
              />
              <Text style={[styles.genderText, gender === Gender.Female && styles.genderTextActive]}>
                女
              </Text>
            </Pressable>
          </View>
        </View>

        {/* 出生年份 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>出生年份 *</Text>
          <TextInput
            style={[styles.input, !birthYearValid && birthYear.length > 0 && styles.inputError]}
            value={birthYear}
            onChangeText={setBirthYear}
            placeholder="如: 1990"
            keyboardType="numeric"
            maxLength={4}
          />
        </View>

        {/* 身高 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>身高 (cm) *</Text>
          <TextInput
            style={[styles.input, !heightValid && height.length > 0 && styles.inputError]}
            value={height}
            onChangeText={setHeight}
            placeholder="如: 170"
            keyboardType="numeric"
            maxLength={3}
          />
        </View>

        {/* 所在地 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>所在地 *</Text>
          <TextInput
            style={[styles.input, !locationValid && location.length > 0 && styles.inputError]}
            value={location}
            onChangeText={setLocation}
            placeholder="如: 北京市朝阳区"
            maxLength={64}
          />
        </View>

        {/* 个人简介 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>个人简介</Text>
          <TextInput
            style={[styles.input, styles.textArea]}
            value={bio}
            onChangeText={setBio}
            placeholder="介绍一下自己..."
            multiline
            numberOfLines={4}
            maxLength={500}
          />
          <Text style={styles.charCount}>{bio.length}/500</Text>
        </View>

        {/* 隐私设置 */}
        <View style={styles.formGroup}>
          <Text style={styles.label}>隐私设置</Text>
          <View style={styles.privacyOptions}>
            <Pressable
              style={[styles.privacyOption, privacyMode === MatchmakingPrivacyMode.Public && styles.privacyOptionActive]}
              onPress={() => setPrivacyMode(MatchmakingPrivacyMode.Public)}
            >
              <Ionicons
                name="globe-outline"
                size={20}
                color={privacyMode === MatchmakingPrivacyMode.Public ? THEME_COLOR : '#666'}
              />
              <Text style={[styles.privacyText, privacyMode === MatchmakingPrivacyMode.Public && styles.privacyTextActive]}>
                公开
              </Text>
            </Pressable>
            <Pressable
              style={[styles.privacyOption, privacyMode === MatchmakingPrivacyMode.MembersOnly && styles.privacyOptionActive]}
              onPress={() => setPrivacyMode(MatchmakingPrivacyMode.MembersOnly)}
            >
              <Ionicons
                name="people-outline"
                size={20}
                color={privacyMode === MatchmakingPrivacyMode.MembersOnly ? THEME_COLOR : '#666'}
              />
              <Text style={[styles.privacyText, privacyMode === MatchmakingPrivacyMode.MembersOnly && styles.privacyTextActive]}>
                仅会员
              </Text>
            </Pressable>
            <Pressable
              style={[styles.privacyOption, privacyMode === MatchmakingPrivacyMode.MatchedOnly && styles.privacyOptionActive]}
              onPress={() => setPrivacyMode(MatchmakingPrivacyMode.MatchedOnly)}
            >
              <Ionicons
                name="heart-outline"
                size={20}
                color={privacyMode === MatchmakingPrivacyMode.MatchedOnly ? THEME_COLOR : '#666'}
              />
              <Text style={[styles.privacyText, privacyMode === MatchmakingPrivacyMode.MatchedOnly && styles.privacyTextActive]}>
                匹配后
              </Text>
            </Pressable>
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
            <Text style={styles.submitButtonText}>创建资料</Text>
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
    height: 100,
    textAlignVertical: 'top',
  },
  charCount: {
    fontSize: 12,
    color: '#999',
    textAlign: 'right',
    marginTop: 4,
  },
  genderRow: {
    flexDirection: 'row',
  },
  genderButton: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#fff',
    borderRadius: 8,
    padding: 12,
    marginRight: 8,
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  genderButtonActive: {
    backgroundColor: THEME_COLOR,
  },
  genderText: {
    fontSize: 16,
    color: THEME_COLOR,
    marginLeft: 8,
  },
  genderTextActive: {
    color: '#fff',
  },
  privacyOptions: {
    flexDirection: 'row',
  },
  privacyOption: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#fff',
    borderRadius: 8,
    padding: 10,
    marginRight: 8,
    borderWidth: 1,
    borderColor: '#e0e0e0',
  },
  privacyOptionActive: {
    borderColor: THEME_COLOR,
    backgroundColor: '#f8f4e8',
  },
  privacyText: {
    fontSize: 13,
    color: '#666',
    marginLeft: 4,
  },
  privacyTextActive: {
    color: THEME_COLOR,
  },
  submitButton: {
    backgroundColor: THEME_COLOR,
    borderRadius: 25,
    padding: 16,
    alignItems: 'center',
    marginTop: 20,
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
