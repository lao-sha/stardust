/**
 * 星尘玄鉴 - 钱包类型定义
 */

/**
 * 钱包账户
 */
export interface WalletAccount {
  address: string;
  name: string;
  encryptedSeed?: string;
  createdAt: number;
}

/**
 * 钱包设置
 */
export interface WalletSettings {
  biometricEnabled: boolean;
  autoLockTimeout: number; // 分钟
  defaultNetwork: string;
}

/**
 * 加密钱包数据
 */
export interface EncryptedWalletData {
  accounts?: WalletAccount[];
  version: number;
  encryptedAt?: number;
  encryptedData?: string;
  createdAt?: number;
  iterations?: number;
  salt?: string;
}

/**
 * 安全参数
 */
export const SECURITY_PARAMS = {
  PBKDF2_ITERATIONS: 100000,
  SALT_LENGTH: 32,
  KEY_LENGTH: 32,
  IV_LENGTH: 16,
  PIN_MIN_LENGTH: 6,
  PIN_MAX_ATTEMPTS: 5,
  LOCKOUT_DURATION: 300000, // 5 分钟
  VERSION: 1,
} as const;

/**
 * 默认钱包设置
 */
export const DEFAULT_WALLET_SETTINGS: WalletSettings = {
  biometricEnabled: false,
  autoLockTimeout: 5,
  defaultNetwork: 'stardust',
};
