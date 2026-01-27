/**
 * 星尘玄鉴 - 交易记录页面
 * 显示钱包交易历史
 * 主题色：金棕色 #B2955D
 */

import { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { BottomNavBar } from '@/components/BottomNavBar';
import { Card, LoadingSpinner, EmptyState } from '@/components/common';
import { useWallet, useAsync } from '@/hooks';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

interface Transaction {
  id: string;
  type: 'send' | 'receive';
  amount: string;
  address: string;
  timestamp: Date;
  status: 'pending' | 'confirmed' | 'failed';
  hash: string;
}

export default function TransactionsPage() {
  const router = useRouter();
  const { address } = useWallet();
  const { execute, isLoading } = useAsync();

  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const loadTransactions = async () => {
    await execute(async () => {
      try {
        const { getApi } = await import('@/lib/api');
        const api = getApi();
        
        if (!address) {
          setTransactions([]);
          return;
        }

        // 从链上获取交易记录
        // 监听 system.events 并解析转账事件
        const events = await api.query.system.events();
        const txList: Transaction[] = [];

        // 获取最近的区块头
        const header = await api.rpc.chain.getHeader();
        const currentBlock = header.number.toNumber();

        // 查询最近 1000 个区块的事件（可调整）
        const blocksToCheck = Math.min(1000, currentBlock);
        
        for (let i = 0; i < blocksToCheck; i++) {
          const blockNumber = currentBlock - i;
          const blockHash = await api.rpc.chain.getBlockHash(blockNumber);
          const blockEvents = await api.query.system.events.at(blockHash);
          const block = await api.rpc.chain.getBlock(blockHash);
          
          for (const record of blockEvents) {
            const { event } = record;
            
            // 检查是否是转账事件
            if (api.events.balances.Transfer.is(event)) {
              const [from, to, amount] = event.data;
              const fromStr = from.toString();
              const toStr = to.toString();
              
              // 只记录与当前地址相关的交易
              if (fromStr === address || toStr === address) {
                const timestamp = block.block.header.number.toNumber();
                txList.push({
                  id: `${blockNumber}-${record.phase.toString()}`,
                  type: fromStr === address ? 'send' : 'receive',
                  amount: (Number(amount.toBigInt()) / 1e12).toFixed(4),
                  address: fromStr === address ? toStr : fromStr,
                  timestamp: new Date(timestamp * 6000), // 假设 6 秒出块
                  status: 'confirmed',
                  hash: blockHash.toHex(),
                });
              }
            }
          }
          
          // 限制返回数量
          if (txList.length >= 50) break;
        }

        setTransactions(txList);
      } catch (error) {
        console.error('Load transactions error:', error);
        // API 未连接时显示空列表
        setTransactions([]);
      }
    });
  };

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await loadTransactions();
    setIsRefreshing(false);
  };

  useEffect(() => {
    loadTransactions();
  }, []);

  const formatAddress = (addr: string) => {
    if (!addr || addr.length < 16) return addr;
    return `${addr.slice(0, 8)}...${addr.slice(-8)}`;
  };

  const formatDate = (date: Date) => {
    return date.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusColor = (status: Transaction['status']) => {
    switch (status) {
      case 'confirmed':
        return '#27AE60';
      case 'pending':
        return '#F39C12';
      case 'failed':
        return '#E74C3C';
      default:
        return '#999';
    }
  };

  const getStatusText = (status: Transaction['status']) => {
    switch (status) {
      case 'confirmed':
        return '已确认';
      case 'pending':
        return '处理中';
      case 'failed':
        return '失败';
      default:
        return '';
    }
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>交易记录</Text>
        <View style={styles.placeholder} />
      </View>

      {/* 钱包地址 */}
      <Card style={styles.addressCard}>
        <Text style={styles.addressLabel}>当前地址</Text>
        <Text style={styles.addressText}>{formatAddress(address || '')}</Text>
      </Card>

      {isLoading ? (
        <LoadingSpinner text="加载中..." />
      ) : transactions.length === 0 ? (
        <ScrollView
          contentContainerStyle={styles.emptyContainer}
          showsVerticalScrollIndicator={false}
          refreshControl={
            <RefreshControl
              refreshing={isRefreshing}
              onRefresh={handleRefresh}
              tintColor={THEME_COLOR}
            />
          }
        >
          <EmptyState
            icon="receipt-outline"
            title="暂无交易记录"
            description="您的交易记录将显示在这里"
          />
          <Pressable
            style={styles.refreshButton}
            onPress={handleRefresh}
          >
            <Ionicons name="refresh-outline" size={18} color={THEME_COLOR} />
            <Text style={styles.refreshButtonText}>刷新</Text>
          </Pressable>
        </ScrollView>
      ) : (
        <ScrollView
          style={styles.listContainer}
          contentContainerStyle={styles.listContent}
          showsVerticalScrollIndicator={false}
          refreshControl={
            <RefreshControl
              refreshing={isRefreshing}
              onRefresh={handleRefresh}
              tintColor={THEME_COLOR}
            />
          }
        >
          {transactions.map((tx) => (
            <Pressable key={tx.id} style={styles.txItem}>
              <View style={[styles.txIcon, tx.type === 'send' ? styles.txIconSend : styles.txIconReceive]}>
                <Ionicons
                  name={tx.type === 'send' ? 'arrow-up' : 'arrow-down'}
                  size={20}
                  color={tx.type === 'send' ? '#E74C3C' : '#27AE60'}
                />
              </View>
              <View style={styles.txInfo}>
                <Text style={styles.txType}>
                  {tx.type === 'send' ? '转出' : '转入'}
                </Text>
                <Text style={styles.txAddress}>{formatAddress(tx.address)}</Text>
              </View>
              <View style={styles.txRight}>
                <Text
                  style={[
                    styles.txAmount,
                    { color: tx.type === 'send' ? '#E74C3C' : '#27AE60' },
                  ]}
                >
                  {tx.type === 'send' ? '-' : '+'}{tx.amount} DUST
                </Text>
                <View style={styles.txMeta}>
                  <View
                    style={[
                      styles.statusDot,
                      { backgroundColor: getStatusColor(tx.status) },
                    ]}
                  />
                  <Text style={styles.txTime}>{formatDate(tx.timestamp)}</Text>
                </View>
              </View>
            </Pressable>
          ))}
        </ScrollView>
      )}

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />
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
  addressCard: {
    marginHorizontal: 16,
    marginTop: 16,
  },
  addressLabel: {
    fontSize: 12,
    color: '#8B6914',
    marginBottom: 6,
  },
  addressText: {
    fontSize: 14,
    color: '#333',
    fontFamily: 'monospace',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 40,
  },
  refreshButton: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    backgroundColor: '#FFF',
    paddingHorizontal: 20,
    paddingVertical: 10,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  refreshButtonText: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  listContainer: {
    flex: 1,
  },
  listContent: {
    padding: 16,
    paddingBottom: 40,
  },
  txItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 14,
    marginBottom: 10,
    gap: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.04,
    shadowRadius: 4,
    elevation: 1,
  },
  txIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    justifyContent: 'center',
    alignItems: 'center',
  },
  txIconSend: {
    backgroundColor: '#FFEBEE',
  },
  txIconReceive: {
    backgroundColor: '#E8F5E9',
  },
  txInfo: {
    flex: 1,
  },
  txType: {
    fontSize: 15,
    fontWeight: '500',
    color: '#333',
    marginBottom: 4,
  },
  txAddress: {
    fontSize: 12,
    color: '#999',
    fontFamily: 'monospace',
  },
  txRight: {
    alignItems: 'flex-end',
  },
  txAmount: {
    fontSize: 15,
    fontWeight: '600',
    marginBottom: 4,
  },
  txMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  statusDot: {
    width: 6,
    height: 6,
    borderRadius: 3,
  },
  txTime: {
    fontSize: 12,
    color: '#999',
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
