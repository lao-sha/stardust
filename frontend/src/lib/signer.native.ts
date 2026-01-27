/**
 * 星尘玄鉴 - 移动端签名服务
 * 使用内置钱包进行交易签名
 * 
 * 安全特性：
 * - 5分钟无操作自动锁定
 * - 页面卸载时强制清理密钥
 */

import { ApiPromise } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { retrieveEncryptedMnemonic, getCurrentAddress } from '@/lib/keystore';
import { AppState, AppStateStatus } from 'react-native';

/**
 * 自动锁定配置
 */
const AUTO_LOCK_CONFIG = {
  /** 自动锁定超时时间（毫秒）- 5分钟 */
  TIMEOUT_MS: 5 * 60 * 1000,
  /** 后台超时时间（毫秒）- 1分钟 */
  BACKGROUND_TIMEOUT_MS: 60 * 1000,
} as const;

/**
 * 移动端签名器
 */
class MobileSigner {
  private static instance: MobileSigner;
  private keyring: Keyring | null = null;
  private currentPair: KeyringPair | null = null;
  private isInitialized = false;
  private lastActivityTime: number = 0;
  private autoLockTimer: ReturnType<typeof setTimeout> | null = null;
  private appStateSubscription: { remove: () => void } | null = null;

  private constructor() {
    this.setupAutoLock();
  }

  /**
   * 获取单例实例
   */
  static getInstance(): MobileSigner {
    if (!MobileSigner.instance) {
      MobileSigner.instance = new MobileSigner();
    }
    return MobileSigner.instance;
  }

  /**
   * 设置自动锁定机制
   */
  private setupAutoLock(): void {
    // 监听应用状态变化（前台/后台）
    this.appStateSubscription = AppState.addEventListener(
      'change',
      this.handleAppStateChange.bind(this)
    );
  }

  /**
   * 处理应用状态变化
   */
  private handleAppStateChange(nextAppState: AppStateStatus): void {
    if (nextAppState === 'background' || nextAppState === 'inactive') {
      // 应用进入后台，设置延迟锁定
      this.autoLockTimer = setTimeout(() => {
        if (this.currentPair) {
          console.log('[MobileSigner] Auto-locking due to background timeout');
          this.lock();
        }
      }, AUTO_LOCK_CONFIG.BACKGROUND_TIMEOUT_MS);
    } else if (nextAppState === 'active') {
      // 应用回到前台，取消延迟锁定
      if (this.autoLockTimer) {
        clearTimeout(this.autoLockTimer);
        this.autoLockTimer = null;
      }
      // 检查是否超过最大空闲时间
      this.checkAutoLock();
    }
  }

  /**
   * 记录用户活动时间
   */
  recordActivity(): void {
    this.lastActivityTime = Date.now();
  }

  /**
   * 检查是否需要自动锁定
   */
  private checkAutoLock(): void {
    if (!this.currentPair) return;

    const now = Date.now();
    const idleTime = now - this.lastActivityTime;

    if (idleTime >= AUTO_LOCK_CONFIG.TIMEOUT_MS) {
      console.log('[MobileSigner] Auto-locking due to inactivity');
      this.lock();
    }
  }

  /**
   * 初始化签名器
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      console.log('[MobileSigner] Initializing...');

      // 等待加密库准备就绪
      await cryptoWaitReady();

      // 创建 Keyring
      this.keyring = new Keyring({ type: 'sr25519' });

      this.isInitialized = true;
      console.log('[MobileSigner] Initialized');
    } catch (error) {
      console.error('[MobileSigner] Initialize error:', error);
      throw error;
    }
  }

  /**
   * 解锁钱包并加载密钥对
   */
  async unlockWallet(password: string): Promise<KeyringPair> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    if (!this.keyring) {
      throw new Error('Keyring not initialized');
    }

    try {
      console.log('[MobileSigner] Unlocking wallet...');

      // 先锁定旧的密钥对
      this.lock();

      // 获取当前地址
      const address = await getCurrentAddress();
      if (!address) {
        throw new Error('No wallet found');
      }

      // 解密助记词
      const mnemonic = await retrieveEncryptedMnemonic(password);

      // 从助记词创建密钥对
      this.currentPair = this.keyring.addFromMnemonic(mnemonic);

      // 记录活动时间
      this.recordActivity();

      console.log('[MobileSigner] Wallet unlocked:', this.currentPair.address);

      return this.currentPair;
    } catch (error) {
      console.error('[MobileSigner] Unlock error:', error);
      throw new Error('Failed to unlock wallet. Please check your password.');
    }
  }

  /**
   * 获取当前密钥对
   */
  getCurrentPair(): KeyringPair | null {
    // 检查是否超时
    this.checkAutoLock();
    
    if (this.currentPair) {
      // 记录活动
      this.recordActivity();
    }
    
    return this.currentPair;
  }

  /**
   * 检查是否已解锁
   */
  isUnlocked(): boolean {
    this.checkAutoLock();
    return this.currentPair !== null;
  }

  /**
   * 锁定钱包（安全清理密钥）
   */
  lock(): void {
    if (this.currentPair) {
      // 从 keyring 中移除密钥对
      if (this.keyring) {
        try {
          this.keyring.removePair(this.currentPair.address);
        } catch {
          // 忽略移除错误
        }
      }
      this.currentPair = null;
      console.log('[MobileSigner] Wallet locked and key cleared');
    }

    // 清除定时器
    if (this.autoLockTimer) {
      clearTimeout(this.autoLockTimer);
      this.autoLockTimer = null;
    }
  }

  /**
   * 签名并发送交易
   */
  async signAndSend(
    api: ApiPromise,
    tx: any,
    onStatusChange?: (status: string) => void
  ): Promise<{ blockHash: string; events: any[] }> {
    // 检查是否超时
    this.checkAutoLock();

    if (!this.currentPair) {
      throw new Error('Wallet is locked. Please unlock first.');
    }

    // 记录活动
    this.recordActivity();

    return new Promise((resolve, reject) => {
      tx.signAndSend(
        this.currentPair,
        ({ status, events, dispatchError }: any) => {
          // 更新状态
          if (status.isReady) {
            onStatusChange?.('准备中...');
          } else if (status.isBroadcast) {
            onStatusChange?.('广播中...');
          } else if (status.isInBlock) {
            onStatusChange?.('已打包...');
            console.log('[MobileSigner] Transaction in block:', status.asInBlock.toHex());

            // 检查错误
            if (dispatchError) {
              if (dispatchError.isModule) {
                const decoded = api.registry.findMetaError(dispatchError.asModule);
                const { docs, name, section } = decoded;
                reject(new Error(`${section}.${name}: ${docs.join(' ')}`));
              } else {
                reject(new Error(dispatchError.toString()));
              }
              return;
            }

            // 交易成功
            resolve({
              blockHash: status.asInBlock.toHex(),
              events,
            });
          } else if (status.isFinalized) {
            onStatusChange?.('已确认');
            console.log('[MobileSigner] Transaction finalized:', status.asFinalized.toHex());
          }
        }
      ).catch(reject);
    });
  }

  /**
   * 获取账户地址
   */
  getAddress(): string | null {
    return this.currentPair?.address || null;
  }

  /**
   * 销毁服务（清理所有资源）
   */
  destroy(): void {
    this.lock();
    if (this.appStateSubscription) {
      this.appStateSubscription.remove();
      this.appStateSubscription = null;
    }
  }
}

// 导出单例
export const mobileSigner = MobileSigner.getInstance();

// 导出便捷方法
export const initializeSigner = () => mobileSigner.initialize();
export const unlockWallet = (password: string) => mobileSigner.unlockWallet(password);
export const isWalletUnlocked = () => mobileSigner.isUnlocked();
export const lockWallet = () => mobileSigner.lock();
export const getCurrentPair = () => mobileSigner.getCurrentPair();
export const getSignerAddress = () => mobileSigner.getAddress();
export const signAndSendTransaction = (
  api: ApiPromise,
  tx: any,
  onStatusChange?: (status: string) => void
) => mobileSigner.signAndSend(api, tx, onStatusChange);
