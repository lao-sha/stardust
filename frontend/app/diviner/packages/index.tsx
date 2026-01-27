/**
 * 套餐管理列表页面
 */

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  ActivityIndicator,
  RefreshControl,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { PackageCard, ServicePackage, DivinationType, ServiceType } from '@/features/diviner';
import { divinationMarketService } from '@/services/divination-market.service';
import { useWalletStore } from '@/stores/wallet.store';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

const THEME_COLOR = '#B2955D';
const MAX_PACKAGES = 10;

export default function PackagesListPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [packages, setPackages] = useState<ServicePackage[]>([]);
  const [providerId, setProviderId] = useState<number | null>(null);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingAction, setPendingAction] = useState<{ type: 'toggle' | 'delete'; id: number; isActive?: boolean } | null>(null);

  const loadData = useCallback(async () => {
    if (!address) return;

    try {
      // 获取当前用户的解卦师信息
      const provider = await divinationMarketService.getProviderByAccount(address);
      if (provider) {
        setProviderId(provider.id);
        // 获取套餐列表
        const pkgs = await divinationMarketService.getProviderPackages(provider.id);
        // 转换为前端格式
        const formattedPkgs: ServicePackage[] = pkgs.map(p => ({
          id: p.id,
          providerId: address,
          divinationType: DivinationType.Bazi, // 需要从链上数据映射
          serviceType: ServiceType.TextReading,
          name: p.name,
          description: p.description,
          price: p.price,
          duration: p.duration,
          followUpCount: 3,
          urgentAvailable: false,
          urgentSurcharge: 0,
          isActive: p.isActive,
          salesCount: 0,
        }));
        setPackages(formattedPkgs);
      }
    } catch (error) {
      console.error('Load packages error:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [address]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const onRefresh = () => {
    setRefreshing(true);
    loadData();
  };

  const handleCreate = () => {
    if (packages.length >= MAX_PACKAGES) {
      Alert.alert('提示', `套餐数量已达上限（${MAX_PACKAGES}个）`);
      return;
    }
    router.push('/diviner/packages/create' as any);
  };

  const handleEdit = (id: number) => {
    router.push(`/diviner/packages/${id}` as any);
  };

  const handleToggle = (id: number, isActive: boolean) => {
    if (!isSignerUnlocked()) {
      setPendingAction({ type: 'toggle', id, isActive });
      setShowUnlockDialog(true);
      return;
    }
    executeToggle(id, isActive);
  };

  const handleDelete = (id: number) => {
    Alert.alert('确认删除', '删除后无法恢复，确定要删除此套餐吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '删除',
        style: 'destructive',
        onPress: () => {
          if (!isSignerUnlocked()) {
            setPendingAction({ type: 'delete', id });
            setShowUnlockDialog(true);
            return;
          }
          executeDelete(id);
        },
      },
    ]);
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      if (pendingAction) {
        if (pendingAction.type === 'toggle') {
          await executeToggle(pendingAction.id, pendingAction.isActive!);
        } else {
          await executeDelete(pendingAction.id);
        }
        setPendingAction(null);
      }
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeToggle = async (id: number, isActive: boolean) => {
    setShowTxStatus(true);
    setTxStatus(isActive ? '正在激活套餐...' : '正在停用套餐...');

    try {
      if (isActive) {
        await divinationMarketService.reactivatePackage(id, (status) => setTxStatus(status));
      } else {
        await divinationMarketService.deactivatePackage(id, (status) => setTxStatus(status));
      }

      setTxStatus('操作成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        loadData();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('操作失败', error.message || '请稍后重试');
    }
  };

  const executeDelete = async (id: number) => {
    setShowTxStatus(true);
    setTxStatus('正在删除套餐...');

    try {
      await divinationMarketService.removePackage(id, (status) => setTxStatus(status));

      setTxStatus('删除成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        loadData();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('删除失败', error.message || '请稍后重试');
    }
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="套餐管理" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="套餐管理" />

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={THEME_COLOR} />}
      >
        {/* 套餐数量提示 */}
        <View style={styles.countSection}>
          <Text style={styles.countText}>
            已创建 {packages.length}/{MAX_PACKAGES} 个套餐
          </Text>
        </View>

        {/* 套餐列表 */}
        <View style={styles.section}>
          {packages.length === 0 ? (
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyIcon}>📦</Text>
              <Text style={styles.emptyText}>还没有创建套餐</Text>
              <Text style={styles.emptySubtext}>创建服务套餐，开始接单赚钱</Text>
            </View>
          ) : (
            packages.map(pkg => (
              <PackageCard
                key={pkg.id}
                package={pkg}
                editable
                onEdit={() => handleEdit(pkg.id)}
                onToggle={(isActive) => handleToggle(pkg.id, isActive)}
                onDelete={() => handleDelete(pkg.id)}
              />
            ))
          )}
        </View>
      </ScrollView>

      {/* 创建按钮 */}
      <View style={styles.footer}>
        <Pressable
          style={[styles.createBtn, packages.length >= MAX_PACKAGES && styles.createBtnDisabled]}
          onPress={handleCreate}
        >
          <Text style={styles.createBtnText}>+ 创建新套餐</Text>
        </Pressable>
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
    paddingBottom: 160,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  countSection: {
    padding: 16,
    paddingBottom: 0,
  },
  countText: {
    fontSize: 14,
    color: '#666',
  },
  section: {
    padding: 16,
  },
  emptyContainer: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 48,
    alignItems: 'center',
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#333',
    marginBottom: 8,
  },
  emptySubtext: {
    fontSize: 14,
    color: '#999',
  },
  footer: {
    position: 'absolute',
    bottom: 80,
    left: 0,
    right: 0,
    padding: 16,
    backgroundColor: '#F5F5F7',
  },
  createBtn: {
    height: 52,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  createBtnDisabled: {
    opacity: 0.5,
  },
  createBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
});
