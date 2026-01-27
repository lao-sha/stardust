/**
 * 星尘玄鉴 - 密钥存储（Web 版本）
 * 
 * 安全实现：
 * - 使用 IndexedDB 存储（替代不安全的 localStorage）
 * - 使用 Web Crypto API 进行 AES-256-GCM 加密
 * - 使用 PBKDF2 进行密钥派生（310,000 iterations，符合 OWASP 2023）
 * - 实现数据完整性校验（HMAC-SHA256）
 * 
 * @see src/lib/secure-storage.web.ts 核心实现
 */

// 从安全存储模块导出所有功能
export {
  initializeCrypto,
  storeEncryptedMnemonic,
  retrieveEncryptedMnemonic,
  verifyPassword,
  loadAllKeystores,
  getCurrentAddress,
  setCurrentAddress,
  getAlias,
  setAlias,
  hasWallet,
  getStoredAddress,
  deleteWalletByAddress,
  deleteWallet,
  deleteAllWallets,
  changePassword,
  exportWalletBackup,
  importWalletBackup,
  type SecureKeystore as LocalKeystore,
} from './secure-storage.web';

// ==================== 助记词生成（使用 @polkadot/util-crypto）====================

import { mnemonicGenerate, mnemonicValidate } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';

/**
 * 生成 BIP39 助记词
 */
export function generateMnemonic(): string {
  return mnemonicGenerate(12);
}

/**
 * 验证助记词
 */
export function validateMnemonic(mnemonic: string): boolean {
  return mnemonicValidate(mnemonic);
}

/**
 * 从助记词创建密钥对
 */
export function createKeyPairFromMnemonic(mnemonic: string): { address: string } {
  const keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const pair = keyring.addFromMnemonic(mnemonic);
  return { address: pair.address };
}
