/**
 * 星尘玄鉴 - IndexedDB 安全存储（Web 版本）
 * 
 * 用于替代 localStorage 的安全存储实现
 * 使用 IndexedDB 存储，比 localStorage 更安全
 */

// IndexedDB 数据库配置
const DB_NAME = 'stardust_storage';
const DB_VERSION = 1;
const STORE_NAME = 'key_value_store';

interface Database {
  db: IDBDatabase | null;
  initPromise: Promise<IDBDatabase> | null;
}

const dbState: Database = {
  db: null,
  initPromise: null,
};

/**
 * 初始化 IndexedDB
 */
function initDB(): Promise<IDBDatabase> {
  if (dbState.db) {
    return Promise.resolve(dbState.db);
  }

  if (dbState.initPromise) {
    return dbState.initPromise;
  }

  dbState.initPromise = new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => {
      dbState.initPromise = null;
      reject(new Error('无法打开 IndexedDB 数据库'));
    };

    request.onsuccess = () => {
      dbState.db = request.result;
      dbState.initPromise = null;
      resolve(request.result);
    };

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      
      // 创建对象存储
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };
  });

  return dbState.initPromise;
}

/**
 * 获取对象存储
 */
async function getStore(mode: IDBTransactionMode = 'readonly'): Promise<IDBObjectStore> {
  const db = await initDB();
  const transaction = db.transaction([STORE_NAME], mode);
  return transaction.objectStore(STORE_NAME);
}

/**
 * 存储接口
 */
export interface SecureStorageInterface {
  getItem(key: string): Promise<string | null>;
  setItem(key: string, value: string): Promise<void>;
  removeItem(key: string): Promise<void>;
  multiRemove(keys: string[]): Promise<void>;
}

/**
 * IndexedDB 存储实现
 */
class IndexedDBStorage implements SecureStorageInterface {
  async getItem(key: string): Promise<string | null> {
    try {
      const store = await getStore('readonly');
      return new Promise((resolve, reject) => {
        const request = store.get(key);
        request.onsuccess = () => {
          resolve(request.result || null);
        };
        request.onerror = () => {
          reject(new Error(`读取 ${key} 失败`));
        };
      });
    } catch (error) {
      console.error('[IndexedDB] Get item error:', error);
      return null;
    }
  }

  async setItem(key: string, value: string): Promise<void> {
    try {
      const store = await getStore('readwrite');
      return new Promise((resolve, reject) => {
        const request = store.put(value, key);
        request.onsuccess = () => resolve();
        request.onerror = () => reject(new Error(`保存 ${key} 失败`));
      });
    } catch (error) {
      console.error('[IndexedDB] Set item error:', error);
      throw new Error(`保存 ${key} 失败`);
    }
  }

  async removeItem(key: string): Promise<void> {
    try {
      const store = await getStore('readwrite');
      return new Promise((resolve, reject) => {
        const request = store.delete(key);
        request.onsuccess = () => resolve();
        request.onerror = () => reject(new Error(`删除 ${key} 失败`));
      });
    } catch (error) {
      console.error('[IndexedDB] Remove item error:', error);
      throw new Error(`删除 ${key} 失败`);
    }
  }

  async multiRemove(keys: string[]): Promise<void> {
    try {
      const store = await getStore('readwrite');
      return Promise.all(keys.map(key => {
        return new Promise<void>((resolve, reject) => {
          const request = store.delete(key);
          request.onsuccess = () => resolve();
          request.onerror = () => reject(new Error(`删除 ${key} 失败`));
        });
      })).then(() => {});
    } catch (error) {
      console.error('[IndexedDB] Multi remove error:', error);
      throw new Error('批量删除失败');
    }
  }
}

/**
 * 检查 IndexedDB 是否可用
 */
function isIndexedDBAvailable(): boolean {
  return typeof indexedDB !== 'undefined';
}

/**
 * 创建存储实例
 */
export function createSecureStorage(): SecureStorageInterface {
  if (isIndexedDBAvailable()) {
    return new IndexedDBStorage();
  } else {
    // 降级到 localStorage（不推荐，但作为后备）
    console.warn('[SecureStorage] IndexedDB not available, falling back to localStorage (insecure)');
    return {
      async getItem(key: string): Promise<string | null> {
        try {
          return localStorage.getItem(key);
        } catch {
          return null;
        }
      },
      async setItem(key: string, value: string): Promise<void> {
        try {
          localStorage.setItem(key, value);
        } catch (error) {
          throw new Error(`保存 ${key} 失败`);
        }
      },
      async removeItem(key: string): Promise<void> {
        try {
          localStorage.removeItem(key);
        } catch (error) {
          throw new Error(`删除 ${key} 失败`);
        }
      },
      async multiRemove(keys: string[]): Promise<void> {
        keys.forEach(key => {
          try {
            localStorage.removeItem(key);
          } catch {
            // 忽略错误
          }
        });
      },
    };
  }
}

/**
 * 默认存储实例（Web 平台）
 */
export const secureStorage: SecureStorageInterface = createSecureStorage();

