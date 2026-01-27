/**
 * IPFS 存储管理服务
 * 处理 IPFS 内容固定、取消固定、账户充值等
 * 
 * 创建日期: 2026-01-25
 */

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/lib/api';
import { signAndSend, getCurrentSignerAddress } from '@/lib/signer';

/**
 * 签名状态回调
 */
export type StatusCallback = (status: string) => void;

/**
 * PIN 层级
 */
export enum PinTier {
  Critical = 'Critical',   // 关键内容，最高优先级
  Standard = 'Standard',   // 标准内容
  Temporary = 'Temporary', // 临时内容，可能被清理
}

/**
 * PIN 状态
 */
export enum PinStatus {
  Pending = 'Pending',     // 等待固定
  Pinned = 'Pinned',       // 已固定
  Failed = 'Failed',       // 固定失败
  Unpinned = 'Unpinned',   // 已取消固定
}

/**
 * 已固定内容信息
 */
export interface PinnedContent {
  cidHash: string;
  cid: string;
  subjectType: string;
  subjectId: string;
  tier: PinTier;
  status: PinStatus;
  pinnedAt: number;
  size?: number;
  replicas?: number;
}

/**
 * 主题账户信息
 */
export interface SubjectAccount {
  subjectType: string;
  subjectId: string;
  balance: bigint;
  totalPinned: number;
  totalSize: bigint;
}

/**
 * IPFS 存储管理服务类
 */
export class IpfsStorageService {
  /**
   * 获取 API 实例
   */
  private getApi(): ApiPromise {
    try {
      return getApi();
    } catch (error) {
      throw new Error('API not initialized. Please initialize API first.');
    }
  }

  // ===== 用户接口 =====

  /**
   * 为主题账户充值
   * @param subjectType 主题类型（如 'bazi', 'chat', 'matchmaking'）
   * @param subjectId 主题ID
   * @param amount 充值金额
   * @param onStatusChange 状态回调
   */
  async fundSubjectAccount(
    subjectType: string,
    subjectId: string,
    amount: bigint,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备充值...');

    if (!api.tx.stardustIpfs?.fundSubjectAccount) {
      throw new Error('IPFS pallet not available');
    }
    const tx = api.tx.stardustIpfs.fundSubjectAccount(
      subjectType,
      subjectId,
      amount.toString()
    );

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 请求固定内容
   * @param subjectType 主题类型
   * @param subjectId 主题ID
   * @param cid IPFS CID
   * @param tier PIN 层级
   * @param onStatusChange 状态回调
   */
  async requestPin(
    subjectType: string,
    subjectId: string,
    cid: string,
    tier: PinTier,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备固定内容...');

    const tierEnum = tier === PinTier.Critical
      ? { Critical: null }
      : tier === PinTier.Standard
        ? { Standard: null }
        : { Temporary: null };

    if (!api.tx.stardustIpfs?.requestPinForSubject) {
      throw new Error('IPFS pallet not available');
    }
    const tx = api.tx.stardustIpfs.requestPinForSubject(
      subjectType,
      subjectId,
      cid,
      tierEnum
    );

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 请求取消固定
   * @param cidHash CID 哈希
   * @param onStatusChange 状态回调
   */
  async requestUnpin(
    cidHash: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备取消固定...');

    if (!api.tx.stardustIpfs?.requestUnpin) {
      throw new Error('IPFS pallet not available');
    }
    const tx = api.tx.stardustIpfs.requestUnpin(cidHash);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 手动触发计费
   * @param subjectType 主题类型
   * @param subjectId 主题ID
   * @param onStatusChange 状态回调
   */
  async chargeDue(
    subjectType: string,
    subjectId: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备计费...');

    if (!api.tx.stardustIpfs?.chargeDue) {
      throw new Error('IPFS pallet not available');
    }
    const tx = api.tx.stardustIpfs.chargeDue(subjectType, subjectId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  // ===== 查询方法 =====

  /**
   * 获取主题账户信息
   * @param subjectType 主题类型
   * @param subjectId 主题ID
   * @returns 账户信息
   */
  async getSubjectAccount(
    subjectType: string,
    subjectId: string
  ): Promise<SubjectAccount | null> {
    const api = this.getApi();

    try {
      if (!api.query.stardustIpfs?.subjectAccounts) {
        return null;
      }
      const account = await api.query.stardustIpfs.subjectAccounts([subjectType, subjectId]);

      if (account.isEmpty) {
        return null;
      }

      const data = account.toJSON() as any;

      return {
        subjectType,
        subjectId,
        balance: BigInt(data.balance || 0),
        totalPinned: data.totalPinned || 0,
        totalSize: BigInt(data.totalSize || 0),
      };
    } catch (error) {
      console.error('[IpfsStorageService] Get account error:', error);
      return null;
    }
  }

  /**
   * 获取用户的已固定内容列表
   * @param account 用户地址
   * @returns 已固定内容列表
   */
  async getUserPinnedContents(account: string): Promise<PinnedContent[]> {
    const api = this.getApi();

    try {
      if (!api.query.stardustIpfs?.pinSubjectOf) {
        return [];
      }
      const entries = await api.query.stardustIpfs.pinSubjectOf.entries();
      const contents: PinnedContent[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as any;
        
        // 检查是否属于该用户（通过 subjectId 匹配）
        if (data && data.owner === account) {
          contents.push({
            cidHash: key.args[0]?.toString() || '',
            cid: data.cid || '',
            subjectType: data.subjectType || '',
            subjectId: data.subjectId || '',
            tier: this.parsePinTier(data.tier),
            status: this.parsePinStatus(data.status),
            pinnedAt: data.pinnedAt || 0,
            size: data.size,
            replicas: data.replicas,
          });
        }
      }

      // 按固定时间倒序排序
      contents.sort((a, b) => b.pinnedAt - a.pinnedAt);

      return contents;
    } catch (error) {
      console.error('[IpfsStorageService] Get pinned contents error:', error);
      return [];
    }
  }

  /**
   * 获取单个 CID 的固定信息
   * @param cidHash CID 哈希
   * @returns 固定信息
   */
  async getPinInfo(cidHash: string): Promise<PinnedContent | null> {
    const api = this.getApi();

    try {
      if (!api.query.stardustIpfs?.pinSubjectOf) {
        return null;
      }
      const info = await api.query.stardustIpfs.pinSubjectOf(cidHash);

      if (info.isEmpty) {
        return null;
      }

      const data = info.toJSON() as any;

      return {
        cidHash,
        cid: data.cid || '',
        subjectType: data.subjectType || '',
        subjectId: data.subjectId || '',
        tier: this.parsePinTier(data.tier),
        status: this.parsePinStatus(data.status),
        pinnedAt: data.pinnedAt || 0,
        size: data.size,
        replicas: data.replicas,
      };
    } catch (error) {
      console.error('[IpfsStorageService] Get pin info error:', error);
      return null;
    }
  }

  /**
   * 获取计费参数
   * @returns 计费参数
   */
  async getBillingParams(): Promise<{
    pricePerBytePerBlock: bigint;
    minBalance: bigint;
    billingPeriod: number;
  } | null> {
    const api = this.getApi();

    try {
      if (!api.query.stardustIpfs?.billingParams) {
        return null;
      }
      const params = await api.query.stardustIpfs.billingParams();
      const data = params.toJSON() as any;

      return {
        pricePerBytePerBlock: BigInt(data.pricePerBytePerBlock || 0),
        minBalance: BigInt(data.minBalance || 0),
        billingPeriod: data.billingPeriod || 0,
      };
    } catch (error) {
      console.error('[IpfsStorageService] Get billing params error:', error);
      return null;
    }
  }

  /**
   * 估算存储费用
   * @param sizeBytes 文件大小（字节）
   * @param durationBlocks 存储时长（区块数）
   * @returns 预估费用
   */
  async estimateStorageCost(
    sizeBytes: number,
    durationBlocks: number
  ): Promise<bigint> {
    const params = await this.getBillingParams();
    if (!params) {
      return BigInt(0);
    }

    return params.pricePerBytePerBlock * BigInt(sizeBytes) * BigInt(durationBlocks);
  }

  // ===== 辅助方法 =====

  private parsePinTier(tier: any): PinTier {
    if (typeof tier === 'string') {
      return tier as PinTier;
    }
    if (tier && typeof tier === 'object') {
      const key = Object.keys(tier)[0];
      return key as PinTier;
    }
    return PinTier.Standard;
  }

  private parsePinStatus(status: any): PinStatus {
    if (typeof status === 'string') {
      return status as PinStatus;
    }
    if (status && typeof status === 'object') {
      const key = Object.keys(status)[0];
      return key as PinStatus;
    }
    return PinStatus.Pending;
  }

  /**
   * 格式化存储大小
   * @param bytes 字节数
   * @returns 格式化后的字符串
   */
  static formatSize(bytes: number | bigint): string {
    const b = Number(bytes);
    if (b < 1024) return `${b} B`;
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(2)} KB`;
    if (b < 1024 * 1024 * 1024) return `${(b / (1024 * 1024)).toFixed(2)} MB`;
    return `${(b / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }
}

// 导出单例
export const ipfsStorageService = new IpfsStorageService();
