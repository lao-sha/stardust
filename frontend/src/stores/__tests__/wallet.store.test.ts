/**
 * 钱包 Store 测试
 */

import { renderHook, act } from '@testing-library/react-hooks';
import { useWalletStore } from '../wallet.store';

// Mock 依赖
jest.mock('@/lib/keystore', () => ({
  initializeCrypto: jest.fn().mockResolvedValue(undefined),
  generateMnemonic: jest.fn().mockReturnValue('test mnemonic words here'),
  validateMnemonic: jest.fn().mockReturnValue(true),
  createKeyPairFromMnemonic: jest.fn().mockReturnValue({
    address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
  }),
  storeEncryptedMnemonic: jest.fn().mockResolvedValue(undefined),
  retrieveEncryptedMnemonic: jest.fn().mockResolvedValue('test mnemonic'),
  hasWallet: jest.fn().mockResolvedValue(false),
  getStoredAddress: jest.fn().mockResolvedValue(null),
  deleteWallet: jest.fn().mockResolvedValue(undefined),
  deleteWalletByAddress: jest.fn().mockResolvedValue(undefined),
  loadAllKeystores: jest.fn().mockResolvedValue([]),
  getCurrentAddress: jest.fn().mockResolvedValue(null),
  setCurrentAddress: jest.fn().mockResolvedValue(undefined),
  getAlias: jest.fn().mockResolvedValue(null),
  setAlias: jest.fn().mockResolvedValue(undefined),
}));

jest.mock('@/lib/api', () => ({
  getApi: jest.fn(),
  isApiReady: jest.fn().mockReturnValue(false),
}));

describe('Wallet Store', () => {
  beforeEach(() => {
    // 重置 store
    useWalletStore.setState({
      isReady: false,
      hasWallet: false,
      isLocked: true,
      address: null,
      error: null,
      isLoading: false,
      accounts: [],
      loadingAccounts: false,
    });
  });

  describe('initialize', () => {
    it('should initialize successfully', async () => {
      const { result } = renderHook(() => useWalletStore());

      await act(async () => {
        await result.current.initialize();
      });

      expect(result.current.isReady).toBe(true);
      expect(result.current.error).toBeNull();
    });

    it('should handle initialization errors', async () => {
      const mockError = new Error('Init failed');
      const { initializeCrypto } = require('@/lib/keystore');
      initializeCrypto.mockRejectedValueOnce(mockError);

      const { result } = renderHook(() => useWalletStore());

      await act(async () => {
        try {
          await result.current.initialize();
        } catch (error) {
          // Expected to throw
        }
      });

      expect(result.current.error).toContain('初始化失败');
    });
  });

  describe('createWallet', () => {
    it('should create wallet successfully', async () => {
      const { result } = renderHook(() => useWalletStore());

      let mnemonic: string = '';
      await act(async () => {
        mnemonic = await result.current.createWallet('password123');
      });

      expect(mnemonic).toBe('test mnemonic words here');
      expect(result.current.hasWallet).toBe(true);
      expect(result.current.isLocked).toBe(false);
      expect(result.current.address).toBeTruthy();
    });

    it('should reject weak passwords', async () => {
      const { result } = renderHook(() => useWalletStore());

      await act(async () => {
        try {
          await result.current.createWallet('weak');
        } catch (error) {
          expect(error).toBeDefined();
        }
      });

      expect(result.current.hasWallet).toBe(false);
    });
  });

  describe('importWallet', () => {
    it('should import wallet successfully', async () => {
      const { result } = renderHook(() => useWalletStore());

      await act(async () => {
        await result.current.importWallet('test mnemonic words here', 'password123');
      });

      expect(result.current.hasWallet).toBe(true);
      expect(result.current.isLocked).toBe(false);
    });

    it('should reject invalid mnemonic', async () => {
      const { validateMnemonic } = require('@/lib/keystore');
      validateMnemonic.mockReturnValueOnce(false);

      const { result } = renderHook(() => useWalletStore());

      await act(async () => {
        try {
          await result.current.importWallet('invalid mnemonic', 'password123');
        } catch (error) {
          expect(error).toBeDefined();
        }
      });

      expect(result.current.hasWallet).toBe(false);
    });
  });

  describe('unlockWallet', () => {
    it('should unlock wallet successfully', async () => {
      const { result } = renderHook(() => useWalletStore());

      // 先设置为已锁定状态
      act(() => {
        useWalletStore.setState({
          hasWallet: true,
          isLocked: true,
        });
      });

      await act(async () => {
        await result.current.unlockWallet('password123');
      });

      expect(result.current.isLocked).toBe(false);
      expect(result.current.address).toBeTruthy();
    });

    it('should handle wrong password', async () => {
      const { retrieveEncryptedMnemonic } = require('@/lib/keystore');
      retrieveEncryptedMnemonic.mockRejectedValueOnce(new Error('密码错误'));

      const { result } = renderHook(() => useWalletStore());

      act(() => {
        useWalletStore.setState({
          hasWallet: true,
          isLocked: true,
        });
      });

      await act(async () => {
        try {
          await result.current.unlockWallet('wrongpassword');
        } catch (error) {
          expect(error).toBeDefined();
        }
      });

      expect(result.current.isLocked).toBe(true);
      expect(result.current.error).toContain('密码错误');
    });
  });

  describe('lockWallet', () => {
    it('should lock wallet', () => {
      const { result } = renderHook(() => useWalletStore());

      act(() => {
        useWalletStore.setState({
          hasWallet: true,
          isLocked: false,
          address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        });
      });

      act(() => {
        result.current.lockWallet();
      });

      expect(result.current.isLocked).toBe(true);
    });
  });

  describe('deleteWallet', () => {
    it('should delete wallet successfully', async () => {
      const { result } = renderHook(() => useWalletStore());

      act(() => {
        useWalletStore.setState({
          hasWallet: true,
          address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        });
      });

      await act(async () => {
        await result.current.deleteWallet();
      });

      expect(result.current.hasWallet).toBe(false);
      expect(result.current.address).toBeNull();
    });
  });

  describe('switchWallet', () => {
    it('should switch to another wallet', async () => {
      const { result } = renderHook(() => useWalletStore());
      const newAddress = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

      await act(async () => {
        await result.current.switchWallet(newAddress);
      });

      expect(result.current.address).toBe(newAddress);
      expect(result.current.isLocked).toBe(true); // 切换后需要重新解锁
    });
  });

  describe('setWalletAlias', () => {
    it('should set wallet alias', async () => {
      const { result } = renderHook(() => useWalletStore());
      const address = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

      act(() => {
        useWalletStore.setState({
          accounts: [
            {
              address,
              alias: '旧别名',
              balance: '0',
              isCurrentAccount: true,
            },
          ],
        });
      });

      await act(async () => {
        await result.current.setWalletAlias(address, '新别名');
      });

      const account = result.current.accounts.find(a => a.address === address);
      expect(account?.alias).toBe('新别名');
    });
  });

  describe('clearError', () => {
    it('should clear error', () => {
      const { result } = renderHook(() => useWalletStore());

      act(() => {
        useWalletStore.setState({ error: 'Test error' });
      });

      expect(result.current.error).toBe('Test error');

      act(() => {
        result.current.clearError();
      });

      expect(result.current.error).toBeNull();
    });
  });
});
