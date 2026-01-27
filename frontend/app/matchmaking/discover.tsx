/**
 * 发现页面 - 浏览推荐的用户
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Dimensions,
  Pressable,
  Alert,
  ActivityIndicator,
  Image,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { matchmakingService, UserProfile, Gender } from '@/services/matchmaking.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';
const { width: SCREEN_WIDTH } = Dimensions.get('window');
const CARD_WIDTH = SCREEN_WIDTH - 32;

export default function DiscoverPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [profiles, setProfiles] = useState<UserProfile[]>([]);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingAction, setPendingAction] = useState<'like' | 'superlike' | 'pass' | null>(null);
  const [quota, setQuota] = useState({ likes: 0, superLikes: 0, views: 0 });

  const loadProfiles = useCallback(async () => {
    if (!address) return;

    try {
      // 获取剩余配额
      const remainingQuota = await matchmakingService.getRemainingQuota(address);
      setQuota(remainingQuota);

      // TODO: 这里应该调用推荐算法获取推荐用户
      // 暂时使用模拟数据
      setProfiles([]);
    } catch (error) {
      console.error('Load profiles error:', error);
    } finally {
      setLoading(false);
    }
  }, [address]);

  useEffect(() => {
    loadProfiles();
  }, [loadProfiles]);

  const currentProfile = profiles[currentIndex];

  const handleAction = async (action: 'like' | 'superlike' | 'pass') => {
    if (!currentProfile) return;

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

  const executeAction = async (action: 'like' | 'superlike' | 'pass') => {
    if (!currentProfile) return;

    setShowTxStatus(true);
    const actionText = action === 'like' ? '喜欢' : action === 'superlike' ? '超级喜欢' : '跳过';
    setTxStatus(`正在${actionText}...`);

    try {
      if (action === 'like') {
        await matchmakingService.like(currentProfile.owner, (status) => setTxStatus(status));
      } else if (action === 'superlike') {
        await matchmakingService.superLike(currentProfile.owner, (status) => setTxStatus(status));
      } else {
        await matchmakingService.pass(currentProfile.owner, (status) => setTxStatus(status));
      }

      setTxStatus('操作成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        // 移动到下一个用户
        if (currentIndex < profiles.length - 1) {
          setCurrentIndex(currentIndex + 1);
        } else {
          // 没有更多用户了
          setProfiles([]);
        }
      }, 1000);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('操作失败', error.message || '请稍后重试');
    }
  };

  const getGenderText = (gender: Gender) => {
    return gender === Gender.Male ? '男' : gender === Gender.Female ? '女' : '其他';
  };

  if (loading) {
    return (
      <View style={styles.container}>
        <PageHeader title="发现" showBack />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
        <BottomNavBar />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="发现" showBack />

      {/* 配额显示 */}
      <View style={styles.quotaBar}>
        <View style={styles.quotaItem}>
          <Ionicons name="heart" size={16} color="#FF6B6B" />
          <Text style={styles.quotaText}>{quota.likes}</Text>
        </View>
        <View style={styles.quotaItem}>
          <Ionicons name="star" size={16} color="#FFD700" />
          <Text style={styles.quotaText}>{quota.superLikes}</Text>
        </View>
        <View style={styles.quotaItem}>
          <Ionicons name="eye" size={16} color={THEME_COLOR} />
          <Text style={styles.quotaText}>{quota.views === 4294967295 ? '∞' : quota.views}</Text>
        </View>
      </View>

      <View style={styles.content}>
        {!currentProfile ? (
          <View style={styles.emptyContainer}>
            <Ionicons name="search-outline" size={80} color="#ccc" />
            <Text style={styles.emptyTitle}>暂无推荐</Text>
            <Text style={styles.emptyText}>
              调整您的择偶条件，或稍后再来看看
            </Text>
            <Pressable
              style={styles.refreshButton}
              onPress={loadProfiles}
            >
              <Text style={styles.refreshButtonText}>刷新</Text>
            </Pressable>
          </View>
        ) : (
          <>
            {/* 用户卡片 */}
            <View style={styles.card}>
              <View style={styles.cardImageContainer}>
                {currentProfile.photoCids && currentProfile.photoCids.length > 0 ? (
                  <Image
                    source={{ uri: `https://ipfs.io/ipfs/${currentProfile.photoCids[0]}` }}
                    style={styles.cardImage}
                  />
                ) : (
                  <View style={styles.cardImagePlaceholder}>
                    <Ionicons name="person" size={80} color="#ccc" />
                  </View>
                )}
              </View>

              <View style={styles.cardInfo}>
                <View style={styles.cardHeader}>
                  <Text style={styles.cardName}>{currentProfile.nickname}</Text>
                  <View style={styles.cardBadge}>
                    <Ionicons
                      name={currentProfile.gender === Gender.Male ? 'male' : 'female'}
                      size={16}
                      color={currentProfile.gender === Gender.Male ? '#4A90D9' : '#FF6B9D'}
                    />
                    <Text style={styles.cardAge}>
                      {new Date().getFullYear() - currentProfile.birthYear}岁
                    </Text>
                  </View>
                </View>

                <View style={styles.cardDetails}>
                  <View style={styles.cardDetailItem}>
                    <Ionicons name="location-outline" size={16} color="#666" />
                    <Text style={styles.cardDetailText}>{currentProfile.location}</Text>
                  </View>
                  <View style={styles.cardDetailItem}>
                    <Ionicons name="resize-outline" size={16} color="#666" />
                    <Text style={styles.cardDetailText}>{currentProfile.height}cm</Text>
                  </View>
                </View>

                {currentProfile.bio && (
                  <Text style={styles.cardBio} numberOfLines={3}>
                    {currentProfile.bio}
                  </Text>
                )}

                {currentProfile.baziChartId && (
                  <View style={styles.baziTag}>
                    <Ionicons name="sparkles" size={14} color={THEME_COLOR} />
                    <Text style={styles.baziTagText}>已绑定八字</Text>
                  </View>
                )}
              </View>
            </View>

            {/* 操作按钮 */}
            <View style={styles.actionButtons}>
              <Pressable
                style={[styles.actionButton, styles.passButton]}
                onPress={() => handleAction('pass')}
              >
                <Ionicons name="close" size={32} color="#999" />
              </Pressable>

              <Pressable
                style={[styles.actionButton, styles.superlikeButton]}
                onPress={() => handleAction('superlike')}
              >
                <Ionicons name="star" size={28} color="#4A90D9" />
              </Pressable>

              <Pressable
                style={[styles.actionButton, styles.likeButton]}
                onPress={() => handleAction('like')}
              >
                <Ionicons name="heart" size={32} color="#FF6B6B" />
              </Pressable>
            </View>

            {/* 进度指示 */}
            <Text style={styles.progressText}>
              {currentIndex + 1} / {profiles.length}
            </Text>
          </>
        )}
      </View>

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
  quotaBar: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#fff',
    paddingVertical: 8,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  quotaItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginHorizontal: 16,
  },
  quotaText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginLeft: 4,
  },
  content: {
    flex: 1,
    padding: 16,
    justifyContent: 'center',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 12,
    color: '#666',
  },
  emptyContainer: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#333',
    marginTop: 20,
  },
  emptyText: {
    fontSize: 14,
    color: '#666',
    marginTop: 8,
    textAlign: 'center',
  },
  refreshButton: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 20,
    marginTop: 20,
  },
  refreshButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  card: {
    backgroundColor: '#fff',
    borderRadius: 16,
    overflow: 'hidden',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 8,
    elevation: 4,
  },
  cardImageContainer: {
    width: '100%',
    height: 300,
  },
  cardImage: {
    width: '100%',
    height: '100%',
  },
  cardImagePlaceholder: {
    width: '100%',
    height: '100%',
    backgroundColor: '#f0f0f0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  cardInfo: {
    padding: 16,
  },
  cardHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  cardName: {
    fontSize: 22,
    fontWeight: 'bold',
    color: '#333',
  },
  cardBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f5f5f5',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  cardAge: {
    fontSize: 14,
    color: '#666',
    marginLeft: 4,
  },
  cardDetails: {
    flexDirection: 'row',
    marginTop: 12,
  },
  cardDetailItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginRight: 16,
  },
  cardDetailText: {
    fontSize: 14,
    color: '#666',
    marginLeft: 4,
  },
  cardBio: {
    fontSize: 14,
    color: '#666',
    marginTop: 12,
    lineHeight: 20,
  },
  baziTag: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f8f4e8',
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 12,
    marginTop: 12,
    alignSelf: 'flex-start',
  },
  baziTagText: {
    fontSize: 12,
    color: THEME_COLOR,
    marginLeft: 4,
  },
  actionButtons: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 24,
  },
  actionButton: {
    width: 60,
    height: 60,
    borderRadius: 30,
    backgroundColor: '#fff',
    justifyContent: 'center',
    alignItems: 'center',
    marginHorizontal: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 2,
  },
  passButton: {
    borderWidth: 2,
    borderColor: '#e0e0e0',
  },
  superlikeButton: {
    borderWidth: 2,
    borderColor: '#4A90D9',
  },
  likeButton: {
    borderWidth: 2,
    borderColor: '#FF6B6B',
  },
  progressText: {
    textAlign: 'center',
    color: '#999',
    fontSize: 12,
    marginTop: 16,
  },
});
