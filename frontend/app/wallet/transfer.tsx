/**
 * 星尘玄鉴 - 转账页面
 * 发送代币到其他地址
 * 主题色：金棕色 #B2955D
 */

import { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWalletStore } from '@/stores';
import { BottomNavBar } from '@/components/BottomNavBar';
import { Card, Button, Input, LoadingSpinner } from '@/components/common';
import { useAsync, useClipboard, useWallet } from '@/hooks';
import { getApi } from '@/lib/api';
import { signAndSend } from '@/lib/signer';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

export default function TransferPage() {
  const router = useRouter();
  const { address, balance, isUnlocked, ensureUnlocked } = useWallet();
  const { execute, isLoading } = useAsync();
  const { getFromClipboard } = useClipboard();

  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [memo, setMemo] = useState('');
  const [recipientError, setRecipientError] = useState('');
  const [amountError, setAmountError] = useState('');

  const isValidAddress = (addr: string) => {
    // Substrate 地址以 5 开头，长度 48
    return addr.startsWith('5') && addr.length === 48;
  };

  const validateForm = (): boolean => {
    let isValid = true;

    // 验证收款地址
    if (!recipient) {
      setRecipientError('请输入收款地址');
      isValid = false;
    } else if (!isValidAddress(recipient)) {
      setRecipientError('收款地址格式不正确');
      isValid = false;
    } else if (recipient === address) {
      setRecipientError('不能转账给自己');
      isValid = false;
    } else {
      setRecipientError('');
    }

    // 验证金额
    const amountNum = parseFloat(amount);
    const balanceNum = Number(balance) / 1e12;
    if (!amount || isNaN(amountNum) || amountNum <= 0) {
      setAmountError('请输入有效的转账金额');
      isValid = false;
    } else if (amountNum > balanceNum) {
      setAmountError('余额不足');
      isValid = false;
    } else {
      setAmountError('');
    }

    return isValid;
  };

  const handleTransfer = async () => {
    if (!validateForm()) return;

    // 确保钱包已解锁
    const unlocked = await ensureUnlocked();
    if (!unlocked) {
      Alert.alert('提示', '请先解锁钱包');
      return;
    }

    // 确认转账
    Alert.alert(
      '确认转账',
      `确定要向 ${recipient.slice(0, 8)}...${recipient.slice(-8)} 转账 ${amount} DUST 吗？`,
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确认',
          onPress: () => executeTransfer(),
        },
      ]
    );
  };

  const executeTransfer = async () => {
    try {
      await execute(async () => {
        const api = await getApi();
        const amountBigInt = BigInt(Math.floor(parseFloat(amount) * 1e12));

        const tx = api.tx.balances.transfer(recipient, amountBigInt.toString());

        await signAndSend(api, tx, address!, (status) => {
          console.log('Transfer status:', status);
        });

        Alert.alert('成功', '转账已提交', [
          {
            text: '查看记录',
            onPress: () => router.push('/wallet/transactions' as any),
          },
          { text: '确定', style: 'cancel' },
        ]);
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '转账失败';
      Alert.alert('转账失败', errorMessage);
    }
  };

  const handlePasteAddress = async () => {
    const text = await getFromClipboard();
    if (text && isValidAddress(text)) {
      setRecipient(text);
      setRecipientError('');
    } else {
      Alert.alert('提示', '剪贴板中没有有效的地址');
    }
  };

  const handleScanQR = () => {
    Alert.alert('提示', '二维码扫描功能即将上线');
  };

  const handleMaxAmount = () => {
    const balanceNum = (Number(balance) / 1e12).toFixed(4);
    setAmount(balanceNum);
    setAmountError('');
  };

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
    >
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>转账</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.scrollView} contentContainerStyle={styles.content} showsVerticalScrollIndicator={false}>
        {/* 余额显示 */}
        <Card style={styles.balanceCard}>
          <Text style={styles.balanceLabel}>可用余额</Text>
          <Text style={styles.balanceAmount}>
            {(Number(balance) / 1e12).toFixed(4)} DUST
          </Text>
        </Card>

        {/* 表单卡片 */}
        <Card>
          {/* 收款地址 */}
          <Input
            label="收款地址"
            placeholder="输入 Substrate 地址 (以 5 开头)"
            value={recipient}
            onChangeText={(text) => {
              setRecipient(text);
              setRecipientError('');
            }}
            error={recipientError}
            autoCapitalize="none"
            autoCorrect={false}
          />
          <View style={styles.addressActions}>
            <Pressable onPress={handlePasteAddress} style={styles.actionButton}>
              <Ionicons name="clipboard-outline" size={18} color={THEME_COLOR} />
              <Text style={styles.actionButtonText}>粘贴</Text>
            </Pressable>
            <Pressable onPress={handleScanQR} style={styles.actionButton}>
              <Ionicons name="scan-outline" size={18} color={THEME_COLOR} />
              <Text style={styles.actionButtonText}>扫码</Text>
            </Pressable>
          </View>

          {/* 转账金额 */}
          <View style={styles.amountContainer}>
            <Input
              label="转账金额"
              placeholder="0.00"
              value={amount}
              onChangeText={(text) => {
                setAmount(text);
                setAmountError('');
              }}
              error={amountError}
              keyboardType="decimal-pad"
              containerStyle={styles.amountInput}
            />
            <Pressable onPress={handleMaxAmount} style={styles.maxButton}>
              <Text style={styles.maxButtonText}>MAX</Text>
            </Pressable>
          </View>

          {/* 备注 */}
          <Input
            label="备注（可选）"
            placeholder="添加转账备注"
            value={memo}
            onChangeText={setMemo}
            maxLength={100}
          />

          {/* 手续费提示 */}
          <View style={styles.feeInfo}>
            <Ionicons name="information-circle-outline" size={16} color="#999" />
            <Text style={styles.feeText}>预估手续费: 0.001 DUST</Text>
          </View>
        </Card>

        {/* 转账按钮 */}
        <Button
          title="确认转账"
          onPress={handleTransfer}
          loading={isLoading}
          disabled={!recipient || !amount || isLoading}
          style={styles.submitButton}
        />

        {/* 安全提示 */}
        <View style={styles.tips}>
          <View style={styles.tipItem}>
            <Ionicons name="shield-checkmark-outline" size={18} color="#27AE60" />
            <Text style={styles.tipText}>请仔细核对收款地址</Text>
          </View>
          <View style={styles.tipItem}>
            <Ionicons name="alert-circle-outline" size={18} color="#F39C12" />
            <Text style={styles.tipText}>转账后无法撤销</Text>
          </View>
        </View>
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />
    </KeyboardAvoidingView>
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
    paddingTop: 50,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backButton: {
    padding: 4,
  },
  navTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  placeholder: {
    width: 32,
  },
  scrollView: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  balanceCard: {
    marginBottom: 16,
    alignItems: 'center',
  },
  balanceLabel: {
    fontSize: 14,
    color: '#999',
    marginBottom: 8,
  },
  balanceAmount: {
    fontSize: 32,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  addressActions: {
    flexDirection: 'row',
    gap: 12,
    marginTop: -8,
    marginBottom: 16,
  },
  actionButton: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    paddingVertical: 6,
    paddingHorizontal: 12,
    borderRadius: 6,
    backgroundColor: THEME_COLOR + '10',
  },
  actionButtonText: {
    fontSize: 13,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  amountContainer: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: 8,
  },
  amountInput: {
    flex: 1,
  },
  maxButton: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderRadius: 8,
    marginTop: 28,
    height: 48,
    justifyContent: 'center',
  },
  maxButtonText: {
    fontSize: 14,
    color: '#FFF',
    fontWeight: '600',
  },
  feeInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  feeText: {
    fontSize: 13,
    color: '#999',
  },
  submitButton: {
    marginTop: 8,
  },
  tips: {
    gap: 12,
  },
  tipItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
  },
  tipText: {
    fontSize: 13,
    color: '#666',
  },
  bottomNav: {
    position: 'absolute',
    bottom: 0,
    left: '50%',
    transform: [{ translateX: -207 }],
    width: 414,
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'center',
    backgroundColor: '#FFF',
    paddingTop: 8,
    paddingBottom: 8,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  bottomNavItem: {
    alignItems: 'center',
    paddingVertical: 4,
    flex: 1,
  },
  bottomNavItemActive: {},
  bottomNavIcon: {
    fontSize: 22,
    marginBottom: 2,
  },
  bottomNavLabel: {
    fontSize: 12,
    color: '#999',
    fontWeight: '500',
  },
  bottomNavLabelActive: {
    color: THEME_COLOR,
  },
});
