/**
 * 钱包操作 Hook
 */

import { useWalletStore } from '@/stores/wallet.store';
import { useCallback } from 'react';

export function useWallet() {
  const {
    address,
    balance,
    isUnlocked,
    isLoading,
    error,
    unlockWallet,
    lockWallet,
    refreshBalance,
  } = useWalletStore();

  const ensureUnlocked = useCallback(async (password?: string) => {
    if (isUnlocked) return true;

    if (!password) {
      throw new Error('需要密码解锁钱包');
    }

    try {
      await unlockWallet(password);
      return true;
    } catch (error) {
      console.error('Unlock wallet error:', error);
      return false;
    }
  }, [isUnlocked, unlockWallet]);

  return {
    address,
    balance,
    isUnlocked,
    isLoading,
    error,
    unlockWallet,
    lockWallet,
    refreshBalance,
    ensureUnlocked,
  };
}
