/**
 * 创建普通订单页面
 * 输入金额 (20-200 USD)，选择做市商，创建订单
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
import { MakerCard, AmountInput, PaymentForm, PaymentData } from '@/features/trading/components';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import { Card, Button } from '@/components/common';
import { useAsync } from '@/hooks';
import { isWebEnvironment, isSignerUnlocked } from '@/lib/signer';

const MIN_AMOUNT = 20;
const MAX_AMOUNT = 200;

export default function CreateOrderPage() {
  const router = useRouter();
  const {
    makers,
    selectedMaker,
    dustPrice,
    loadingOrder,
    fetchMakers,
    fetchDustPrice,
    selectMaker,
    createOrder,
  } = useTradingStore();

  const { execute, isLoading } = useAsync();
  const [amount, setAmount] = useState<string>('50');
  const [estimatedDust, setEstimatedDust] = useState<string>('0');
  const [actualPrice, setActualPrice] = useState<number>(0);
  const [showPaymentForm, setShowPaymentForm] = useState(false);
  const [paymentData, setPaymentData] = useState<PaymentData | null>(null);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');
  const [pendingTx, setPendingTx] = useState<{
    makerId: number;
    dustAmount: bigint;
    paymentCommit: string;
    contactCommit: string;
  } | null>(null);

  useEffect(() => {
    execute(async () => {
      await Promise.all([fetchMakers(), fetchDustPrice()]);
    });
  }, []);

  useEffect(() => {
    if (dustPrice && selectedMaker && amount) {
      const usdAmount = parseFloat(amount);
      if (!isNaN(usdAmount)) {
        const dustAmount = TradingService.calculateDustAmount(
          usdAmount,
          dustPrice,
          selectedMaker.sellPremiumBps
        );
        setEstimatedDust(TradingService.formatDustAmount(dustAmount));

        const price = dustPrice * (1 + selectedMaker.sellPremiumBps / 10000);
        setActualPrice(price);
      }
    }
  }, [dustPrice, selectedMaker, amount]);

  const handleCreateOrder = async () => {
    if (!selectedMaker) {
      Alert.alert('提示', '请选择做市商');
      return;
    }

    const usdAmount = parseFloat(amount);
    if (isNaN(usdAmount) || usdAmount < MIN_AMOUNT || usdAmount > MAX_AMOUNT) {
      Alert.alert('提示', `请输入 ${MIN_AMOUNT}-${MAX_AMOUNT} USD 之间的金额`);
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

      const usdAmount = parseFloat(amount);

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

      const dustAmount = TradingService.calculateDustAmount(
        usdAmount,
        dustPrice,
        selectedMaker.sellPremiumBps
      );

      // 检查是否需要解锁钱包（移动端）
      if (!isWebEnvironment() && !isSignerUnlocked()) {
        // 保存待处理的交易
        setPendingTx({
          makerId: selectedMaker.id,
          dustAmount,
          paymentCommit,
          contactCommit,
        });
        // 显示解锁对话框
        setShowUnlockDialog(true);
        return;
      }

      // 直接创建订单
      await executeCreateOrder(selectedMaker.id, dustAmount, paymentCommit, contactCommit);
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
      pendingTx.dustAmount,
      pendingTx.paymentCommit,
      pendingTx.contactCommit
    );

    setPendingTx(null);
  };

  const executeCreateOrder = async (
    makerId: number,
    dustAmount: bigint,
    paymentCommit: string,
    contactCommit: string
  ) => {
    try {
      setShowTxStatus(true);
      setTxStatus('准备中...');

      const orderId = await createOrder(
        makerId,
        dustAmount,
        paymentCommit,
        contactCommit,
        (status) => setTxStatus(status)
      );

      setShowTxStatus(false);

      Alert.alert('成功', '订单创建成功', [
        {
          text: '查看订单',
          onPress: () => router.push(`/wallet/buy-dust/${orderId}` as any),
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
      <PageHeader title="创建订单" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 金额输入 */}
        <View style={styles.section}>
          <AmountInput
            value={amount}
            onChangeText={setAmount}
            min={MIN_AMOUNT}
            max={MAX_AMOUNT}
            label="购买金额"
            unit="USD"
            quickAmounts={[20, 50, 100, 200]}
          />
        </View>

        {/* 预计获得 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>预计获得</Text>
          <Card style={styles.estimateCard}>
            <Text style={styles.estimateValue}>≈ {estimatedDust} DUST</Text>
            <View style={styles.estimateDetails}>
              <Text style={styles.estimateDetail}>
                单价: {actualPrice.toFixed(6)} USDT/DUST
              </Text>
              {selectedMaker && (
                <Text style={styles.estimateDetail}>
                  溢价: {selectedMaker.sellPremiumBps >= 0 ? '+' : ''}
                  {(selectedMaker.sellPremiumBps / 100).toFixed(1)}%
                </Text>
              )}
            </View>
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

        {/* 创建订单按钮 */}
        <View style={styles.section}>
          <Button
            title="创建订单"
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
  estimateCard: {
    padding: 20,
  },
  estimateValue: {
    fontSize: 28,
    fontWeight: '600',
    color: '#000000',
    textAlign: 'center',
    marginBottom: 12,
  },
  estimateDetails: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  estimateDetail: {
    fontSize: 14,
    color: '#666666',
  },
});
