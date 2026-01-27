/**
 * IPFS 服务
 * 支持多端点故障转移和多网关下载
 */

import AsyncStorage from '@react-native-async-storage/async-storage';

/**
 * IPFS 配置接口
 */
interface IpfsConfig {
  /** 上传 API 端点（支持多个备选） */
  apiEndpoints: string[];
  /** 下载网关（支持多个备选） */
  gateways: string[];
  /** 请求超时（毫秒） */
  timeout: number;
  /** 重试次数 */
  retries: number;
}

/**
 * 默认配置 - 支持多环境
 */
const DEFAULT_CONFIG: IpfsConfig = {
  apiEndpoints: [
    process.env.EXPO_PUBLIC_IPFS_API || '',
    'https://api.pinata.cloud/pinning/pinFileToIPFS',
    'https://api.web3.storage/upload',
  ].filter(Boolean),
  gateways: [
    process.env.EXPO_PUBLIC_IPFS_GATEWAY || '',
    'https://gateway.pinata.cloud/ipfs/',
    'https://w3s.link/ipfs/',
    'https://cloudflare-ipfs.com/ipfs/',
    'https://ipfs.io/ipfs/',
  ].filter(Boolean),
  timeout: 30000,
  retries: 3,
};

let config: IpfsConfig = { ...DEFAULT_CONFIG };

/**
 * 初始化 IPFS 配置
 */
export async function initIpfsService(
  customConfig?: Partial<IpfsConfig>
): Promise<void> {
  // 从本地存储加载配置
  try {
    const stored = await AsyncStorage.getItem('ipfs_config');
    if (stored) {
      config = { ...DEFAULT_CONFIG, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.warn('Failed to load IPFS config from storage:', error);
  }

  // 合并自定义配置
  if (customConfig) {
    config = { ...config, ...customConfig };
  }
}

/**
 * 更新 IPFS 配置
 */
export async function updateIpfsConfig(
  newConfig: Partial<IpfsConfig>
): Promise<void> {
  config = { ...config, ...newConfig };
  await AsyncStorage.setItem('ipfs_config', JSON.stringify(config));
}

/**
 * 获取当前配置
 */
export function getIpfsConfig(): IpfsConfig {
  return { ...config };
}

/**
 * 上传加密内容到 IPFS（带重试和故障转移）
 */
export async function uploadToIpfs(content: Uint8Array): Promise<string> {
  const errors: Error[] = [];

  for (const endpoint of config.apiEndpoints) {
    for (let attempt = 0; attempt < config.retries; attempt++) {
      try {
        const cid = await uploadToEndpoint(endpoint, content);
        return cid;
      } catch (error) {
        errors.push(error as Error);
        // 短暂延迟后重试
        await delay(1000 * (attempt + 1));
      }
    }
  }

  throw new AggregateError(errors, 'All IPFS upload attempts failed');
}

/**
 * 上传到指定端点
 */
async function uploadToEndpoint(
  endpoint: string,
  content: Uint8Array
): Promise<string> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), config.timeout);

  try {
    // 检测端点类型并使用对应的上传方式
    if (endpoint.includes('pinata.cloud')) {
      return await uploadToPinata(endpoint, content, controller.signal);
    } else if (endpoint.includes('web3.storage')) {
      return await uploadToWeb3Storage(endpoint, content, controller.signal);
    } else {
      return await uploadToStandardIpfs(endpoint, content, controller.signal);
    }
  } finally {
    clearTimeout(timeoutId);
  }
}

/**
 * 标准 IPFS API 上传
 */
async function uploadToStandardIpfs(
  endpoint: string,
  content: Uint8Array,
  signal: AbortSignal
): Promise<string> {
  const formData = new FormData();
  formData.append(
    'file',
    new Blob([content], { type: 'application/octet-stream' })
  );

  const response = await fetch(`${endpoint}/add?pin=true`, {
    method: 'POST',
    body: formData,
    signal,
  });

  if (!response.ok) {
    throw new Error(`IPFS upload failed: ${response.status}`);
  }

  const result = await response.json();
  return result.Hash;
}

/**
 * Pinata 上传
 */
async function uploadToPinata(
  endpoint: string,
  content: Uint8Array,
  signal: AbortSignal
): Promise<string> {
  const apiKey = process.env.EXPO_PUBLIC_PINATA_API_KEY;
  const apiSecret = process.env.EXPO_PUBLIC_PINATA_API_SECRET;

  if (!apiKey || !apiSecret) {
    throw new Error('Pinata API credentials not configured');
  }

  const formData = new FormData();
  formData.append(
    'file',
    new Blob([content], { type: 'application/octet-stream' })
  );

  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      pinata_api_key: apiKey,
      pinata_secret_api_key: apiSecret,
    },
    body: formData,
    signal,
  });

  if (!response.ok) {
    throw new Error(`Pinata upload failed: ${response.status}`);
  }

  const result = await response.json();
  return result.IpfsHash;
}

/**
 * Web3.Storage 上传
 */
async function uploadToWeb3Storage(
  endpoint: string,
  content: Uint8Array,
  signal: AbortSignal
): Promise<string> {
  const token = process.env.EXPO_PUBLIC_WEB3_STORAGE_TOKEN;

  if (!token) {
    throw new Error('Web3.Storage token not configured');
  }

  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': 'application/octet-stream',
    },
    body: content,
    signal,
  });

  if (!response.ok) {
    throw new Error(`Web3.Storage upload failed: ${response.status}`);
  }

  const result = await response.json();
  return result.cid;
}

/**
 * 从 IPFS 下载内容（带故障转移）
 */
export async function downloadFromIpfs(cid: string): Promise<Uint8Array> {
  const errors: Error[] = [];

  for (const gateway of config.gateways) {
    try {
      return await downloadFromGateway(gateway, cid);
    } catch (error) {
      errors.push(error as Error);
    }
  }

  throw new AggregateError(errors, 'All IPFS download attempts failed');
}

/**
 * 从指定网关下载
 */
async function downloadFromGateway(
  gateway: string,
  cid: string
): Promise<Uint8Array> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), config.timeout);

  try {
    const response = await fetch(`${gateway}${cid}`, {
      signal: controller.signal,
    });

    if (!response.ok) {
      throw new Error(`IPFS download failed: ${response.status}`);
    }

    const buffer = await response.arrayBuffer();
    return new Uint8Array(buffer);
  } finally {
    clearTimeout(timeoutId);
  }
}

/**
 * 检查 CID 是否可访问
 */
export async function checkCidAvailability(cid: string): Promise<boolean> {
  for (const gateway of config.gateways) {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000);

      const response = await fetch(`${gateway}${cid}`, {
        method: 'HEAD',
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (response.ok) {
        return true;
      }
    } catch {
      // 继续尝试下一个网关
    }
  }
  return false;
}

/**
 * 获取 CID 的完整 URL（使用第一个可用网关）
 */
export function getCidUrl(cid: string): string {
  const gateway = config.gateways[0] || 'https://ipfs.io/ipfs/';
  return `${gateway}${cid}`;
}

/**
 * 上传图片文件到 IPFS
 * @param imageUri 图片的本地 URI
 * @returns IPFS CID
 */
export async function uploadImageToIpfs(imageUri: string): Promise<string> {
  try {
    // 读取图片文件
    const response = await fetch(imageUri);
    const blob = await response.blob();
    
    // 转换为 Uint8Array
    const arrayBuffer = await blob.arrayBuffer();
    const content = new Uint8Array(arrayBuffer);
    
    // 上传到 IPFS
    const cid = await uploadToIpfs(content);
    return cid;
  } catch (error) {
    console.error('Upload image to IPFS failed:', error);
    throw new Error(`图片上传失败: ${(error as Error).message}`);
  }
}

/**
 * 延迟函数
 */
function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
