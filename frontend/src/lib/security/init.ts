/**
 * 星尘玄鉴 - 安全初始化
 * 
 * 在应用启动时初始化安全功能：
 * 1. 防止点击劫持
 * 2. 检查安全环境
 * 3. 初始化加密系统
 */

import { Platform } from 'react-native';
import { preventClickjacking, isInIframe } from './xss-protection';
import { initializeCrypto } from '../secure-storage.web';

/**
 * 初始化 Web 平台安全功能
 */
export async function initWebSecurity(): Promise<void> {
  if (Platform.OS !== 'web') {
    return;
  }

  try {
    // 1. 防止点击劫持
    if (typeof window !== 'undefined') {
      preventClickjacking();
    }

    // 2. 初始化加密系统（IndexedDB + Web Crypto API）
    await initializeCrypto();

    console.log('[Security] Web security initialized');
  } catch (error) {
    console.error('[Security] Failed to initialize web security:', error);
    // 不阻止应用启动，但记录错误
  }
}

/**
 * 检查安全环境
 */
export function checkSecurityEnvironment(): {
  indexedDB: boolean;
  webCrypto: boolean;
  inIframe: boolean;
} {
  if (Platform.OS !== 'web') {
    return {
      indexedDB: true, // Native 平台不需要
      webCrypto: true, // Native 平台不需要
      inIframe: false,
    };
  }

  return {
    indexedDB: typeof indexedDB !== 'undefined',
    webCrypto: typeof crypto !== 'undefined' && typeof crypto.subtle !== 'undefined',
    inIframe: isInIframe(),
  };
}

