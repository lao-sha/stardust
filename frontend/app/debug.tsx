/**
 * 调试页面 - 仅在开发环境下可用
 * 生产环境会重定向到首页
 */

import React, { useEffect, useState } from 'react';
import { View, Text, StyleSheet, ScrollView, Button, Pressable } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWalletStore } from '@/stores/wallet.store';
import * as SecureStore from 'expo-secure-store';

const __DEV__ = process.env.NODE_ENV === 'development' || process.env.EXPO_PUBLIC_DEV_MODE === 'true';

export default function DebugPage() {
  const router = useRouter();
  const [logs, setLogs] = useState<string[]>([]);
  const walletState = useWalletStore();

  // 生产环境下显示提示并返回
  if (!__DEV__) {
    return (
      <View style={styles.prodContainer}>
        <Ionicons name="lock-closed-outline" size={64} color="#999" />
        <Text style={styles.prodTitle}>页面不可用</Text>
        <Text style={styles.prodDesc}>此页面仅在开发环境下可用</Text>
        <Pressable style={styles.backButton} onPress={() => router.replace('/')}>
          <Text style={styles.backButtonText}>返回首页</Text>
        </Pressable>
      </View>
    );
  }

  const addLog = (message: string) => {
    setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ${message}`]);
  };

  useEffect(() => {
    addLog('Debug page mounted');
    addLog(`isReady: ${walletState.isReady}`);
    addLog(`hasWallet: ${walletState.hasWallet}`);
    addLog(`isLocked: ${walletState.isLocked}`);
    addLog(`address: ${walletState.address}`);
  }, []);

  const testSecureStore = async () => {
    try {
      addLog('Testing SecureStore...');
      await SecureStore.setItemAsync('test_key', 'test_value');
      const value = await SecureStore.getItemAsync('test_key');
      addLog(`SecureStore test: ${value === 'test_value' ? 'PASS' : 'FAIL'}`);
      await SecureStore.deleteItemAsync('test_key');
    } catch (error) {
      addLog(`SecureStore error: ${error}`);
    }
  };

  const testInitialize = async () => {
    try {
      addLog('Testing wallet initialize...');
      await walletState.initialize();
      addLog('Initialize complete');
    } catch (error) {
      addLog(`Initialize error: ${error}`);
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable onPress={() => router.back()} style={styles.backIcon}>
          <Ionicons name="chevron-back" size={24} color="#fff" />
        </Pressable>
        <Text style={styles.title}>调试信息</Text>
        <View style={styles.devBadge}>
          <Text style={styles.devBadgeText}>DEV</Text>
        </View>
      </View>
      
      <View style={styles.stateBox}>
        <Text style={styles.stateText}>isReady: {String(walletState.isReady)}</Text>
        <Text style={styles.stateText}>hasWallet: {String(walletState.hasWallet)}</Text>
        <Text style={styles.stateText}>isLocked: {String(walletState.isLocked)}</Text>
        <Text style={styles.stateText}>address: {walletState.address || 'null'}</Text>
        <Text style={styles.stateText}>error: {walletState.error || 'null'}</Text>
      </View>

      <View style={styles.buttons}>
        <Button title="测试 SecureStore" onPress={testSecureStore} />
        <Button title="测试初始化" onPress={testInitialize} />
      </View>

      <ScrollView style={styles.logContainer}>
        {logs.map((log, index) => (
          <Text key={index} style={styles.logText}>{log}</Text>
        ))}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0a0a0f',
    padding: 20,
    paddingTop: 60,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 20,
  },
  backIcon: {
    padding: 4,
    marginRight: 12,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
    flex: 1,
  },
  devBadge: {
    backgroundColor: '#8b5cf6',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  devBadgeText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
  },
  stateBox: {
    backgroundColor: '#1a1a2e',
    padding: 15,
    borderRadius: 8,
    marginBottom: 20,
  },
  stateText: {
    color: '#8b5cf6',
    fontSize: 14,
    marginBottom: 5,
  },
  buttons: {
    flexDirection: 'row',
    gap: 10,
    marginBottom: 20,
  },
  logContainer: {
    flex: 1,
    backgroundColor: '#1a1a2e',
    padding: 10,
    borderRadius: 8,
  },
  logText: {
    color: '#9ca3af',
    fontSize: 12,
    marginBottom: 3,
  },
  // 生产环境样式
  prodContainer: {
    flex: 1,
    backgroundColor: '#F5F5F7',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  prodTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
    marginTop: 16,
    marginBottom: 8,
  },
  prodDesc: {
    fontSize: 14,
    color: '#666',
    marginBottom: 24,
  },
  backButton: {
    backgroundColor: '#B2955D',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  backButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
