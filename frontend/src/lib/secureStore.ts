/**
 * 星尘玄鉴 - 安全存储（跨平台版本）
 * 
 * Web: 使用 IndexedDB（比 localStorage 更安全）
 * Native: 使用 expo-secure-store（硬件级安全存储）
 * 
 * ⚠️ 注意：此模块用于非敏感数据存储
 * 敏感数据（如助记词）应使用 secure-storage.web.ts
 */

import { Platform } from 'react-native';
import { secureStorage } from './secure-storage-indexeddb';

// Web 存储实现（使用 IndexedDB）
const webStorage = {
  async setItemAsync(key: string, value: string): Promise<void> {
    await secureStorage.setItem(key, value);
  },

  async getItemAsync(key: string): Promise<string | null> {
    return await secureStorage.getItem(key);
  },

  async deleteItemAsync(key: string): Promise<void> {
    await secureStorage.removeItem(key);
  },
};

// 根据平台选择存储实现
let SecureStore: typeof webStorage;

if (Platform.OS === 'web') {
  SecureStore = webStorage;
} else {
  // 动态导入 native 模块
  SecureStore = require('expo-secure-store');
}

export async function setItemAsync(key: string, value: string): Promise<void> {
  return SecureStore.setItemAsync(key, value);
}

export async function getItemAsync(key: string): Promise<string | null> {
  return SecureStore.getItemAsync(key);
}

export async function deleteItemAsync(key: string): Promise<void> {
  return SecureStore.deleteItemAsync(key);
}
