/**
 * 星尘玄鉴 - 钱包状态管理
 * 使用 Zustand 管理多钱包状态
 */

import { create } from 'zustand';
import { WalletError, AuthenticationError } from '@/lib/errors';
import {
  initializeCrypto,
  generateMnemonic,
  validateMnemonic,
  createKeyPairFromMnemonic,
  storeEncryptedMnemonic,
  retrieveEncryptedMnemonic,
  hasWallet as checkHasWallet,
  getStoredAddress,
  deleteWallet as removeWallet,
  deleteWalletByAddress,
  loadAllKeystores,
  getCurrentAddress,
  setCurrentAddress,
  getAlias,
  setAlias,
  type LocalKeystore,
} from '@/lib/keystore';
import { getApi, isApiReady } from '@/lib/api';
import type { AccountInfo } from '@/types/substrate.types';

/** 账户信息的 JSON 表示 */
interface AccountInfoJson {
  nonce?: number;
  consumers?: number;
  providers?: number;
  sufficients?: number;
  data?: {
    free?: string | number;
    reserved?: string | number;
    frozen?: string | number;
  };
}

/**
 * 账户资产信息
 */
export interface AccountAsset {
  address: string;
  alias: string;
  balance: string;
  isCurrentAccount: boolean;
}

interface WalletState {
  // 状态
  isReady: boolean;
  hasWallet: boolean;
  isLocked: boolean;
  address: string | null;
  error: string | null;
  isLoading: boolean;

  // 多钱包状态
  accounts: AccountAsset[];
  loadingAccounts: boolean;

  // 操作方法
  initialize: () => Promise<void>;
  createWallet: (password: string) => Promise<string>;
  importWallet: (mnemonic: string, password: string) => Promise<void>;
  unlockWallet: (password: string) => Promise<void>;
  lockWallet: () => void;
  deleteWallet: () => Promise<void>;
  deleteWalletByAddress: (address: string) => Promise<void>;
  clearError: () => void;

  // 多钱包方法
  loadAllAccounts: () => Promise<void>;
  switchWallet: (address: string) => Promise<void>;
  setWalletAlias: (address: string, alias: string) => Promise<void>;
}

export const useWalletStore = create<WalletState>()((set, get) => ({
  // 初始状态
  isReady: false,
  hasWallet: false,
  isLocked: true,
  address: null,
  error: null,
  isLoading: false,

  // 多钱包状态
  accounts: [],
  loadingAccounts: false,

  /**
   * 初始化钱包系统
   */
  initialize: async () => {
    try {
      console.log('[Wallet] Starting initialization...');
      set({ isLoading: true, error: null });

      console.log('[Wallet] Initializing crypto...');
      await initializeCrypto();
      console.log('[Wallet] Crypto initialized');

      console.log('[Wallet] Checking for existing wallet...');
      const hasWallet = await checkHasWallet();
      console.log('[Wallet] Has wallet:', hasWallet);

      const address = hasWallet ? await getStoredAddress() : null;
      console.log('[Wallet] Stored address:', address);

      set({
        isReady: true,
        hasWallet,
        isLocked: hasWallet,
        address,
        error: null,
        isLoading: false,
      });

      console.log('[Wallet] Initialized successfully:', { hasWallet, address });
      
      // 延迟加载账户列表，不阻塞初始化
      if (hasWallet) {
        setTimeout(() => {
          console.log('[Wallet] Loading accounts in background...');
          get().loadAllAccounts().catch(err => {
            console.error('[Wallet] Failed to load accounts:', err);
          });
        }, 100);
      }
    } catch (error) {
      console.error('[Wallet] Initialize error:', error);
      set({
        isReady: true,
        hasWallet: false,
        isLocked: true,
        error: '初始化失败: ' + (error instanceof Error ? error.message : String(error)),
        isLoading: false,
      });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 加载所有账户
   */
  loadAllAccounts: async () => {
    try {
      set({ loadingAccounts: true });
      const keystores = await loadAllKeystores();
      const currentAddr = await getCurrentAddress();

      const accounts: AccountAsset[] = await Promise.all(
        keystores.map(async (ks: LocalKeystore) => {
          const alias = await getAlias(ks.address) || `钱包 ${ks.address.slice(0, 6)}`;
          
          // 查询链上余额
          let balance = '0.0000';
          try {
            if (isApiReady()) {
              const api = getApi();
              const accountInfo = await api.query.system.account(ks.address);
              const data = accountInfo.toJSON() as AccountInfoJson;
              const freeBalance = BigInt(data?.data?.free ?? 0);
              balance = (Number(freeBalance) / 1e12).toFixed(4);
            }
          } catch (balanceError) {
            console.warn('[Wallet] Failed to fetch balance for', ks.address, balanceError);
          }
          
          return {
            address: ks.address,
            alias,
            balance,
            isCurrentAccount: ks.address === currentAddr,
          };
        })
      );

      set({ accounts });
    } catch (error) {
      console.error('[Wallet] Load accounts error:', error);
    } finally {
      set({ loadingAccounts: false });
    }
  },

  /**
   * 创建新钱包
   */
  createWallet: async (password: string) => {
    try {
      set({ isLoading: true, error: null });

      if (!password || password.length < 8) {
        throw new WalletError('密码至少需要 8 位');
      }

      const mnemonic = generateMnemonic();
      const pair = createKeyPairFromMnemonic(mnemonic);

      await storeEncryptedMnemonic(mnemonic, password, pair.address);

      set({
        hasWallet: true,
        isLocked: false,
        address: pair.address,
        error: null,
      });

      // 重新加载账户列表
      await get().loadAllAccounts();

      console.log('[Wallet] Created:', pair.address);
      return mnemonic;
    } catch (error) {
      const message = error instanceof Error ? error.message : '创建钱包失败';
      console.error('[Wallet] Create error:', error);
      set({ error: message });
      throw new WalletError(message, error);
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 导入钱包
   */
  importWallet: async (mnemonic: string, password: string) => {
    try {
      set({ isLoading: true, error: null });

      if (!validateMnemonic(mnemonic)) {
        throw new WalletError('无效的助记词');
      }

      if (!password || password.length < 8) {
        throw new WalletError('密码至少需要 8 位');
      }

      const pair = createKeyPairFromMnemonic(mnemonic);

      await storeEncryptedMnemonic(mnemonic, password, pair.address);

      set({
        hasWallet: true,
        isLocked: false,
        address: pair.address,
        error: null,
      });

      // 重新加载账户列表
      await get().loadAllAccounts();

      console.log('[Wallet] Imported:', pair.address);
    } catch (error) {
      const message = error instanceof Error ? error.message : '导入钱包失败';
      console.error('[Wallet] Import error:', error);
      set({ error: message });
      throw new WalletError(message, error);
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 切换钱包
   */
  switchWallet: async (address: string) => {
    try {
      set({ isLoading: true, error: null });

      await setCurrentAddress(address);

      set({
        address,
        isLocked: true, // 切换后需要重新解锁
        error: null,
      });

      // 更新账户列表的当前状态
      const accounts = get().accounts.map(acc => ({
        ...acc,
        isCurrentAccount: acc.address === address,
      }));
      set({ accounts });

      console.log('[Wallet] Switched to:', address);
    } catch (error) {
      const message = '切换钱包失败';
      console.error('[Wallet] Switch error:', error);
      set({ error: message });
      throw new WalletError(message, error);
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 设置钱包别名
   */
  setWalletAlias: async (address: string, alias: string) => {
    try {
      await setAlias(address, alias);

      // 更新账户列表中的别名
      const accounts = get().accounts.map(acc =>
        acc.address === address ? { ...acc, alias } : acc
      );
      set({ accounts });

      console.log('[Wallet] Alias set:', address, alias);
    } catch (error) {
      console.error('[Wallet] Set alias error:', error);
      throw new WalletError('设置别名失败', error);
    }
  },

  /**
   * 解锁钱包
   */
  unlockWallet: async (password: string) => {
    try {
      set({ isLoading: true, error: null });

      const mnemonic = await retrieveEncryptedMnemonic(password);
      const pair = createKeyPairFromMnemonic(mnemonic);

      set({
        isLocked: false,
        address: pair.address,
        error: null,
      });

      console.log('[Wallet] Unlocked:', pair.address);
    } catch (error) {
      const message = error instanceof AuthenticationError ? '密码错误' : '解锁失败';
      console.error('[Wallet] Unlock error:', error);
      set({ error: message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 锁定钱包
   */
  lockWallet: () => {
    set({
      isLocked: true,
      error: null,
    });
    console.log('[Wallet] Locked');
  },

  /**
   * 删除当前钱包
   */
  deleteWallet: async () => {
    try {
      set({ isLoading: true, error: null });

      await removeWallet();

      const hasWallet = await checkHasWallet();
      const address = hasWallet ? await getStoredAddress() : null;

      set({
        hasWallet,
        isLocked: hasWallet,
        address,
        error: null,
      });

      // 重新加载账户列表
      if (hasWallet) {
        await get().loadAllAccounts();
      } else {
        set({ accounts: [] });
      }

      console.log('[Wallet] Deleted current wallet');
    } catch (error) {
      console.error('[Wallet] Delete error:', error);
      const message = '删除钱包失败';
      set({ error: message });
      throw new WalletError(message, error);
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 删除指定钱包
   */
  deleteWalletByAddress: async (address: string) => {
    try {
      set({ isLoading: true, error: null });

      await deleteWalletByAddress(address);

      const hasWallet = await checkHasWallet();
      const currentAddr = hasWallet ? await getStoredAddress() : null;

      set({
        hasWallet,
        isLocked: hasWallet,
        address: currentAddr,
        error: null,
      });

      // 重新加载账户列表
      if (hasWallet) {
        await get().loadAllAccounts();
      } else {
        set({ accounts: [] });
      }

      console.log('[Wallet] Deleted wallet:', address);
    } catch (error) {
      console.error('[Wallet] Delete by address error:', error);
      const message = '删除钱包失败';
      set({ error: message });
      throw new WalletError(message, error);
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 清除错误
   */
  clearError: () => {
    set({ error: null });
  },
}));
