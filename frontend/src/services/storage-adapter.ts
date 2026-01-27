/**
 * 星尘玄鉴 - 存储适配器
 * 为不同平台提供统一的存储接口
 */

import { Platform } from 'react-native';

interface StorageAdapter {
  getItemAsync(key: string, options?: { requireAuthentication?: boolean }): Promise<string | null>;
  setItemAsync(key: string, value: string, options?: { requireAuthentication?: boolean }): Promise<void>;
  deleteItemAsync(key: string): Promise<void>;
}

/**
 * Web 平台使用 IndexedDB 存储
 * 比 localStorage 更安全，不易被 XSS 直接读取
 */
class WebStorageAdapter implements StorageAdapter {
  private prefix = 'stardust_secure_';
  private storage: import('../lib/secure-storage-indexeddb').SecureStorageInterface;

  constructor() {
    // 动态导入以避免循环依赖
    const { secureStorage } = require('../lib/secure-storage-indexeddb');
    this.storage = secureStorage;
  }

  async getItemAsync(key: string): Promise<string | null> {
    try {
      return await this.storage.getItem(this.prefix + key);
    } catch (error) {
      console.warn('[WebStorage] Failed to get:', error);
      return null;
    }
  }

  async setItemAsync(key: string, value: string): Promise<void> {
    try {
      await this.storage.setItem(this.prefix + key, value);
    } catch (error) {
      console.warn('[WebStorage] Failed to save:', error);
      throw error;
    }
  }

  async deleteItemAsync(key: string): Promise<void> {
    try {
      await this.storage.removeItem(this.prefix + key);
    } catch (error) {
      console.warn('[WebStorage] Failed to delete:', error);
      throw error;
    }
  }
}

/**
 * 原生平台使用 expo-secure-store
 */
class NativeStorageAdapter implements StorageAdapter {
  private secureStore: typeof import('expo-secure-store') | null = null;

  private async getSecureStore() {
    if (!this.secureStore) {
      this.secureStore = await import('expo-secure-store');
    }
    return this.secureStore;
  }

  async getItemAsync(key: string, options?: { requireAuthentication?: boolean }): Promise<string | null> {
    const store = await this.getSecureStore();
    return store.getItemAsync(key, options);
  }

  async setItemAsync(key: string, value: string, options?: { requireAuthentication?: boolean }): Promise<void> {
    const store = await this.getSecureStore();
    await store.setItemAsync(key, value, options);
  }

  async deleteItemAsync(key: string): Promise<void> {
    const store = await this.getSecureStore();
    await store.deleteItemAsync(key);
  }
}

/**
 * 根据平台选择适当的存储适配器
 */
export const storageAdapter: StorageAdapter = Platform.OS === 'web'
  ? new WebStorageAdapter()
  : new NativeStorageAdapter();
