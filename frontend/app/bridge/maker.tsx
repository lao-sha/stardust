/**
 * 做市商桥接页面
 * 选择做市商进行 DUST → USDT 兑换
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { Card, Button, LoadingSpinner, EmptyState } from '@/components/common';
import {
  SwapAmountInput,
  TronAddressInput,
  BridgeMakerCard,
} from '@/features/bridge/components';
import { BridgeMaker } from '@/features/bridge/types';
import { bridgeService } from '@/services/bridge.service';
import { tradingService } from '@/services/trading.service';
import { useWallet, useAsync } from '@/hooks';
import { isWebEnvironment, isSignerUnlocked } from '@/lib/signer';

export default function MakerBridgePage() {
  const router = useRouter();
  const { address, balance, ensureUnlocked } = useWallet();
  const { execute, isLoading } = useAsync();

  const [dustAmount, setDustAmount] = useState('');
  const [tronAddress, setTronAddress] = useState('');
  const [selectedMaker, setSelectedMaker] = useState<BridgeMaker | null>(null);
  const [dustPrice, setDustPrice] = useState(0.10);
  const [makers, setMakers] = useState<BridgeMaker[]>([]);
  const [loadingMakers, setLoadingMakers] = useState(true);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');

  useEffect(() => {
    loadMakers();
    loadPrice();
  }, []);

  const loadMakers = async () => {
    try {
      setLoadingMakers(true);
      const makerList = await tradingService.getMakers();
      // 转换为 BridgeMaker 格式
      const bridgeMakers: BridgeMaker[] = makerList.map((m) => ({
        id: m.id,
        account: m.owner,
        tronAddress: m.tronAddress,
        isActive: !m.servicePaused,
        rating: m.rating,
        completedSwaps: m.usersServed,
        avgResponseTime: 600, // 默认值
        creditLevel: m.rating >= 4.8 ? 'A+' : m.rating >= 4.5 ? 'A' : 'B+',
      }));
      setMakers(bridgeMakers);
    } catch (error) {
      console.error('Load makers error:', error);
      Alert.alert('错误', '加载做市商列表失败');
    } finally {
      setLoadingMakers(false);
    }
  };

  const loadPrice = async () => {
    try {
      const price = await bridgeService.getDustPrice();
      setDustPrice(price);
    } catch (error) {
      console.error('Load price error:', error);
    }
  };

  const validateForm = (): boolean => {
    const amount = parseFloat(dustAmount);
    const balanceNum = Number(balance) / 1e12;

    if (isNaN(amount) || amount < MIN_AMOUNT) {
      Alert.alert('提示', `最小兑换金额为 ${MIN_AMOUNT} DUST`);
      return false;
    }

    if (amount > balanceNum) {
      Alert.alert('提示', 'DUST 余额不足');
      return false;
    }

    // 验证 TRON 地址
    const tronRegex = /^T[A-Za-z1-9]{33}$/;
    if (!tronRegex.test(tronAddress)) {
      Alert.alert('提示', '请输入有效的 TRON 地址');
      return false;
    }

    if (!selectedMaker) {
      Alert.alert('提示', '请选择做市商');
      return false;
    }

    if (!selectedMaker.isActive) {
      Alert.alert('提示', '该做市商当前离线，请选择其他做市商');
      return false;
    }

    return true;
  };

  const handleSwap = async () => {
    if (!validateForm()) return;

    // 确保钱包已解锁
    const unlocked = await ensureUnlocked();
    if (!unlocked) {
      setShowUnlockDialog(true);
      return;
    }

    await executeSwap();
  };

  const handleWalletUnlocked = async () => {
    setShowUnlockDialog(false);
    await executeSwap();
  };

  const executeSwap = async () => {
    if (!selectedMaker || !address) return;

    try {
      await execute(async () => {
        setShowTxStatus(true);
        setTxStatus('正在创建兑换请求...');

        const dustAmountBigInt = BigInt(Math.floor(parseFloat(dustAmount) * 1e12));

        const swapId = await bridgeService.makerSwap(
          selectedMaker.id,
          dustAmountBigInt,
          tronAddress,
          (status) => {
            setTxStatus(status);
          }
        );

        setShowTxStatus(false);

        Alert.alert(
          '成功',
          `兑换请求已创建 (ID: ${swapId})，做市商将在 30 分钟内转账`,
          [
            {
              text: '查看记录',
              onPress: () => router.push('/bridge/history' as any),
            },
            {
              text: '确定',
              style: 'cancel',
            },
          ]
        );
      });
    } catch (error) {
      setShowTxStatus(false);
      const errorMessage = error instanceof Error ? error.message : '创建兑换失败';
      Alert.alert('错误', errorMessage);
    }
  };

  const usdtEstimate = (parseFloat(dustAmount) || 0) * dustPrice;
  const activeMakersCount = makers.filter(m => m.isActive).length;

  return (
    <View style={styles.wrapper}>
      <PageHeader title="做市商桥接" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 说明卡片 */}
        <View style={styles.section}>
          <Card style={styles.infoCard}>
            <Text style={styles.infoTitle}>👥 做市商桥接</Text>
            <Text style={styles.infoText}>
              选择做市商进行兑换，通常 30 分钟内到账。
              超时未完成将自动退款。
            </Text>
          </Card>
        </View>

        {/* 金额输入 */}
        <View style={styles.section}>
          <SwapAmountInput
            value={dustAmount}
            onChangeText={setDustAmount}
            dustPrice={dustPrice}
            balance={(Number(balance) / 1e12).toFixed(4)}
            minAmount={MIN_AMOUNT}
          />
        </View>

        {/* TRON 地址输入 */}
        <View style={styles.section}>
          <TronAddressInput
            value={tronAddress}
            onChangeText={setTronAddress}
          />
        </View>

        {/* 选择做市商 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>选择做市商</Text>
            <Text style={styles.sectionSubtitle}>
              {activeMakersCount} 位做市商在线
            </Text>
          </View>

          {loadingMakers ? (
            <LoadingSpinner text="加载做市商列表..." />
          ) : makers.length === 0 ? (
            <EmptyState
              icon="people-outline"
              title="暂无可用做市商"
              description="请稍后再试"
            />
          ) : (
            makers.map((maker) => (
              <BridgeMakerCard
                key={maker.id}
                maker={maker}
                selected={selectedMaker?.id === maker.id}
                onPress={() => setSelectedMaker(maker)}
              />
            ))
          )}
        </View>

        {/* 兑换详情 */}
        {selectedMaker && (
          <View style={styles.section}>
            <Card>
              <Text style={styles.detailTitle}>兑换详情</Text>
              <View style={styles.detailRow}>
                <Text style={styles.detailLabel}>支付</Text>
                <Text style={styles.detailValue}>
                  {dustAmount || '0'} DUST
                </Text>
              </View>
              <View style={styles.detailRow}>
                <Text style={styles.detailLabel}>汇率</Text>
                <Text style={styles.detailValue}>
                  1 DUST = {dustPrice.toFixed(4)} USDT
                </Text>
              </View>
              <View style={styles.detailRow}>
                <Text style={styles.detailLabel}>做市商</Text>
                <Text style={styles.detailValue}>
                  #{selectedMaker.id} ({selectedMaker.creditLevel})
                </Text>
              </View>
              <View style={styles.detailRow}>
                <Text style={styles.detailLabel}>超时时间</Text>
                <Text style={styles.detailValue}>30 分钟</Text>
              </View>
              <View style={styles.divider} />
              <View style={styles.detailRow}>
                <Text style={styles.detailLabelBold}>预计获得</Text>
                <Text style={styles.detailValueGreen}>
                  ≈ {usdtEstimate.toFixed(2)} USDT
                </Text>
              </View>
            </Card>
          </View>
        )}

        {/* 提交按钮 */}
        <View style={styles.section}>
          <Button
            title="确认兑换"
            onPress={handleSwap}
            loading={isLoading}
            disabled={!dustAmount || !tronAddress || !selectedMaker}
          />
        </View>

        {/* 注意事项 */}
        <View style={styles.section}>
          <Text style={styles.noticeTitle}>⚠️ 注意事项</Text>
          <Text style={styles.noticeText}>• 兑换请求提交后，DUST 将被锁定</Text>
          <Text style={styles.noticeText}>• 做市商需在 30 分钟内完成转账</Text>
          <Text style={styles.noticeText}>• 超时未完成将自动退款</Text>
          <Text style={styles.noticeText}>• 如遇问题可发起举报</Text>
        </View>
      </ScrollView>

      <BottomNavBar activeTab="profile" />

      {/* 解锁钱包对话框 */}
      <UnlockWalletDialog
        visible={showUnlockDialog}
        onUnlock={handleWalletUnlocked}
        onCancel={() => setShowUnlockDialog(false)}
      />

      {/* 交易状态对话框 */}
      <TransactionStatusDialog
        visible={showTxStatus}
        status={txStatus}
        title="创建兑换中"
      />
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
    paddingBottom: 20,
  },
  section: {
    padding: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000000',
  },
  sectionSubtitle: {
    fontSize: 14,
    color: '#666666',
  },
  infoCard: {
    backgroundColor: '#FFF9F0',
    borderWidth: 1,
    borderColor: '#B2955D',
  },
  infoTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 14,
    color: '#666666',
    lineHeight: 20,
  },
  loading: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  loadingText: {
    fontSize: 14,
    color: '#666666',
    marginTop: 12,
  },
  empty: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  emptyText: {
    fontSize: 14,
    color: '#999999',
  },
  detailCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
  },
  detailTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  detailRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  detailLabel: {
    fontSize: 14,
    color: '#666666',
  },
  detailLabelBold: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
  },
  detailValue: {
    fontSize: 14,
    color: '#000000',
  },
  detailValueGreen: {
    fontSize: 16,
    fontWeight: '600',
    color: '#4CD964',
  },
  divider: {
    height: 1,
    backgroundColor: '#F0F0F0',
    marginVertical: 8,
  },
  submitButton: {
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
  },
  submitButtonDisabled: {
    backgroundColor: '#CCCCCC',
  },
  submitButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 13,
    color: '#666666',
    marginBottom: 4,
  },
});
