/**
 * 择偶条件设置页面
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
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { matchmakingService, MatchPreferences } from '@/services/matchmaking.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';

const EDUCATION_LEVELS = [
  { value: 0, label: '不限' },
  { value: 1, label: '高中及以下' },
  { value: 2, label: '大专' },
  { value: 3, label: '本科' },
  { value: 4, label: '硕士' },
  { value: 5, label: '博士' },
];

const COMMON_LOCATIONS = [
  '北京', '上海', '广州', '深圳', '杭州', '成都', '重庆', '武汉',
  '南京', '西安', '苏州', '天津', '郑州', '长沙', '东莞', '青岛',
];

export default function PreferencesPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');

  // 表单状态
  const [minAge, setMinAge] = useState('22');
  const [maxAge, setMaxAge] = useState('35');
  const [minHeight, setMinHeight] = useState('155');
  const [maxHeight, setMaxHeight] = useState('185');
  const [minEducation, setMinEducation] = useState(0);
  const [selectedLocations, setSelectedLocations] = useState<string[]>([]);
  const [minIncome, setMinIncome] = useState('');
  const [housingRequired, setHousingRequired] = useState(false);

  const loadPreferences = useCallback(async () => {
    if (!address) return;

    try {
      const prefs = await matchmakingService.getPreferences(address);
      if (prefs) {
        setMinAge(prefs.minAge.toString());
        setMaxAge(prefs.maxAge.toString());
        setMinHeight(prefs.minHeight.toString());
        setMaxHeight(prefs.maxHeight.toString());
        setMinEducation(prefs.minEducation);
        setSelectedLocations(prefs.locations);
        setMinIncome(prefs.minIncome?.toString() || '');
        setHousingRequired(prefs.housingRequired);
      }
    } catch (error) {
      console.error('Load preferences error:', error);
    } finally {
      setLoading(false);
    }
  }, [address]);

  useEffect(() => {
    loadPreferences();
  }, [loadPreferences]);

  const toggleLocation = (location: string) => {
    if (selectedLocations.includes(location)) {
      setSelectedLocations(selectedLocations.filter(l => l !== location));
    } else if (selectedLocations.length < 5) {
      setSelectedLocations([...selectedLocations, location]);
    } else {
      Alert.alert('提示', '最多选择5个地区');
    }
  };

  const handleSave = async () => {
    // 验证
    const minAgeNum = parseInt(minAge);
    const maxAgeNum = parseInt(maxAge);
    const minHeightNum = parseInt(minHeight);
    const maxHeightNum = parseInt(maxHeight);

    if (isNaN(minAgeNum) || isNaN(maxAgeNum) || minAgeNum < 18 || maxAgeNum > 80) {
      Alert.alert('提示', '请输入有效的年龄范围（18-80岁）');
      return;
    }

    if (minAgeNum > maxAgeNum) {
      Alert.alert('提示', '最小年龄不能大于最大年龄');
      return;
    }

    if (isNaN(minHeightNum) || isNaN(maxHeightNum) || minHeightNum < 140 || maxHeightNum > 220) {
      Alert.alert('提示', '请输入有效的身高范围（140-220cm）');
      return;
    }

    if (minHeightNum > maxHeightNum) {
      Alert.alert('提示', '最小身高不能大于最大身高');
      return;
    }

    if (!isSignerUnlocked()) {
      setShowUnlockDialog(true);
      return;
    }

    await executeSave();
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      await executeSave();
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeSave = async () => {
    setShowTxStatus(true);
    setTxStatus('正在保存择偶条件...');

    try {
      const preferences: MatchPreferences = {
        minAge: parseInt(minAge),
        maxAge: parseInt(maxAge),
        minHeight: parseInt(minHeight),
        maxHeight: parseInt(maxHeight),
        minEducation,
        locations: selectedLocations,
        minIncome: minIncome ? parseInt(minIncome) : undefined,
        housingRequired,
      };

      await matchmakingService.updatePreferences(preferences, (status) => setTxStatus(status));

      setTxStatus('保存成功！');
      setTimeout(() => {
        setShowTxStatus(false);
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('保存失败', error.message || '请稍后重试');
    }
  };

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="择偶条件" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="择偶条件" showBack />

      <ScrollView style={styles.content}>
        {/* 年龄范围 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>年龄范围</Text>
          <View style={styles.rangeRow}>
            <TextInput
              style={styles.rangeInput}
              value={minAge}
              onChangeText={setMinAge}
              keyboardType="number-pad"
              placeholder="最小"
            />
            <Text style={styles.rangeSeparator}>-</Text>
            <TextInput
              style={styles.rangeInput}
              value={maxAge}
              onChangeText={setMaxAge}
              keyboardType="number-pad"
              placeholder="最大"
            />
            <Text style={styles.rangeUnit}>岁</Text>
          </View>
        </View>

        {/* 身高范围 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>身高范围</Text>
          <View style={styles.rangeRow}>
            <TextInput
              style={styles.rangeInput}
              value={minHeight}
              onChangeText={setMinHeight}
              keyboardType="number-pad"
              placeholder="最小"
            />
            <Text style={styles.rangeSeparator}>-</Text>
            <TextInput
              style={styles.rangeInput}
              value={maxHeight}
              onChangeText={setMaxHeight}
              keyboardType="number-pad"
              placeholder="最大"
            />
            <Text style={styles.rangeUnit}>cm</Text>
          </View>
        </View>

        {/* 学历要求 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>最低学历</Text>
          <View style={styles.optionsRow}>
            {EDUCATION_LEVELS.map((edu) => (
              <Pressable
                key={edu.value}
                style={[
                  styles.optionChip,
                  minEducation === edu.value && styles.optionChipSelected,
                ]}
                onPress={() => setMinEducation(edu.value)}
              >
                <Text style={[
                  styles.optionChipText,
                  minEducation === edu.value && styles.optionChipTextSelected,
                ]}>
                  {edu.label}
                </Text>
              </Pressable>
            ))}
          </View>
        </View>

        {/* 地区偏好 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>地区偏好（最多5个）</Text>
          <View style={styles.optionsRow}>
            {COMMON_LOCATIONS.map((location) => (
              <Pressable
                key={location}
                style={[
                  styles.optionChip,
                  selectedLocations.includes(location) && styles.optionChipSelected,
                ]}
                onPress={() => toggleLocation(location)}
              >
                <Text style={[
                  styles.optionChipText,
                  selectedLocations.includes(location) && styles.optionChipTextSelected,
                ]}>
                  {location}
                </Text>
              </Pressable>
            ))}
          </View>
          {selectedLocations.length > 0 && (
            <Text style={styles.selectedHint}>
              已选: {selectedLocations.join('、')}
            </Text>
          )}
        </View>

        {/* 收入要求 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>最低年收入（可选）</Text>
          <View style={styles.incomeRow}>
            <TextInput
              style={styles.incomeInput}
              value={minIncome}
              onChangeText={setMinIncome}
              keyboardType="number-pad"
              placeholder="不限"
            />
            <Text style={styles.incomeUnit}>万元/年</Text>
          </View>
        </View>

        {/* 房产要求 */}
        <View style={styles.section}>
          <Pressable
            style={styles.switchRow}
            onPress={() => setHousingRequired(!housingRequired)}
          >
            <Text style={styles.sectionTitle}>要求有房</Text>
            <View style={[
              styles.switchTrack,
              housingRequired && styles.switchTrackActive,
            ]}>
              <View style={[
                styles.switchThumb,
                housingRequired && styles.switchThumbActive,
              ]} />
            </View>
          </Pressable>
        </View>

        {/* 保存按钮 */}
        <Pressable style={styles.saveButton} onPress={handleSave}>
          <Text style={styles.saveButtonText}>保存设置</Text>
        </Pressable>

        <View style={{ height: 32 }} />
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
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  content: {
    flex: 1,
  },
  section: {
    backgroundColor: '#fff',
    marginTop: 16,
    marginHorizontal: 16,
    borderRadius: 12,
    padding: 16,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  rangeRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  rangeInput: {
    flex: 1,
    height: 44,
    backgroundColor: '#f8f8f8',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 16,
    textAlign: 'center',
  },
  rangeSeparator: {
    fontSize: 18,
    color: '#999',
    marginHorizontal: 12,
  },
  rangeUnit: {
    fontSize: 14,
    color: '#666',
    marginLeft: 8,
  },
  optionsRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginHorizontal: -4,
  },
  optionChip: {
    paddingHorizontal: 14,
    paddingVertical: 8,
    backgroundColor: '#f8f8f8',
    borderRadius: 16,
    margin: 4,
  },
  optionChipSelected: {
    backgroundColor: THEME_COLOR,
  },
  optionChipText: {
    fontSize: 13,
    color: '#666',
  },
  optionChipTextSelected: {
    color: '#fff',
  },
  selectedHint: {
    fontSize: 12,
    color: THEME_COLOR,
    marginTop: 8,
  },
  incomeRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  incomeInput: {
    flex: 1,
    height: 44,
    backgroundColor: '#f8f8f8',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 16,
  },
  incomeUnit: {
    fontSize: 14,
    color: '#666',
    marginLeft: 8,
  },
  switchRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  switchTrack: {
    width: 50,
    height: 28,
    backgroundColor: '#e0e0e0',
    borderRadius: 14,
    padding: 2,
  },
  switchTrackActive: {
    backgroundColor: THEME_COLOR,
  },
  switchThumb: {
    width: 24,
    height: 24,
    backgroundColor: '#fff',
    borderRadius: 12,
  },
  switchThumbActive: {
    transform: [{ translateX: 22 }],
  },
  saveButton: {
    backgroundColor: THEME_COLOR,
    marginHorizontal: 16,
    marginTop: 24,
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  saveButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
