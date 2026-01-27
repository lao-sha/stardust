/**
 * 星尘玄鉴 - 个人资料编辑页面
 * 对应 membership pallet 的 MemberProfile
 * 主题色：金棕色 #B2955D
 */

import { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  TextInput,
  Alert,
  Platform,
  Modal,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { BottomNavBar } from '@/components/BottomNavBar';
import { useUserStore } from '@/stores/user.store';
import { Card, Button } from '@/components/common';
import { useAsync } from '@/hooks';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_BG = '#F5F5F7';

// 十二时辰
const SHICHEN = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

export default function ProfileEditPage() {
  const router = useRouter();
  const { myProfile, updateProfile } = useUserStore();
  const { execute, isLoading } = useAsync();

  // 个人资料状态 (对应 membership pallet)
  const [profile, setProfile] = useState({
    displayName: myProfile?.nickname || '星尘用户',
    avatarCid: myProfile?.avatarCid || '',
    signature: myProfile?.signature || '',
    gender: null as 'male' | 'female' | 'other' | null,
    birthYear: '',
    birthMonth: '',
    birthDay: '',
    birthHour: '',
    longitude: '',
    latitude: '',
    isProvider: false,
  });

  // 加载用户资料
  useEffect(() => {
    if (myProfile) {
      setProfile((prev) => ({
        ...prev,
        displayName: myProfile.nickname || prev.displayName,
        avatarCid: myProfile.avatarCid || prev.avatarCid,
        signature: myProfile.signature || prev.signature,
      }));
    }
  }, [myProfile]);

  // 保存成功弹窗状态
  const [saveModalVisible, setSaveModalVisible] = useState(false);
  const [errorModalVisible, setErrorModalVisible] = useState(false);
  const [errorMessage, setErrorMessage] = useState('');

  // 获取时辰显示
  const getShichenText = (hour: string) => {
    if (!hour) return '';
    const h = parseInt(hour);
    if (isNaN(h) || h < 0 || h > 23) return '';
    const index = Math.floor((h + 1) % 24 / 2);
    return `(${SHICHEN[index]}时)`;
  };

  // 打开个人资料编辑页面
  const handleBack = () => {
    if (router.canGoBack()) {
      router.back();
    } else {
      router.push('/profile');
    }
  };

  // 保存个人资料
  const handleSave = async () => {
    // 验证数据
    if (profile.birthYear && (parseInt(profile.birthYear) < 1900 || parseInt(profile.birthYear) > 2100)) {
      setErrorMessage('出生年份应在 1900-2100 之间');
      setErrorModalVisible(true);
      return;
    }
    if (profile.birthMonth && (parseInt(profile.birthMonth) < 1 || parseInt(profile.birthMonth) > 12)) {
      setErrorMessage('月份应在 1-12 之间');
      setErrorModalVisible(true);
      return;
    }
    if (profile.birthDay && (parseInt(profile.birthDay) < 1 || parseInt(profile.birthDay) > 31)) {
      setErrorMessage('日期应在 1-31 之间');
      setErrorModalVisible(true);
      return;
    }
    if (profile.birthHour && (parseInt(profile.birthHour) < 0 || parseInt(profile.birthHour) > 23)) {
      setErrorMessage('时辰应在 0-23 之间');
      setErrorModalVisible(true);
      return;
    }

    await execute(async () => {
      // 更新聊天用户资料
      await updateProfile({
        nickname: profile.displayName,
        avatarCid: profile.avatarCid || undefined,
        signature: profile.signature || undefined,
      });

      // TODO: 调用链上交易保存命理资料到 membership pallet
      // 这部分需要单独的 API 调用

      setSaveModalVisible(true);
    });
  };

  // 确认保存后返回
  const handleConfirmSave = () => {
    setSaveModalVisible(false);
    if (router.canGoBack()) {
      router.back();
    } else {
      router.push('/profile');
    }
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.navBtn} onPress={handleBack}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>编辑命理资料</Text>
        <Pressable style={styles.navBtn} onPress={handleSave}>
          <Text style={styles.saveText}>保存</Text>
        </Pressable>
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false} contentContainerStyle={styles.scrollContent}>
        {/* 昵称和签名 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>基本信息</Text>
          <Card>
            <View style={styles.formItem}>
              <Text style={styles.formLabel}>昵称</Text>
              <TextInput
                style={styles.formInput}
                value={profile.displayName}
                onChangeText={(v) => setProfile(prev => ({ ...prev, displayName: v }))}
                placeholder="请输入昵称"
                placeholderTextColor="#999"
                maxLength={64}
              />
            </View>
            <View style={styles.divider} />
            <View style={styles.formItem}>
              <Text style={styles.formLabel}>个性签名</Text>
              <TextInput
                style={styles.formInput}
                value={profile.signature}
                onChangeText={(v) => setProfile(prev => ({ ...prev, signature: v }))}
                placeholder="请输入个性签名"
                placeholderTextColor="#999"
                maxLength={200}
                multiline
              />
            </View>
          </Card>
        </View>

        {/* 性别选择 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>性别</Text>
          <Card>
            <View style={styles.genderRow}>
              {(['male', 'female', 'other'] as const).map((g) => (
                <Pressable
                  key={g}
                  style={[
                    styles.genderOption,
                    profile.gender === g && styles.genderOptionActive,
                  ]}
                  onPress={() => setProfile(prev => ({ ...prev, gender: g }))}
                >
                  <Ionicons
                    name={g === 'male' ? 'male' : g === 'female' ? 'female' : 'male-female'}
                    size={20}
                    color={profile.gender === g ? THEME_COLOR : '#999'}
                  />
                  <Text style={[
                    styles.genderText,
                    profile.gender === g && styles.genderTextActive,
                  ]}>
                    {g === 'male' ? '男' : g === 'female' ? '女' : '其他'}
                  </Text>
                </Pressable>
              ))}
            </View>
          </Card>
        </View>

        {/* 出生日期 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>出生日期（公历）</Text>
          <Card>
            <View style={styles.dateRow}>
              <View style={styles.dateItem}>
                <TextInput
                  style={styles.dateInput}
                  value={profile.birthYear}
                  onChangeText={(v) => setProfile(prev => ({ ...prev, birthYear: v.replace(/[^0-9]/g, '') }))}
                  placeholder="1990"
                  placeholderTextColor="#999"
                  keyboardType="numeric"
                  maxLength={4}
                />
                <Text style={styles.dateUnit}>年</Text>
              </View>
              <View style={styles.dateItem}>
                <TextInput
                  style={styles.dateInput}
                  value={profile.birthMonth}
                  onChangeText={(v) => setProfile(prev => ({ ...prev, birthMonth: v.replace(/[^0-9]/g, '') }))}
                  placeholder="01"
                  placeholderTextColor="#999"
                  keyboardType="numeric"
                  maxLength={2}
                />
                <Text style={styles.dateUnit}>月</Text>
              </View>
              <View style={styles.dateItem}>
                <TextInput
                  style={styles.dateInput}
                  value={profile.birthDay}
                  onChangeText={(v) => setProfile(prev => ({ ...prev, birthDay: v.replace(/[^0-9]/g, '') }))}
                  placeholder="01"
                  placeholderTextColor="#999"
                  keyboardType="numeric"
                  maxLength={2}
                />
                <Text style={styles.dateUnit}>日</Text>
              </View>
            </View>
          </Card>
        </View>

        {/* 出生时辰 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>出生时辰</Text>
          <Card>
            <View style={styles.hourRow}>
              <TextInput
                style={styles.hourInput}
                value={profile.birthHour}
                onChangeText={(v) => setProfile(prev => ({ ...prev, birthHour: v.replace(/[^0-9]/g, '') }))}
                placeholder="如: 14"
                placeholderTextColor="#999"
                keyboardType="numeric"
                maxLength={2}
              />
              <Text style={styles.hourUnit}>时</Text>
              {profile.birthHour && (
                <View style={styles.shichenBadge}>
                  <Text style={styles.shichenText}>{getShichenText(profile.birthHour)}</Text>
                </View>
              )}
            </View>
            <Text style={styles.formHint}>24小时制，不确定可留空</Text>
          </Card>
        </View>

        {/* 出生地点 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>出生地点（经纬度）</Text>
          <Card>
            <View style={styles.locationRow}>
              <View style={styles.locationItem}>
                <Text style={styles.locationLabel}>经度</Text>
                <TextInput
                  style={styles.locationInput}
                  value={profile.longitude}
                  onChangeText={(v) => setProfile(prev => ({ ...prev, longitude: v }))}
                  placeholder="如: 116.4074"
                  placeholderTextColor="#999"
                  keyboardType="decimal-pad"
                />
              </View>
              <View style={styles.locationItem}>
                <Text style={styles.locationLabel}>纬度</Text>
                <TextInput
                  style={styles.locationInput}
                  value={profile.latitude}
                  onChangeText={(v) => setProfile(prev => ({ ...prev, latitude: v }))}
                  placeholder="如: 39.9042"
                  placeholderTextColor="#999"
                  keyboardType="decimal-pad"
                />
              </View>
            </View>
            <Text style={styles.formHint}>用于计算真太阳时，可通过地图获取精确坐标</Text>
            <Pressable style={styles.mapBtn} onPress={() => Alert.alert('提示', '地图选点功能即将上线')}>
              <Ionicons name="location-outline" size={18} color={THEME_COLOR} />
              <Text style={styles.mapBtnText}>从地图选取</Text>
            </Pressable>
          </Card>
        </View>

        {/* 服务提供者 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>服务提供者</Text>
          <Card>
            <Pressable
              style={styles.switchRow}
              onPress={() => setProfile(prev => ({ ...prev, isProvider: !prev.isProvider }))}
            >
              <View style={styles.switchInfo}>
                <Text style={styles.switchTitle}>申请成为服务提供者</Text>
                <Text style={styles.switchDesc}>提供命理咨询服务并获得收益</Text>
              </View>
              <View style={[styles.switch, profile.isProvider && styles.switchActive]}>
                <View style={[styles.switchThumb, profile.isProvider && styles.switchThumbActive]} />
              </View>
            </Pressable>
          </Card>
        </View>

        {/* 隐私提示 */}
        <Card style={styles.privacyCard}>
          <Ionicons name="shield-checkmark" size={20} color={THEME_COLOR} />
          <View style={styles.privacyContent}>
            <Text style={styles.privacyTitle}>隐私保护</Text>
            <Text style={styles.privacyText}>
              您的出生信息将加密存储在链上，仅您本人可查看完整信息。占卜师只能看到必要的命理数据。
            </Text>
          </View>
        </Card>

        {/* 底部间距 */}
        <View style={{ height: 100 }} />
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />

      {/* 保存成功弹窗 */}
      <Modal visible={saveModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalIcon}>
              <Ionicons name="checkmark-circle" size={48} color="#52c41a" />
            </View>
            <Text style={styles.modalTitle}>保存成功</Text>
            <Text style={styles.modalText}>个人资料已保存</Text>
            <Text style={styles.modalTip}>提示：修改资料需要发起链上交易</Text>
            <Pressable style={styles.modalBtn} onPress={handleConfirmSave}>
              <Text style={styles.modalBtnText}>确定</Text>
            </Pressable>
          </View>
        </View>
      </Modal>

      {/* 错误提示弹窗 */}
      <Modal visible={errorModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalIcon}>
              <Ionicons name="alert-circle" size={48} color="#ff4d4f" />
            </View>
            <Text style={styles.modalTitle}>错误</Text>
            <Text style={styles.modalText}>{errorMessage}</Text>
            <Pressable style={styles.modalBtn} onPress={() => setErrorModalVisible(false)}>
              <Text style={styles.modalBtnText}>确定</Text>
            </Pressable>
          </View>
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  navBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: Platform.select({ ios: 50, android: 40, default: 20 }),
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  navBtn: {
    padding: 4,
    minWidth: 50,
  },
  navTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  saveText: {
    fontSize: 16,
    color: THEME_COLOR,
    fontWeight: '600',
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 16,
  },
  section: {
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#666',
    marginBottom: 8,
    paddingHorizontal: 4,
  },
  formItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  formLabel: {
    fontSize: 15,
    color: '#333',
    width: 60,
  },
  formInput: {
    flex: 1,
    height: 44,
    fontSize: 15,
    color: '#333',
  },
  formHint: {
    fontSize: 12,
    color: '#999',
    marginTop: 8,
  },
  divider: {
    height: 1,
    backgroundColor: '#F5F5F5',
    marginVertical: 8,
  },
  // 性别
  genderRow: {
    flexDirection: 'row',
    gap: 12,
  },
  genderOption: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 6,
    paddingVertical: 14,
    borderRadius: 8,
    backgroundColor: '#f5f5f5',
    borderWidth: 2,
    borderColor: 'transparent',
  },
  genderOptionActive: {
    backgroundColor: THEME_COLOR + '15',
    borderColor: THEME_COLOR,
  },
  genderText: {
    fontSize: 15,
    color: '#666',
  },
  genderTextActive: {
    color: THEME_COLOR,
    fontWeight: '600',
  },
  // 日期
  dateRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 4,
  },
  dateItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  dateInput: {
    width: 70,
    height: 40,
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 8,
    fontSize: 15,
    textAlign: 'center',
    color: '#333',
  },
  dateUnit: {
    fontSize: 14,
    color: '#666',
    marginLeft: 4,
    marginRight: 8,
  },
  // 时辰
  hourRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  hourInput: {
    width: 80,
    height: 44,
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 16,
    textAlign: 'center',
    color: '#333',
  },
  hourUnit: {
    fontSize: 14,
    color: '#666',
    marginLeft: 6,
  },
  shichenBadge: {
    marginLeft: 12,
    paddingHorizontal: 10,
    paddingVertical: 4,
    backgroundColor: THEME_COLOR + '20',
    borderRadius: 12,
  },
  shichenText: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  // 地点
  locationRow: {
    flexDirection: 'row',
    gap: 16,
  },
  locationItem: {
    flex: 1,
  },
  locationLabel: {
    fontSize: 13,
    color: '#999',
    marginBottom: 6,
  },
  locationInput: {
    height: 44,
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 15,
    color: '#333',
  },
  mapBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 6,
    marginTop: 12,
    paddingVertical: 10,
    backgroundColor: THEME_COLOR + '10',
    borderRadius: 8,
  },
  mapBtnText: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  // 开关
  switchRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  switchInfo: {
    flex: 1,
  },
  switchTitle: {
    fontSize: 15,
    color: '#333',
    fontWeight: '500',
  },
  switchDesc: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
  switch: {
    width: 50,
    height: 28,
    borderRadius: 14,
    backgroundColor: '#e0e0e0',
    padding: 2,
  },
  switchActive: {
    backgroundColor: THEME_COLOR,
  },
  switchThumb: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: '#FFF',
  },
  switchThumbActive: {
    marginLeft: 22,
  },
  // 隐私提示
  privacyCard: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: 12,
    backgroundColor: THEME_COLOR + '10',
    marginTop: 4,
  },
  privacyContent: {
    flex: 1,
  },
  privacyTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME_COLOR,
    marginBottom: 4,
  },
  privacyText: {
    fontSize: 13,
    color: '#666',
    lineHeight: 20,
  },
  // 弹窗样式
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modalContent: {
    backgroundColor: '#FFF',
    borderRadius: 16,
    padding: 24,
    width: '100%',
    maxWidth: 320,
    alignItems: 'center',
  },
  modalIcon: {
    marginBottom: 16,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  modalText: {
    fontSize: 15,
    color: '#666',
    textAlign: 'center',
    marginBottom: 8,
  },
  modalTip: {
    fontSize: 13,
    color: '#999',
    textAlign: 'center',
    marginBottom: 20,
  },
  modalBtn: {
    backgroundColor: THEME_COLOR,
    paddingVertical: 12,
    paddingHorizontal: 48,
    borderRadius: 8,
  },
  modalBtnText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFF',
  },
});
