/**
 * 首购页面
 * 固定 10 USD，选择做市商，创建首购订单
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
import { useTradingStore } from '@/stores/trading.store';
import { TradingService } from '@/services/trading.service';
import { MakerCard, PaymentForm, PaymentData } from '@/features/trading/components';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import { Card, Button } from '@/components/common';
import { useAsync } from '@/hooks';
import { isWebEnvironment, isSignerUnlocked } from '@/lib/signer';

const FIRST_PURCHASE_USD = 10;

export default function FirstPurchasePage() {
  const router = useRouter();
  const {
    makers,
    selectedMaker,
    dustPrice,
    loadingOrder,
    fetchMakers,
    fetchDustPrice,
    selectMaker,
    createFirstPurchase,
  } = useTradingStore();

  const { execute, isLoading } = useAsync();
  const [estimatedDust, setEstimatedDust] = useState<string>('0');
  const [showPaymentForm, setShowPaymentForm] = useState(false);
  const [paymentData, setPaymentData] = useState<PaymentData | null>(null);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');
  const [pendingTx, setPendingTx] = useState<{
    makerId: number;
    paymentCommit: string;
    contactCommit: string;
  } | null>(null);

  useEffect(() => {
    execute(async () => {
      await Promise.all([fetchMakers(), fetchDustPrice()]);
    });
  }, []);

  useEffect(() => {
    if (dustPrice && selectedMaker) {
      const amount = TradingService.calculateDustAmount(
        FIRST_PURCHASE_USD,
        dustPrice,
        selectedMaker.sellPremiumBps
      );
      setEstimatedDust(TradingService.formatDustAmount(amount));
    }
  }, [dustPrice, selectedMaker]);

  const handleCreateOrder = async () => {
    if (!selectedMaker) {
      Alert.alert('提示', '请选择做市商');
      return;
    }

    // 显示支付信息表单
    setShowPaymentForm(true);
  };

  const handlePaymentSubmit = async (data: PaymentData) => {
    if (!selectedMaker) return;

    try {
      setPaymentData(data);
      setShowPaymentForm(false);

      // 生成支付承诺哈希
      const paymentCommit = TradingService.generatePaymentCommit(
        data.realName,
        data.idCard,
        data.phone
      );
      const contactCommit = TradingService.generateContactCommit(
        data.wechatId,
        data.phone
      );

      // 检查是否需要解锁钱包（移动端）
      if (!isWebEnvironment() && !isSignerUnlocked()) {
        // 保存待处理的交易
        setPendingTx({
          makerId: selectedMaker.id,
          paymentCommit,
          contactCommit,
        });
        // 显示解锁对话框
        setShowUnlockDialog(true);
        return;
      }

      // 直接创建订单
      await executeCreateOrder(selectedMaker.id, paymentCommit, contactCommit);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '创建订单失败，请重试';
      Alert.alert('错误', errorMessage);
    }
  };

  const handleWalletUnlocked = async () => {
    setShowUnlockDialog(false);

    if (!pendingTx) return;

    await executeCreateOrder(
      pendingTx.makerId,
      pendingTx.paymentCommit,
      pendingTx.contactCommit
    );

    setPendingTx(null);
  };

  const executeCreateOrder = async (
    makerId: number,
    paymentCommit: string,
    contactCommit: string
  ) => {
    try {
      setShowTxStatus(true);
      setTxStatus('准备中...');

      const orderId = await createFirstPurchase(
        makerId,
        paymentCommit,
        contactCommit,
        (status) => setTxStatus(status)
      );

      setShowTxStatus(false);

      Alert.alert('成功', '首购订单创建成功', [
        {
          text: '查看订单',
          onPress: () => router.push(`/wallet/buy-dust/${orderId}`),
        },
      ]);
    } catch (error) {
      setShowTxStatus(false);
      const errorMessage = error instanceof Error ? error.message : '创建订单失败，请重试';
      Alert.alert('错误', errorMessage);
    }
  };

  // 如果显示支付表单
  if (showPaymentForm) {
    return (
      <PaymentForm
        onSubmit={handlePaymentSubmit}
        onCancel={() => setShowPaymentForm(false)}
        initialData={paymentData || undefined}
      />
    );
  }

  return (
    <View style={styles.wrapper}>
      {/* 页面头部 */}
      <PageHeader title="首购" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 购买金额 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>购买金额</Text>
          <Card style={styles.amountCard}>
            <Text style={styles.amountValue}>{FIRST_PURCHASE_USD}.00 USD</Text>
            <Text style={styles.amountLabel}>(固定)</Text>
          </Card>
        </View>

        {/* 预计获得 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>预计获得</Text>
          <Card style={styles.estimateCard}>
            <Text style={styles.estimateValue}>≈ {estimatedDust} DUST</Text>
            <Text style={styles.estimateLabel}>(含首购优惠)</Text>
          </Card>
        </View>

        {/* 选择做市商 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>选择做市商</Text>
          {makers.map((maker) => (
            <MakerCard
              key={maker.id}
              maker={maker}
              selected={selectedMaker?.id === maker.id}
              onPress={() => selectMaker(maker.id)}
            />
          ))}
        </View>

        {/* 首购说明 */}
        <View style={styles.section}>
          <Card style={styles.infoCard}>
            <Text style={styles.infoTitle}>💡 首购说明</Text>
            <Text style={styles.infoText}>• 每个账户仅限一次首购</Text>
            <Text style={styles.infoText}>• 金额固定为 10 USD</Text>
            <Text style={styles.infoText}>• 完成首购后可进行普通交易</Text>
          </Card>
        </View>

        {/* 创建订单按钮 */}
        <View style={styles.section}>
          <Button
            title="创建首购订单"
            onPress={handleCreateOrder}
            loading={loadingOrder || isLoading}
            disabled={!selectedMaker || loadingOrder || isLoading}
          />
        </View>
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />

      {/* 解锁钱包对话框 */}
      <UnlockWalletDialog
        visible={showUnlockDialog}
        onUnlock={handleWalletUnlocked}
        onCancel={() => {
          setShowUnlockDialog(false);
          setPendingTx(null);
        }}
      />

      {/* 交易状态对话框 */}
      <TransactionStatusDialog
        visible={showTxStatus}
        status={txStatus}
        title="创建订单中"
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
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  amountCard: {
    alignItems: 'center',
  },
  amountValue: {
    fontSize: 32,
    fontWeight: '700',
    color: '#B2955D',
  },
  amountLabel: {
    fontSize: 14,
    color: '#666666',
    marginTop: 4,
  },
  estimateCard: {
    alignItems: 'center',
  },
  estimateValue: {
    fontSize: 24,
    fontWeight: '600',
    color: '#000000',
  },
  estimateLabel: {
    fontSize: 14,
    color: '#666666',
    marginTop: 4,
  },
  infoCard: {
    backgroundColor: '#FFF9F0',
  },
  infoTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  infoText: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 6,
  },
});
