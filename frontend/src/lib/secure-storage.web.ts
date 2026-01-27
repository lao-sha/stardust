/**
 * 星尘玄鉴 - Web 安全存储服务
 * 
 * 安全架构：
 * 1. 使用 IndexedDB 替代 localStorage（不可被简单 XSS 读取）
 * 2. 使用 Web Crypto API 进行 AES-256-GCM 加密
 * 3. 使用 PBKDF2 进行密钥派生（310,000 iterations，符合 OWASP 2023 建议）
 * 4. 实现数据完整性校验（HMAC-SHA256）
 * 5. 敏感数据内存保护（使用后立即清零）
 */

import { CryptoError, WalletError, AuthenticationError } from './errors';

// ==================== 常量定义 ====================

const DB_NAME = 'stardust_secure_vault';
const DB_VERSION = 1;
const STORE_NAME = 'encrypted_data';

/**
 * 安全参数配置（符合 OWASP 2023 标准）
 */
const SECURITY_CONFIG = {
  // PBKDF2 迭代次数（OWASP 2023 建议 SHA-256 至少 310,000）
  PBKDF2_ITERATIONS: 310000,
  // 盐长度（32 字节 = 256 位）
  SALT_LENGTH: 32,
  // IV 长度（12 字节，AES-GCM 推荐）
  IV_LENGTH: 12,
  // 密钥长度（32 字节 = 256 位）
  KEY_LENGTH: 32,
  // 认证标签长度（128 位）
  TAG_LENGTH: 128,
  // 数据版本（用于未来迁移）
  DATA_VERSION: 1,
} as const;

/**
 * 存储键名
 */
const STORAGE_KEYS = {
  KEYSTORES: 'keystores',
  CURRENT_ADDRESS: 'current_address',
  ALIASES: 'aliases',
  MASTER_KEY_CHECK: 'master_key_check',
} as const;

// ==================== 类型定义 ====================

/**
 * 加密数据包结构
 */
interface EncryptedPackage {
  /** 数据版本 */
  version: number;
  /** 加密后的数据（Base64） */
  ciphertext: string;
  /** 初始化向量（Base64） */
  iv: string;
  /** 盐值（Base64） */
  salt: string;
  /** 创建时间戳 */
  createdAt: number;
  /** 数据完整性校验（HMAC） */
  hmac: string;
}

/**
 * 密钥存储结构
 */
export interface SecureKeystore {
  address: string;
  encryptedMnemonic: EncryptedPackage;
  createdAt: number;
}

/**
 * 内存中的解密密钥（使用后需清零）
 */
interface DerivedKeys {
  encryptionKey: CryptoKey;
  hmacKey: CryptoKey;
}

// ==================== IndexedDB 操作 ====================

/**
 * 打开 IndexedDB 数据库
 */
function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => {
      reject(new WalletError('无法打开安全存储数据库'));
    };

    request.onsuccess = () => {
      resolve(request.result);
    };

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      
      // 创建对象存储
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: 'key' });
      }
    };
  });
}

/**
 * 从 IndexedDB 读取数据
 */
async function dbGet<T>(key: string): Promise<T | null> {
  const db = await openDatabase();
  
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readonly');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.get(key);

    request.onerror = () => {
      db.close();
      reject(new WalletError('读取数据失败'));
    };

    request.onsuccess = () => {
      db.close();
      resolve(request.result?.value ?? null);
    };
  });
}

/**
 * 向 IndexedDB 写入数据
 */
async function dbSet<T>(key: string, value: T): Promise<void> {
  const db = await openDatabase();
  
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.put({ key, value });

    request.onerror = () => {
      db.close();
      reject(new WalletError('写入数据失败'));
    };

    request.onsuccess = () => {
      db.close();
      resolve();
    };
  });
}

/**
 * 从 IndexedDB 删除数据
 */
async function dbDelete(key: string): Promise<void> {
  const db = await openDatabase();
  
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.delete(key);

    request.onerror = () => {
      db.close();
      reject(new WalletError('删除数据失败'));
    };

    request.onsuccess = () => {
      db.close();
      resolve();
    };
  });
}

/**
 * 清空 IndexedDB
 */
async function dbClear(): Promise<void> {
  const db = await openDatabase();
  
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.clear();

    request.onerror = () => {
      db.close();
      reject(new WalletError('清空数据失败'));
    };

    request.onsuccess = () => {
      db.close();
      resolve();
    };
  });
}

// ==================== 加密工具函数 ====================

/**
 * 生成加密安全的随机字节
 */
function generateRandomBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  crypto.getRandomValues(bytes);
  return bytes;
}

/**
 * Uint8Array 转 Base64
 */
function uint8ArrayToBase64(bytes: Uint8Array): string {
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]!);
  }
  return btoa(binary);
}

/**
 * Base64 转 Uint8Array
 */
function base64ToUint8Array(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/**
 * 字符串转 Uint8Array
 */
function stringToUint8Array(str: string): Uint8Array {
  return new TextEncoder().encode(str);
}

/**
 * Uint8Array 转字符串
 */
function uint8ArrayToString(bytes: Uint8Array): string {
  return new TextDecoder().decode(bytes);
}

/**
 * 安全清零 Uint8Array（防止内存残留）
 */
function secureZero(array: Uint8Array): void {
  crypto.getRandomValues(array); // 先用随机数覆盖
  array.fill(0); // 再填充零
}

/**
 * 使用 PBKDF2 派生密钥
 */
async function deriveKeys(
  password: string,
  salt: Uint8Array
): Promise<DerivedKeys> {
  // 导入密码作为原始密钥材料
  const passwordBytes = stringToUint8Array(password);
  const passwordKey = await crypto.subtle.importKey(
    'raw',
    passwordBytes.buffer as ArrayBuffer,
    'PBKDF2',
    false,
    ['deriveBits']
  );

  // 派生 64 字节（32 字节加密密钥 + 32 字节 HMAC 密钥）
  const derivedBits = await crypto.subtle.deriveBits(
    {
      name: 'PBKDF2',
      salt: salt.buffer as ArrayBuffer,
      iterations: SECURITY_CONFIG.PBKDF2_ITERATIONS,
      hash: 'SHA-256',
    },
    passwordKey,
    512 // 64 字节 = 512 位
  );

  const derivedArray = new Uint8Array(derivedBits);
  const encryptionKeyBytes = derivedArray.slice(0, 32);
  const hmacKeyBytes = derivedArray.slice(32, 64);

  // 导入 AES-GCM 加密密钥
  const encryptionKey = await crypto.subtle.importKey(
    'raw',
    encryptionKeyBytes.buffer as ArrayBuffer,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt']
  );

  // 导入 HMAC 密钥
  const hmacKey = await crypto.subtle.importKey(
    'raw',
    hmacKeyBytes.buffer as ArrayBuffer,
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign', 'verify']
  );

  // 清零临时数组
  secureZero(encryptionKeyBytes);
  secureZero(hmacKeyBytes);
  secureZero(derivedArray);

  return { encryptionKey, hmacKey };
}

/**
 * 计算 HMAC
 */
async function computeHmac(
  hmacKey: CryptoKey,
  data: Uint8Array
): Promise<string> {
  const signature = await crypto.subtle.sign('HMAC', hmacKey, data.buffer as ArrayBuffer);
  return uint8ArrayToBase64(new Uint8Array(signature));
}

/**
 * 验证 HMAC
 */
async function verifyHmac(
  hmacKey: CryptoKey,
  data: Uint8Array,
  expectedHmac: string
): Promise<boolean> {
  const expectedBytes = base64ToUint8Array(expectedHmac);
  return await crypto.subtle.verify('HMAC', hmacKey, expectedBytes.buffer as ArrayBuffer, data.buffer as ArrayBuffer);
}

// ==================== 核心加密/解密 ====================

/**
 * 加密数据
 */
async function encryptData(
  plaintext: string,
  password: string
): Promise<EncryptedPackage> {
  // 生成随机盐和 IV
  const salt = generateRandomBytes(SECURITY_CONFIG.SALT_LENGTH);
  const iv = generateRandomBytes(SECURITY_CONFIG.IV_LENGTH);

  // 派生密钥
  const { encryptionKey, hmacKey } = await deriveKeys(password, salt);

  // 加密数据
  const plaintextBytes = stringToUint8Array(plaintext);
  const ciphertext = await crypto.subtle.encrypt(
    {
      name: 'AES-GCM',
      iv: iv.buffer as ArrayBuffer,
      tagLength: SECURITY_CONFIG.TAG_LENGTH,
    },
    encryptionKey,
    plaintextBytes.buffer as ArrayBuffer
  );

  const ciphertextBase64 = uint8ArrayToBase64(new Uint8Array(ciphertext));
  const ivBase64 = uint8ArrayToBase64(iv);
  const saltBase64 = uint8ArrayToBase64(salt);

  // 计算 HMAC（对 version + ciphertext + iv + salt 计算）
  const hmacData = stringToUint8Array(
    `${SECURITY_CONFIG.DATA_VERSION}:${ciphertextBase64}:${ivBase64}:${saltBase64}`
  );
  const hmac = await computeHmac(hmacKey, hmacData);

  // 清零敏感数据
  secureZero(plaintextBytes);
  secureZero(salt);
  secureZero(iv);

  return {
    version: SECURITY_CONFIG.DATA_VERSION,
    ciphertext: ciphertextBase64,
    iv: ivBase64,
    salt: saltBase64,
    createdAt: Date.now(),
    hmac,
  };
}

/**
 * 解密数据
 */
async function decryptData(
  encryptedPackage: EncryptedPackage,
  password: string
): Promise<string> {
  const { version, ciphertext, iv, salt, hmac } = encryptedPackage;

  // 版本检查
  if (version !== SECURITY_CONFIG.DATA_VERSION) {
    throw new CryptoError('数据版本不兼容，请重新导入钱包');
  }

  // 解码 Base64
  const saltBytes = base64ToUint8Array(salt);
  const ivBytes = base64ToUint8Array(iv);
  const ciphertextBytes = base64ToUint8Array(ciphertext);

  // 派生密钥
  const { encryptionKey, hmacKey } = await deriveKeys(password, saltBytes);

  // 验证 HMAC（防止数据篡改）
  const hmacData = stringToUint8Array(
    `${version}:${ciphertext}:${iv}:${salt}`
  );
  const isValid = await verifyHmac(hmacKey, hmacData, hmac);
  
  if (!isValid) {
    throw new CryptoError('数据完整性校验失败，可能已被篡改');
  }

  try {
    // 解密数据
    const plaintext = await crypto.subtle.decrypt(
      {
        name: 'AES-GCM',
        iv: ivBytes.buffer as ArrayBuffer,
        tagLength: SECURITY_CONFIG.TAG_LENGTH,
      },
      encryptionKey,
      ciphertextBytes.buffer as ArrayBuffer
    );

    const result = uint8ArrayToString(new Uint8Array(plaintext));

    // 清零敏感数据
    secureZero(saltBytes);
    secureZero(ivBytes);
    secureZero(ciphertextBytes);

    return result;
  } catch (error) {
    // AES-GCM 解密失败通常意味着密码错误
    throw new AuthenticationError('密码错误');
  }
}

// ==================== 公开 API ====================

/**
 * 初始化加密系统
 */
export async function initializeCrypto(): Promise<void> {
  // 检查 Web Crypto API 可用性
  if (!crypto?.subtle) {
    throw new CryptoError('当前浏览器不支持 Web Crypto API，请使用现代浏览器');
  }

  // 检查 IndexedDB 可用性
  if (!indexedDB) {
    throw new CryptoError('当前浏览器不支持 IndexedDB，请使用现代浏览器');
  }

  // 初始化数据库
  await openDatabase();
  
  console.log('[SecureStorage] Initialized with IndexedDB + Web Crypto API');
}

/**
 * 存储加密的助记词
 */
export async function storeEncryptedMnemonic(
  mnemonic: string,
  password: string,
  address: string
): Promise<void> {
  // 验证密码强度
  if (!password || password.length < 8) {
    throw new WalletError('密码至少需要 8 位');
  }

  // 加密助记词
  const encryptedMnemonic = await encryptData(mnemonic, password);

  // 创建密钥存储
  const keystore: SecureKeystore = {
    address,
    encryptedMnemonic,
    createdAt: Date.now(),
  };

  // 加载现有密钥存储
  const keystores = await loadAllKeystores();
  
  // 更新或添加
  const existingIndex = keystores.findIndex(ks => ks.address === address);
  if (existingIndex >= 0) {
    keystores[existingIndex] = keystore;
  } else {
    keystores.push(keystore);
  }

  // 保存到 IndexedDB
  await dbSet(STORAGE_KEYS.KEYSTORES, keystores);
  await setCurrentAddress(address);
  await setAlias(address, `钱包 ${address.slice(0, 6)}`);

  // 存储密码验证标记（用于快速验证密码是否正确）
  const checkData = await encryptData('STARDUST_CHECK', password);
  await dbSet(STORAGE_KEYS.MASTER_KEY_CHECK, checkData);

  console.log('[SecureStorage] Mnemonic stored securely');
}

/**
 * 检索并解密助记词
 */
export async function retrieveEncryptedMnemonic(
  password: string,
  address?: string
): Promise<string> {
  const keystores = await loadAllKeystores();
  const targetAddress = address || await getCurrentAddress();

  if (!targetAddress) {
    throw new WalletError('未找到钱包');
  }

  const keystore = keystores.find(ks => ks.address === targetAddress);
  if (!keystore) {
    throw new WalletError('未找到钱包数据');
  }

  return await decryptData(keystore.encryptedMnemonic, password);
}

/**
 * 验证密码是否正确（不解密助记词）
 */
export async function verifyPassword(password: string): Promise<boolean> {
  try {
    const checkData = await dbGet<EncryptedPackage>(STORAGE_KEYS.MASTER_KEY_CHECK);
    if (!checkData) return false;

    const decrypted = await decryptData(checkData, password);
    return decrypted === 'STARDUST_CHECK';
  } catch {
    return false;
  }
}

/**
 * 加载所有密钥存储
 */
export async function loadAllKeystores(): Promise<SecureKeystore[]> {
  const keystores = await dbGet<SecureKeystore[]>(STORAGE_KEYS.KEYSTORES);
  return keystores ?? [];
}

/**
 * 获取当前地址
 */
export async function getCurrentAddress(): Promise<string | null> {
  return await dbGet<string>(STORAGE_KEYS.CURRENT_ADDRESS);
}

/**
 * 设置当前地址
 */
export async function setCurrentAddress(address: string): Promise<void> {
  await dbSet(STORAGE_KEYS.CURRENT_ADDRESS, address);
}

/**
 * 获取钱包别名
 */
export async function getAlias(address: string): Promise<string | null> {
  const aliases = await dbGet<Record<string, string>>(STORAGE_KEYS.ALIASES);
  return aliases?.[address] ?? null;
}

/**
 * 设置钱包别名
 */
export async function setAlias(address: string, alias: string): Promise<void> {
  const aliases = await dbGet<Record<string, string>>(STORAGE_KEYS.ALIASES) ?? {};
  aliases[address] = alias;
  await dbSet(STORAGE_KEYS.ALIASES, aliases);
}

/**
 * 检查是否有钱包
 */
export async function hasWallet(): Promise<boolean> {
  const keystores = await loadAllKeystores();
  return keystores.length > 0;
}

/**
 * 获取存储的地址
 */
export async function getStoredAddress(): Promise<string | null> {
  const currentAddr = await getCurrentAddress();
  if (currentAddr) return currentAddr;

  const keystores = await loadAllKeystores();
  if (keystores.length > 0 && keystores[0]) {
    await setCurrentAddress(keystores[0].address);
    return keystores[0].address;
  }
  return null;
}

/**
 * 删除指定地址的钱包
 */
export async function deleteWalletByAddress(address: string): Promise<void> {
  const keystores = await loadAllKeystores();
  const filtered = keystores.filter(ks => ks.address !== address);
  await dbSet(STORAGE_KEYS.KEYSTORES, filtered);

  const currentAddr = await getCurrentAddress();
  if (currentAddr === address) {
    if (filtered.length > 0 && filtered[0]) {
      await setCurrentAddress(filtered[0].address);
    } else {
      await dbDelete(STORAGE_KEYS.CURRENT_ADDRESS);
      await dbDelete(STORAGE_KEYS.MASTER_KEY_CHECK);
    }
  }
}

/**
 * 删除当前钱包
 */
export async function deleteWallet(): Promise<void> {
  const currentAddr = await getCurrentAddress();
  if (currentAddr) {
    await deleteWalletByAddress(currentAddr);
  }
}

/**
 * 删除所有钱包数据
 */
export async function deleteAllWallets(): Promise<void> {
  await dbClear();
}

/**
 * 更改密码
 */
export async function changePassword(
  oldPassword: string,
  newPassword: string
): Promise<void> {
  // 验证新密码强度
  if (!newPassword || newPassword.length < 8) {
    throw new WalletError('新密码至少需要 8 位');
  }

  // 验证旧密码
  const isValid = await verifyPassword(oldPassword);
  if (!isValid) {
    throw new AuthenticationError('当前密码错误');
  }

  // 加载所有钱包
  const keystores = await loadAllKeystores();
  const updatedKeystores: SecureKeystore[] = [];

  // 重新加密每个钱包
  for (const keystore of keystores) {
    const mnemonic = await decryptData(keystore.encryptedMnemonic, oldPassword);
    const newEncrypted = await encryptData(mnemonic, newPassword);
    
    updatedKeystores.push({
      ...keystore,
      encryptedMnemonic: newEncrypted,
    });
  }

  // 保存更新后的钱包
  await dbSet(STORAGE_KEYS.KEYSTORES, updatedKeystores);

  // 更新密码验证标记
  const checkData = await encryptData('STARDUST_CHECK', newPassword);
  await dbSet(STORAGE_KEYS.MASTER_KEY_CHECK, checkData);

  console.log('[SecureStorage] Password changed successfully');
}

/**
 * 导出钱包数据（用于备份）
 */
export async function exportWalletBackup(password: string): Promise<string> {
  const isValid = await verifyPassword(password);
  if (!isValid) {
    throw new AuthenticationError('密码错误');
  }

  const keystores = await loadAllKeystores();
  const aliases = await dbGet<Record<string, string>>(STORAGE_KEYS.ALIASES) ?? {};
  const currentAddress = await getCurrentAddress();

  const backup = {
    version: SECURITY_CONFIG.DATA_VERSION,
    exportedAt: Date.now(),
    keystores,
    aliases,
    currentAddress,
  };

  // 加密备份数据
  const encrypted = await encryptData(JSON.stringify(backup), password);
  return JSON.stringify(encrypted);
}

/**
 * 导入钱包备份
 */
export async function importWalletBackup(
  backupData: string,
  password: string
): Promise<void> {
  const encrypted: EncryptedPackage = JSON.parse(backupData);
  const decrypted = await decryptData(encrypted, password);
  const backup = JSON.parse(decrypted);

  // 版本检查
  if (backup.version !== SECURITY_CONFIG.DATA_VERSION) {
    throw new WalletError('备份版本不兼容');
  }

  // 恢复数据
  await dbSet(STORAGE_KEYS.KEYSTORES, backup.keystores);
  await dbSet(STORAGE_KEYS.ALIASES, backup.aliases);
  if (backup.currentAddress) {
    await dbSet(STORAGE_KEYS.CURRENT_ADDRESS, backup.currentAddress);
  }

  // 设置密码验证标记
  const checkData = await encryptData('STARDUST_CHECK', password);
  await dbSet(STORAGE_KEYS.MASTER_KEY_CHECK, checkData);

  console.log('[SecureStorage] Wallet backup imported successfully');
}
