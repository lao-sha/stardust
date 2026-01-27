/**
 * 星尘玄鉴 - 我的钱包页面
 * 复刻自 stardust-dapp MyWalletPage
 * 主题色：金棕色 #B2955D
 */

import { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  Alert,
  Modal,
  TextInput,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWalletStore } from '@/stores';
import * as Clipboard from 'expo-clipboard';
import { QRCode } from '@/components/QRCode';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#C9B07A';
const THEME_BG = '#f5f5f5';

// 菜单项接口
interface MenuItem {
  icon: keyof typeof Ionicons.glyphMap;
  title: string;
  badge?: number;
  onPress: () => void;
}

export default function ProfilePage() {
  const router = useRouter();
  const { isReady, hasWallet, isLocked, address, lockWallet, deleteWallet, initialize } = useWalletStore();

  const [nickname, setNickname] = useState('星尘用户');
  const [editModalVisible, setEditModalVisible] = useState(false);
  const [receiveModalVisible, setReceiveModalVisible] = useState(false);
  const [profileModalVisible, setProfileModalVisible] = useState(false);
  const [newNickname, setNewNickname] = useState('');
  const [language, setLanguage] = useState('简体中文');

  // 个人资料摘要 (对应 membership pallet)
  const [profile, setProfile] = useState({
    gender: null as 'male' | 'female' | 'other' | null,
    birthYear: '',
    birthMonth: '',
    birthDay: '',
    birthHour: '',
    longitude: '',
    latitude: '',
    isProvider: false,
  });

  // 初始化钱包状态
  useEffect(() => {
    if (!isReady) {
      initialize();
    }
  }, [isReady, initialize]);

  // 复制地址
  const handleCopyAddress = async () => {
    if (address) {
      await Clipboard.setStringAsync(address);
      Alert.alert('成功', '地址已复制到剪贴板');
    }
  };

  // 格式化地址
  const formatAddress = (addr: string | null) => {
    if (!addr) return '未连接';
    return `${addr.slice(0, 8)}...${addr.slice(-6)}`;
  };

  // 保存昵称
  const handleSaveNickname = () => {
    if (newNickname.trim()) {
      setNickname(newNickname.trim());
      setEditModalVisible(false);
      Alert.alert('成功', '昵称已保存');
    }
  };

  // 获取性别显示文本
  const getGenderText = (gender: 'male' | 'female' | 'other' | null) => {
    switch (gender) {
      case 'male': return '男';
      case 'female': return '女';
      case 'other': return '其他';
      default: return '未设置';
    }
  };

  // 计算资料完成度
  const getProfileCompleteness = () => {
    let count = 0;
    if (profile.gender) count++;
    if (profile.birthYear && profile.birthMonth && profile.birthDay) count++;
    if (profile.birthHour) count++;
    if (profile.longitude && profile.latitude) count++;
    return count;
  };

  // 获取出生日期显示文本
  const getBirthDateText = () => {
    if (profile.birthYear && profile.birthMonth && profile.birthDay) {
      return `${profile.birthYear}年${profile.birthMonth}月${profile.birthDay}日`;
    }
    return '未设置';
  };

  // 获取出生时辰显示文本
  const getBirthHourText = () => {
    if (profile.birthHour) {
      const SHICHEN = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];
      const h = parseInt(profile.birthHour);
      if (!isNaN(h) && h >= 0 && h <= 23) {
        const index = Math.floor((h + 1) % 24 / 2);
        return `${profile.birthHour}时 (${SHICHEN[index]}时)`;
      }
      return `${profile.birthHour}时`;
    }
    return '未设置';
  };

  // 打开个人资料编辑弹窗
  const handleOpenProfileEdit = () => {
    setProfileModalVisible(true);
  };

  // 保存个人资料
  const handleSaveProfile = () => {
    setProfileModalVisible(false);
    Alert.alert('成功', '命理资料已保存');
  };

  // 锁定钱包
  const handleLock = () => {
    lockWallet();
    router.replace('/auth/unlock');
  };

  // 删除钱包
  const handleDelete = () => {
    Alert.alert(
      '删除钱包',
      '确定要删除钱包吗？此操作无法撤销，请确保已备份助记词。',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '删除',
          style: 'destructive',
          onPress: async () => {
            await deleteWallet();
          },
        },
      ]
    );
  };

  // 菜单项配置
  const menuItems: MenuItem[] = [
    {
      icon: 'wallet-outline',
      title: '钱包管理',
      onPress: () => router.push('/wallet/manage'),
    },
    {
      icon: 'lock-closed-outline',
      title: '修改密码',
      onPress: () => Alert.alert('提示', '修改密码功能即将上线'),
    },
    {
      icon: 'shield-checkmark-outline',
      title: '隐私与授权',
      onPress: () => router.push('/profile/privacy'),
    },
    {
      icon: 'time-outline',
      title: '交易历史',
      onPress: () => router.push('/wallet/transactions'),
    },
    {
      icon: 'swap-horizontal-outline',
      title: '跨链桥接',
      onPress: () => router.push('/bridge/official'),
    },
    {
      icon: 'book-outline',
      title: '我的占卜记录',
      onPress: () => router.push('/divination/history'),
    },
    {
      icon: 'server-outline',
      title: '查上链网系统',
      onPress: () => Alert.alert('提示', '链上数据查询即将上线'),
    },
    {
      icon: 'storefront-outline',
      title: '占卜市场',
      onPress: () => router.push('/market'),
    },
    {
      icon: 'person-add-outline',
      title: '成为解卦师',
      onPress: () => router.push('/diviner/register'),
    },
    {
      icon: 'business-outline',
      title: '做市商管理中心',
      onPress: () => router.push('/maker'),
    },
    {
      icon: 'globe-outline',
      title: 'Web运营平台',
      onPress: () => Alert.alert('提示', '请在电脑端访问 governance.dustapps.net'),
    },
    {
      icon: 'people-outline',
      title: '联盟治理',
      onPress: () => Alert.alert('提示', '联盟治理功能即将上线'),
    },
    {
      icon: 'language-outline',
      title: '语言',
      onPress: () => {
        const newLang = language === '简体中文' ? '繁體中文' : language === '繁體中文' ? 'English' : '简体中文';
        setLanguage(newLang);
        Alert.alert('成功', `语言已切换为：${newLang}`);
      },
    },
    {
      icon: 'megaphone-outline',
      title: '公告',
      badge: 1,
      onPress: () => Alert.alert('提示', '公告功能即将上线'),
    },
    {
      icon: 'chatbubble-outline',
      title: '系统消息',
      onPress: () => Alert.alert('提示', '系统消息功能即将上线'),
    },
    {
      icon: 'information-circle-outline',
      title: '关于我们',
      onPress: () => Alert.alert('关于星尘玄鉴', '版本 1.0.0\n\n星尘玄鉴是基于区块链的玄学服务平台'),
    },
  ];

  // 加载中状态
  if (!isReady) {
    return (
      <View style={styles.container}>
        <View style={styles.loadingSection}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
      </View>
    );
  }

  // 没有钱包时显示创建/导入入口
  if (!hasWallet) {
    return (
      <View style={styles.container}>
        <View style={styles.welcomeSectionCentered}>
          <View style={styles.iconCircle}>
            <Ionicons name="wallet-outline" size={48} color={THEME_COLOR} />
          </View>
          <Text style={styles.welcomeTitle}>欢迎使用星尘玄鉴</Text>
          <Text style={styles.welcomeSubtitle}>创建或导入钱包以开始使用</Text>

          <View style={styles.buttonGroupInline}>
            <Pressable style={styles.primaryButton} onPress={() => router.push('/auth/create')}>
              <Ionicons name="add-circle-outline" size={24} color="#FFF" />
              <Text style={styles.primaryButtonText}>创建钱包</Text>
            </Pressable>

            <Pressable style={styles.secondaryButton} onPress={() => router.push('/auth/import')}>
              <Ionicons name="download-outline" size={24} color={THEME_COLOR} />
              <Text style={styles.secondaryButtonText}>导入钱包</Text>
            </Pressable>
          </View>
        </View>
      </View>
    );
  }

  // 钱包已锁定
  if (isLocked) {
    return (
      <View style={styles.container}>
        <View style={styles.welcomeSection}>
          <View style={styles.iconCircle}>
            <Ionicons name="lock-closed" size={48} color={THEME_COLOR} />
          </View>
          <Text style={styles.welcomeTitle}>钱包已锁定</Text>
          <Text style={styles.welcomeSubtitle}>请输入密码解锁钱包</Text>

          <Pressable style={styles.primaryButton} onPress={() => router.push('/auth/unlock')}>
            <Ionicons name="lock-open-outline" size={24} color="#FFF" />
            <Text style={styles.primaryButtonText}>解锁钱包</Text>
          </Pressable>
        </View>
      </View>
    );
  }

  // 主页面
  return (
    <View style={styles.container}>
      <ScrollView style={styles.scrollView} contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
        {/* 黑色顶部用户信息区域 */}
        <View style={styles.topHeader}>
          {/* 头像 */}
          <View style={styles.headerAvatar}>
            <Text style={styles.avatarText}>{nickname.charAt(0)}</Text>
          </View>

          {/* 用户信息 */}
          <View style={styles.headerUserInfo}>
            <View style={styles.headerUserRow}>
              <Text style={styles.headerNickname}>{nickname}</Text>
              <Pressable onPress={() => {
                setNewNickname(nickname);
                setEditModalVisible(true);
              }}>
                <Ionicons name="create-outline" size={16} color="rgba(255,255,255,0.6)" />
              </Pressable>
              <View style={styles.headerTag}>
                <Text style={styles.headerTagText}>会员</Text>
              </View>
              <View style={[styles.headerTag, styles.headerTagVip]}>
                <Text style={styles.headerTagVipText}>VIP</Text>
              </View>
            </View>
            <Text style={styles.headerAddress}>{formatAddress(address)}</Text>
          </View>

          {/* 通知图标 */}
          <Pressable style={styles.notificationBtn}>
            <Ionicons name="notifications-outline" size={22} color="rgba(255,255,255,0.8)" />
            <View style={styles.notificationBadge}>
              <Text style={styles.notificationBadgeText}>1</Text>
            </View>
          </Pressable>
        </View>

        {/* VIP会员卡片 */}
        <Pressable style={styles.vipCard}>
          <View style={styles.vipCardLeft}>
            <Text style={styles.vipIcon}>💎</Text>
            <View style={styles.vipInfo}>
              <Text style={styles.vipTitle}>星尘VIP会员</Text>
              <Text style={styles.vipDesc}>成为星尘VIP享受专属特权</Text>
            </View>
          </View>
          <View style={styles.vipCardBtn}>
            <Text style={styles.vipCardBtnText}>会员特权</Text>
            <Ionicons name="chevron-forward" size={14} color="#1a1a1a" />
          </View>
        </Pressable>

        {/* 个人资料卡片 - 命理信息 */}
        <View style={styles.profileCard}>
          <View style={styles.profileHeader}>
            <View style={styles.profileHeaderLeft}>
              <Ionicons name="person-circle-outline" size={22} color={THEME_COLOR} />
              <Text style={styles.profileTitle}>命理资料</Text>
            </View>
            <Pressable style={styles.profileEditBtn} onPress={handleOpenProfileEdit}>
              <Ionicons name="create-outline" size={16} color={THEME_COLOR} />
              <Text style={styles.profileEditText}>编辑</Text>
            </Pressable>
          </View>

          <View style={styles.profileContent}>
            <View style={styles.profileRow}>
              <Text style={styles.profileLabel}>性别</Text>
              <Text style={styles.profileValue}>{getGenderText(profile.gender)}</Text>
            </View>
            <View style={styles.profileDivider} />

            <View style={styles.profileRow}>
              <Text style={styles.profileLabel}>出生日期</Text>
              <Text style={styles.profileValue}>{getBirthDateText()}</Text>
            </View>
            <View style={styles.profileDivider} />

            <View style={styles.profileRow}>
              <Text style={styles.profileLabel}>出生时辰</Text>
              <Text style={styles.profileValue}>{getBirthHourText()}</Text>
            </View>
            <View style={styles.profileDivider} />

            <View style={styles.profileRow}>
              <Text style={styles.profileLabel}>出生地点</Text>
              <Text style={styles.profileValue}>
                {profile.longitude && profile.latitude
                  ? `经${profile.longitude}° 纬${profile.latitude}°`
                  : '未设置'}
              </Text>
            </View>
            <View style={styles.profileDivider} />

            <View style={styles.profileRow}>
              <Text style={styles.profileLabel}>服务提供者</Text>
              <Text style={[styles.profileValue, profile.isProvider && styles.profileValueActive]}>
                {profile.isProvider ? '已认证' : '未认证'}
              </Text>
            </View>
          </View>

          <View style={styles.profileTip}>
            <Ionicons name="information-circle-outline" size={14} color="#999" />
            <Text style={styles.profileTipText}>
              填写准确的出生信息可获得更精准的命理分析
            </Text>
          </View>
        </View>

        {/* 快捷操作 */}
        <View style={styles.quickActions}>
          <Pressable style={styles.actionCard} onPress={() => router.push('/wallet/transfer')}>
            <View style={[styles.actionIcon, styles.actionIconTransfer]}>
              <Ionicons name="send" size={20} color="#FFF" />
            </View>
            <Text style={styles.actionTitle}>转账</Text>
          </Pressable>

          <Pressable style={styles.actionCard} onPress={() => setReceiveModalVisible(true)}>
            <View style={[styles.actionIcon, styles.actionIconReceive]}>
              <Ionicons name="qr-code" size={20} color="#FFF" />
            </View>
            <Text style={styles.actionTitle}>收款</Text>
          </Pressable>

          <Pressable style={styles.actionCard} onPress={() => router.push('/wallet/buy-dust')}>
            <View style={[styles.actionIcon, styles.actionIconBuy]}>
              <Ionicons name="cart" size={20} color="#FFF" />
            </View>
            <Text style={styles.actionTitle}>购买DUST</Text>
          </Pressable>

          <Pressable style={styles.actionCard} onPress={() => router.push('/bridge' as any)}>
            <View style={[styles.actionIcon, styles.actionIconExchange]}>
              <Ionicons name="swap-horizontal" size={20} color="#FFF" />
            </View>
            <Text style={styles.actionTitle}>兑换DUST</Text>
          </Pressable>
        </View>

        {/* 菜单列表 */}
        <View style={styles.menuList}>
          {menuItems.map((item, index) => (
            <View key={index}>
              <Pressable style={styles.menuItem} onPress={item.onPress}>
                <View style={styles.menuLeft}>
                  <View style={styles.menuIcon}>
                    <Ionicons name={item.icon} size={20} color={THEME_COLOR} />
                  </View>
                  <Text style={styles.menuTitle}>{item.title}</Text>
                </View>
                <View style={styles.menuRight}>
                  {item.title === '语言' && (
                    <Text style={styles.languageText}>{language}</Text>
                  )}
                  {item.badge && item.badge > 0 && item.title !== '语言' && (
                    <View style={styles.menuBadge}>
                      <Text style={styles.menuBadgeText}>{item.badge}</Text>
                    </View>
                  )}
                  <Ionicons name="chevron-forward" size={16} color="#bfbfbf" />
                </View>
              </Pressable>
              {(index === 2 || index === 6) && <View style={styles.menuDivider} />}
            </View>
          ))}
        </View>

        {/* 操作按钮 */}
        <View style={styles.actions}>
          <Pressable style={styles.actionButton} onPress={handleLock}>
            <Ionicons name="lock-closed-outline" size={20} color="#666" />
            <Text style={styles.actionButtonText}>锁定钱包</Text>
          </Pressable>

          <Pressable style={[styles.actionButton, styles.dangerButton]} onPress={handleDelete}>
            <Ionicons name="trash-outline" size={20} color="#E74C3C" />
            <Text style={[styles.actionButtonText, styles.dangerButtonText]}>删除钱包</Text>
          </Pressable>
        </View>

        {/* 水印 */}
        <View style={styles.watermark}>
          <Text style={styles.watermarkText}>https://www.dustapps.net</Text>
        </View>
      </ScrollView>

      {/* 编辑昵称弹窗 */}
      <Modal visible={editModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>修改昵称</Text>
            <TextInput
              style={styles.modalInput}
              value={newNickname}
              onChangeText={setNewNickname}
              placeholder="请输入昵称"
              maxLength={64}
            />
            <View style={styles.modalTip}>
              <Text style={styles.modalTipText}>💡 提示：修改昵称需要发起链上交易并签名确认。</Text>
            </View>
            <View style={styles.modalButtons}>
              <Pressable style={styles.modalCancelBtn} onPress={() => setEditModalVisible(false)}>
                <Text style={styles.modalCancelText}>取消</Text>
              </Pressable>
              <Pressable style={styles.modalConfirmBtn} onPress={handleSaveNickname}>
                <Text style={styles.modalConfirmText}>保存</Text>
              </Pressable>
            </View>
          </View>
        </View>
      </Modal>

      {/* 收款二维码弹窗 */}
      <Modal visible={receiveModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalHeader}>
              <Ionicons name="qr-code" size={20} color={THEME_COLOR} />
              <Text style={styles.modalTitle}>收款二维码</Text>
            </View>

            {/* 二维码 */}
            <View style={styles.qrContainer}>
              {address ? (
                <QRCode value={address} size={180} color={THEME_COLOR} backgroundColor="#FFF" />
              ) : (
                <View style={styles.qrPlaceholder}>
                  <Ionicons name="qr-code-outline" size={120} color={THEME_COLOR} />
                  <Text style={styles.qrPlaceholderText}>无钱包地址</Text>
                </View>
              )}
            </View>

            <Text style={styles.addressLabel}>我的钱包地址</Text>
            <View style={styles.addressDisplay}>
              <Text style={styles.addressDisplayText}>{address}</Text>
            </View>

            <Pressable style={styles.copyAddressBtn} onPress={handleCopyAddress}>
              <Ionicons name="copy-outline" size={18} color="#FFF" />
              <Text style={styles.copyAddressBtnText}>复制地址</Text>
            </Pressable>

            <View style={styles.modalTip}>
              <Text style={styles.modalTipText}>💡 提示：请将此二维码或地址发送给付款方，对方扫码或输入地址即可向您转账。</Text>
            </View>

            <Pressable style={styles.closeModalBtn} onPress={() => setReceiveModalVisible(false)}>
              <Text style={styles.closeModalBtnText}>关闭</Text>
            </Pressable>
          </View>
        </View>
      </Modal>

      {/* 命理资料编辑弹窗 */}
      <Modal visible={profileModalVisible} transparent animationType="slide">
        <View style={styles.modalOverlay}>
          <View style={[styles.modalContent, styles.profileModalContent]}>
            <View style={styles.modalHeader}>
              <Ionicons name="person-circle-outline" size={20} color={THEME_COLOR} />
              <Text style={styles.modalTitle}>编辑命理资料</Text>
              <Pressable style={styles.modalCloseIcon} onPress={() => setProfileModalVisible(false)}>
                <Ionicons name="close" size={24} color="#999" />
              </Pressable>
            </View>

            <ScrollView style={styles.profileFormScroll} showsVerticalScrollIndicator={false}>
              {/* 性别 */}
              <View style={styles.formGroup}>
                <Text style={styles.formLabel}>性别</Text>
                <View style={styles.genderOptions}>
                  {(['male', 'female', 'other'] as const).map((g) => (
                    <Pressable
                      key={g}
                      style={[styles.genderOption, profile.gender === g && styles.genderOptionActive]}
                      onPress={() => setProfile({ ...profile, gender: g })}
                    >
                      <Text style={[styles.genderOptionText, profile.gender === g && styles.genderOptionTextActive]}>
                        {g === 'male' ? '男' : g === 'female' ? '女' : '其他'}
                      </Text>
                    </Pressable>
                  ))}
                </View>
              </View>

              {/* 出生日期 */}
              <View style={styles.formGroup}>
                <Text style={styles.formLabel}>出生日期</Text>
                <View style={styles.dateInputRow}>
                  <TextInput
                    style={[styles.dateInput, styles.dateInputYear]}
                    placeholder="年"
                    placeholderTextColor="#999"
                    keyboardType="number-pad"
                    maxLength={4}
                    value={profile.birthYear}
                    onChangeText={(v) => setProfile({ ...profile, birthYear: v })}
                  />
                  <TextInput
                    style={styles.dateInput}
                    placeholder="月"
                    placeholderTextColor="#999"
                    keyboardType="number-pad"
                    maxLength={2}
                    value={profile.birthMonth}
                    onChangeText={(v) => setProfile({ ...profile, birthMonth: v })}
                  />
                  <TextInput
                    style={styles.dateInput}
                    placeholder="日"
                    placeholderTextColor="#999"
                    keyboardType="number-pad"
                    maxLength={2}
                    value={profile.birthDay}
                    onChangeText={(v) => setProfile({ ...profile, birthDay: v })}
                  />
                </View>
              </View>

              {/* 出生时辰 */}
              <View style={styles.formGroup}>
                <Text style={styles.formLabel}>出生时辰（0-23时）</Text>
                <TextInput
                  style={styles.formInput}
                  placeholder="请输入出生时辰，如 14"
                  placeholderTextColor="#999"
                  keyboardType="number-pad"
                  maxLength={2}
                  value={profile.birthHour}
                  onChangeText={(v) => setProfile({ ...profile, birthHour: v })}
                />
              </View>

              {/* 出生地点 */}
              <View style={styles.formGroup}>
                <Text style={styles.formLabel}>出生地点（经纬度）</Text>
                <View style={styles.locationInputRow}>
                  <TextInput
                    style={[styles.formInput, styles.locationInput]}
                    placeholder="经度"
                    placeholderTextColor="#999"
                    keyboardType="decimal-pad"
                    value={profile.longitude}
                    onChangeText={(v) => setProfile({ ...profile, longitude: v })}
                  />
                  <TextInput
                    style={[styles.formInput, styles.locationInput]}
                    placeholder="纬度"
                    placeholderTextColor="#999"
                    keyboardType="decimal-pad"
                    value={profile.latitude}
                    onChangeText={(v) => setProfile({ ...profile, latitude: v })}
                  />
                </View>
              </View>

              {/* 服务提供者 */}
              <View style={styles.formGroup}>
                <View style={styles.switchRow}>
                  <Text style={styles.formLabel}>申请成为服务提供者</Text>
                  <Pressable
                    style={[styles.switchBtn, profile.isProvider && styles.switchBtnActive]}
                    onPress={() => setProfile({ ...profile, isProvider: !profile.isProvider })}
                  >
                    <View style={[styles.switchThumb, profile.isProvider && styles.switchThumbActive]} />
                  </Pressable>
                </View>
              </View>
            </ScrollView>

            <View style={styles.modalTip}>
              <Text style={styles.modalTipText}>💡 提示：准确的出生信息可获得更精准的命理分析。修改需要链上签名确认。</Text>
            </View>

            <View style={styles.modalButtons}>
              <Pressable style={styles.modalCancelBtn} onPress={() => setProfileModalVisible(false)}>
                <Text style={styles.modalCancelText}>取消</Text>
              </Pressable>
              <Pressable style={styles.modalConfirmBtn} onPress={handleSaveProfile}>
                <Text style={styles.modalConfirmText}>保存</Text>
              </Pressable>
            </View>
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
  scrollView: {
    flex: 1,
  },
  scrollContent: {
    paddingBottom: 100,
  },
  // 加载状态
  loadingSection: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 16,
    fontSize: 15,
    color: '#999',
  },
  // 欢迎页
  welcomeSection: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 24,
    paddingTop: 100,
  },
  welcomeSectionCentered: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 24,
    paddingBottom: 100,
  },
  iconCircle: {
    width: 100,
    height: 100,
    borderRadius: 50,
    backgroundColor: '#FFF',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 24,
  },
  welcomeTitle: {
    fontSize: 22,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  welcomeSubtitle: {
    fontSize: 15,
    color: '#999',
    marginBottom: 32,
  },
  buttonGroup: {
    paddingHorizontal: 24,
    gap: 12,
  },
  buttonGroupInline: {
    width: '100%',
    gap: 12,
  },
  primaryButton: {
    backgroundColor: THEME_COLOR,
    paddingVertical: 16,
    borderRadius: 8,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
  },
  primaryButtonText: {
    fontSize: 17,
    fontWeight: '600',
    color: '#FFF',
  },
  secondaryButton: {
    backgroundColor: '#FFF',
    paddingVertical: 16,
    borderRadius: 8,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  secondaryButtonText: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  // 顶部头部
  topHeader: {
    backgroundColor: '#1a1a1a',
    paddingTop: 50,
    paddingBottom: 24,
    paddingHorizontal: 20,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 16,
  },
  headerAvatar: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: THEME_COLOR,
    borderWidth: 2,
    borderColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    fontSize: 24,
    color: '#1a1a1a',
    fontWeight: 'bold',
  },
  headerUserInfo: {
    flex: 1,
  },
  headerUserRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 6,
    flexWrap: 'wrap',
  },
  headerNickname: {
    fontSize: 18,
    fontWeight: '600',
    color: '#FFF',
  },
  headerTag: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
    backgroundColor: 'rgba(255,255,255,0.1)',
    borderWidth: 1,
    borderColor: 'rgba(255,255,255,0.2)',
  },
  headerTagText: {
    fontSize: 11,
    color: 'rgba(255,255,255,0.8)',
  },
  headerTagVip: {
    backgroundColor: THEME_COLOR,
    borderWidth: 0,
  },
  headerTagVipText: {
    fontSize: 11,
    color: '#1a1a1a',
    fontWeight: '600',
  },
  headerAddress: {
    fontSize: 13,
    color: 'rgba(255,255,255,0.5)',
    fontFamily: 'monospace',
  },
  notificationBtn: {
    position: 'relative',
  },
  notificationBadge: {
    position: 'absolute',
    top: -5,
    right: -5,
    backgroundColor: '#E74C3C',
    width: 16,
    height: 16,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  notificationBadgeText: {
    fontSize: 10,
    color: '#FFF',
    fontWeight: 'bold',
  },
  // VIP卡片
  vipCard: {
    marginTop: -12,
    marginHorizontal: 16,
    marginBottom: 16,
    padding: 16,
    backgroundColor: THEME_COLOR,
    borderRadius: 12,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    shadowColor: THEME_COLOR,
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.3,
    shadowRadius: 12,
    elevation: 4,
  },
  vipCardLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  vipIcon: {
    fontSize: 24,
  },
  vipInfo: {
    gap: 2,
  },
  vipTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFF',
  },
  vipDesc: {
    fontSize: 12,
    color: 'rgba(255,255,255,0.8)',
  },
  vipCardBtn: {
    backgroundColor: 'rgba(255,255,255,0.9)',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  vipCardBtnText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1a1a1a',
  },
  // 快捷操作
  quickActions: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    paddingHorizontal: 16,
    gap: 12,
    marginBottom: 16,
  },
  actionCard: {
    width: '47%',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 12,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
  },
  actionIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  actionIconTransfer: {
    backgroundColor: THEME_COLOR,
  },
  actionIconReceive: {
    backgroundColor: '#52c41a',
  },
  actionIconBuy: {
    backgroundColor: '#faad14',
  },
  actionIconExchange: {
    backgroundColor: '#13c2c2',
  },
  actionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#262626',
  },
  // 菜单列表
  menuList: {
    backgroundColor: '#FFF',
    borderRadius: 16,
    marginHorizontal: 16,
    overflow: 'hidden',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  menuLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 16,
  },
  menuIcon: {
    width: 24,
    alignItems: 'center',
  },
  menuTitle: {
    fontSize: 16,
    color: '#262626',
    fontWeight: '500',
  },
  menuRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  languageText: {
    fontSize: 14,
    color: '#8c8c8c',
  },
  menuBadge: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 10,
  },
  menuBadgeText: {
    fontSize: 12,
    color: '#FFF',
    fontWeight: 'bold',
  },
  menuDivider: {
    height: 8,
    backgroundColor: '#f5f5f5',
  },
  // 操作按钮
  actions: {
    marginHorizontal: 16,
    marginTop: 16,
    gap: 12,
  },
  actionButton: {
    backgroundColor: '#FFF',
    paddingVertical: 14,
    borderRadius: 8,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
  },
  actionButtonText: {
    fontSize: 15,
    color: '#666',
    fontWeight: '500',
  },
  dangerButton: {
    borderColor: '#FFEBEE',
    backgroundColor: '#FFF5F5',
  },
  dangerButtonText: {
    color: '#E74C3C',
  },
  // 水印
  watermark: {
    alignItems: 'center',
    paddingVertical: 20,
    marginTop: 20,
  },
  watermarkText: {
    fontSize: 12,
    color: '#8c8c8c',
  },
  // 弹窗
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
    padding: 20,
    width: '100%',
    maxWidth: 380,
  },
  modalHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 20,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  modalInput: {
    height: 48,
    borderWidth: 2,
    borderColor: '#e0e0e0',
    borderRadius: 8,
    paddingHorizontal: 16,
    fontSize: 15,
    marginBottom: 12,
  },
  modalTip: {
    backgroundColor: 'rgba(178,149,93,0.08)',
    padding: 12,
    borderRadius: 8,
    borderLeftWidth: 4,
    borderLeftColor: THEME_COLOR,
    marginBottom: 16,
  },
  modalTipText: {
    fontSize: 12,
    color: '#8c8c8c',
    lineHeight: 18,
  },
  modalButtons: {
    flexDirection: 'row',
    gap: 12,
  },
  modalCancelBtn: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#e0e0e0',
    alignItems: 'center',
  },
  modalCancelText: {
    fontSize: 15,
    color: '#666',
  },
  modalConfirmBtn: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    backgroundColor: THEME_COLOR,
    alignItems: 'center',
  },
  modalConfirmText: {
    fontSize: 15,
    color: '#FFF',
    fontWeight: '600',
  },
  // 收款弹窗
  qrContainer: {
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
    backgroundColor: '#FFF',
    borderRadius: 12,
    marginBottom: 20,
    shadowColor: THEME_COLOR,
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.15,
    shadowRadius: 12,
    elevation: 4,
  },
  qrPlaceholder: {
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  qrPlaceholderText: {
    marginTop: 8,
    fontSize: 14,
    color: '#999',
  },
  addressLabel: {
    fontSize: 12,
    color: '#8c8c8c',
    textAlign: 'center',
    marginBottom: 8,
  },
  addressDisplay: {
    backgroundColor: 'rgba(178,149,93,0.05)',
    padding: 12,
    borderRadius: 8,
    borderWidth: 2,
    borderColor: 'rgba(178,149,93,0.1)',
    marginBottom: 16,
  },
  addressDisplayText: {
    fontSize: 13,
    fontFamily: 'monospace',
    color: '#333',
    textAlign: 'center',
  },
  copyAddressBtn: {
    backgroundColor: THEME_COLOR,
    paddingVertical: 14,
    borderRadius: 24,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    marginBottom: 16,
    shadowColor: THEME_COLOR,
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.3,
    shadowRadius: 12,
    elevation: 4,
  },
  copyAddressBtnText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFF',
  },
  closeModalBtn: {
    paddingVertical: 12,
    alignItems: 'center',
  },
  closeModalBtnText: {
    fontSize: 15,
    color: '#666',
  },
  // 个人资料卡片
  profileCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    marginHorizontal: 16,
    marginBottom: 16,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
  },
  profileHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  profileHeaderLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  profileTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  profileEditBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    paddingHorizontal: 12,
    paddingVertical: 6,
    backgroundColor: THEME_COLOR + '15',
    borderRadius: 16,
  },
  profileEditText: {
    fontSize: 13,
    color: THEME_COLOR,
  },
  profileContent: {},
  profileRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
  },
  profileLabel: {
    fontSize: 14,
    color: '#666',
  },
  profileValue: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  profileValueActive: {
    color: '#52c41a',
  },
  profileDivider: {
    height: 1,
    backgroundColor: '#f5f5f5',
  },
  profileTip: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#f5f5f5',
  },
  profileTipText: {
    flex: 1,
    fontSize: 12,
    color: '#999',
  },
  // 命理资料弹窗
  profileModalContent: {
    maxHeight: '80%',
  },
  modalCloseIcon: {
    position: 'absolute',
    right: 0,
    top: 0,
    padding: 4,
  },
  profileFormScroll: {
    maxHeight: 350,
  },
  formGroup: {
    marginBottom: 20,
  },
  formLabel: {
    fontSize: 14,
    color: '#666',
    marginBottom: 8,
  },
  formInput: {
    height: 44,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 15,
    color: '#333',
    backgroundColor: '#FAFAFA',
  },
  genderOptions: {
    flexDirection: 'row',
    gap: 12,
  },
  genderOption: {
    flex: 1,
    height: 44,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#FAFAFA',
  },
  genderOptionActive: {
    borderColor: THEME_COLOR,
    backgroundColor: THEME_COLOR + '15',
  },
  genderOptionText: {
    fontSize: 15,
    color: '#666',
  },
  genderOptionTextActive: {
    color: THEME_COLOR,
    fontWeight: '600',
  },
  dateInputRow: {
    flexDirection: 'row',
    gap: 8,
  },
  dateInput: {
    flex: 1,
    height: 44,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 15,
    color: '#333',
    backgroundColor: '#FAFAFA',
    textAlign: 'center',
  },
  dateInputYear: {
    flex: 2,
  },
  locationInputRow: {
    flexDirection: 'row',
    gap: 8,
  },
  locationInput: {
    flex: 1,
  },
  switchRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  switchBtn: {
    width: 50,
    height: 28,
    borderRadius: 14,
    backgroundColor: '#E8E8E8',
    padding: 2,
  },
  switchBtnActive: {
    backgroundColor: THEME_COLOR,
  },
  switchThumb: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: '#FFF',
  },
  switchThumbActive: {
    transform: [{ translateX: 22 }],
  },
});
