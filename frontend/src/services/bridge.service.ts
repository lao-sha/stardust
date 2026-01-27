/**
 * Bridge/Swap 服务 - 处理 DUST 与 USDT 的兑换交易
 *
 * 重要变更 (2026-01-19):
 * - 链端 pallet-swap 已移除官方兑换功能，仅支持做市商兑换
 * - officialSwap 方法已弃用
 * - 新增 markSwapComplete 和 reportSwap 方法
 */

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/lib/api';
import { signAndSend, getCurrentSignerAddress } from '@/lib/signer';
import { u8aToHex, hexToU8a } from '@polkadot/util';
import { parseSwapStatus, parseNumber } from '@/types/type-guards';
import type { SwapStatus as SwapStatusType } from '@/types/substrate.types';

/** 链上兑换记录的 JSON 表示 */
interface SwapRecordJson {
  buyer?: string;
  makerId?: number;
  dustAmount?: string | number;
  usdtAmount?: string | number;
  buyerTronAddress?: string;
  makerTronAddress?: string;
  status?: string | { [key: string]: null };
  timeInfo?: {
    createdAt?: number;
    completedAt?: number;
  };
  tronTxHash?: string;
}

/** 链上市场统计的 JSON 表示 */
interface MarketStatsJson {
  otcPrice?: number;
  bridgePrice?: number;
  weightedPrice?: number;
  simpleAvgPrice?: number;
  otcVolume?: string | number;
  bridgeVolume?: string | number;
  totalVolume?: string | number;
}

/** 链上账户数据的 JSON 表示 */
interface AccountDataJson {
  data?: {
    free?: string | number;
    reserved?: string | number;
    frozen?: string | number;
  };
}

/**
 * 签名状态回调
 */
export type StatusCallback = (status: string) => void;

/**
 * 兑换类型（已简化，官方兑换已移除）
 */
export enum SwapType {
  Maker = 'Maker',  // 做市商兑换
}

/**
 * 兑换状态
 */
export enum SwapStatus {
  Pending = 'Pending',     // 待处理
  Completed = 'Completed', // 已完成
  Timeout = 'Timeout',     // 超时
  Reported = 'Reported',   // 被举报
  Refunded = 'Refunded',   // 已退款
}

/**
 * 兑换记录
 */
export interface SwapRecord {
  id: number;
  buyer: string;
  makerId: number;
  dustAmount: bigint;
  usdtAmount: bigint;
  buyerTronAddress: string;
  makerTronAddress: string;
  status: SwapStatus;
  createdAt: number;
  completedAt?: number;
  tronTxHash?: string;
}

/**
 * @deprecated 使用 SwapType 替代
 */
export enum BridgeType {
  Official = 'Official',
  Maker = 'Maker',
}

/**
 * Bridge 服务类
 */
export class BridgeService {
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

  /**
   * @deprecated 链端已移除官方兑换功能，请使用 makerSwap 方法
   *
   * 官方桥接：DUST → USDT
   * @throws 始终抛出错误，提示使用 makerSwap
   */
  async officialSwap(
    dustAmount: bigint,
    tronAddress: string,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    throw new Error(
      '官方兑换功能已停用。链端 pallet-swap 已移除此功能，请使用 makerSwap 进行做市商兑换。'
    );
  }

  /**
   * 做市商兑换：DUST → USDT
   *
   * 用户通过做市商将 DUST 兑换为 USDT。
   * 流程：
   * 1. 用户调用此方法，DUST 被锁定
   * 2. 做市商向用户的 TRON 地址转账 USDT
   * 3. 做市商调用 markSwapComplete 确认完成
   *
   * @param makerId 做市商ID
   * @param dustAmount DUST 数量（最小单位，精度 10^12）
   * @param tronAddress 用户的 TRON 地址（接收 USDT）
   * @param onStatusChange 状态变化回调
   * @returns 兑换记录ID
   */
  async makerSwap(
    makerId: number,
    dustAmount: bigint,
    tronAddress: string,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    // 验证 TRON 地址格式
    const tronRegex = /^T[A-Za-z1-9]{33}$/;
    if (!tronRegex.test(tronAddress)) {
      throw new Error('Invalid TRON address format');
    }

    onStatusChange?.('准备交易...');

    // 将 TRON 地址转换为字节数组
    const tronAddressBytes = new TextEncoder().encode(tronAddress);

    // 创建交易 - 注意: pallet 名称是 swap 而不是 bridge
    const tx = api.tx.swap.makerSwap(
      makerId,
      dustAmount.toString(),
      u8aToHex(tronAddressBytes)
    );

    onStatusChange?.('等待签名...');

    // 签名并发送交易
    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    // 从事件中提取兑换记录ID
    const swapEvent = events.find(
      ({ event }: any) =>
        event.section === 'swap' &&
        event.method === 'MakerSwapCreated'
    );

    if (!swapEvent) {
      throw new Error('未找到兑换创建事件');
    }

    // 提取记录ID（事件格式: [swapId, buyer, makerId, dustAmount, usdtAmount]）
    const swapId = swapEvent.event.data[0].toString();

    return parseInt(swapId, 10);
  }

  /**
   * 标记兑换完成（做市商调用）
   *
   * 做市商在向用户 TRON 地址转账 USDT 后调用此方法确认完成。
   *
   * @param swapId 兑换记录ID
   * @param tronTxHash TRON 网络的交易哈希（用于验证和防重放）
   * @param onStatusChange 状态变化回调
   */
  async markSwapComplete(
    swapId: number,
    tronTxHash: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    // 验证 TRON 交易哈希格式（64位十六进制）
    const tronTxHashRegex = /^[a-fA-F0-9]{64}$/;
    if (!tronTxHashRegex.test(tronTxHash)) {
      throw new Error('Invalid TRON transaction hash format');
    }

    onStatusChange?.('准备确认交易...');

    // 创建交易
    const tx = api.tx.swap.markSwapComplete(swapId, tronTxHash);

    onStatusChange?.('等待签名...');

    await signAndSend(api, tx, accountAddress, onStatusChange);

    console.log('[BridgeService] Swap marked as complete:', swapId);
  }

  /**
   * 举报兑换（用户调用）
   *
   * 如果做市商未在规定时间内完成转账，用户可以举报。
   * 举报后会触发仲裁流程。
   *
   * @param swapId 兑换记录ID
   * @param evidenceCid 证据的 IPFS CID（可选，如截图等）
   * @param onStatusChange 状态变化回调
   */
  async reportSwap(
    swapId: number,
    evidenceCid?: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备举报交易...');

    // 创建交易
    const tx = api.tx.swap.reportSwap(swapId, evidenceCid || null);

    onStatusChange?.('等待签名...');

    await signAndSend(api, tx, accountAddress, onStatusChange);

    console.log('[BridgeService] Swap reported:', swapId);
  }

  /**
   * 取消兑换（用户在做市商确认前可取消）
   * @param swapId 兑换记录ID
   * @param onStatusChange 状态变化回调
   */
  async cancelSwap(
    swapId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备取消兑换...');

    const tx = api.tx.swap.cancelSwap(swapId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);

    console.log('[BridgeService] Swap cancelled:', swapId);
  }

  /**
   * 发起兑换争议
   * @param swapId 兑换记录ID
   * @param reason 争议原因
   * @param evidenceCid 证据的 IPFS CID（可选）
   * @param onStatusChange 状态变化回调
   */
  async disputeSwap(
    swapId: number,
    reason: string,
    evidenceCid?: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备发起争议...');

    const tx = api.tx.swap.disputeSwap(swapId, reason, evidenceCid || null);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);

    console.log('[BridgeService] Swap disputed:', swapId);
  }

  /**
   * 查询用户的兑换历史记录
   * @param account 用户地址
   * @returns 兑换记录列表
   */
  async getSwapHistory(account: string): Promise<SwapRecord[]> {
    const api = this.getApi();

    try {
      // 查询链上存储 - 使用 swap pallet
      const entries = await api.query.swap.makerSwapRecords.entries();

      const records: SwapRecord[] = [];

      for (const [key, value] of entries) {
        const record = value.toJSON() as SwapRecordJson;

        // 过滤用户（作为买家）
        if (record.buyer === account) {
          records.push({
            id: parseInt(key.args[0].toString(), 10),
            buyer: record.buyer ?? '',
            makerId: parseNumber(record.makerId),
            dustAmount: BigInt(record.dustAmount ?? 0),
            usdtAmount: BigInt(record.usdtAmount ?? 0),
            buyerTronAddress: record.buyerTronAddress ?? '',
            makerTronAddress: record.makerTronAddress ?? '',
            status: parseSwapStatus(record.status),
            createdAt: record.timeInfo?.createdAt ?? 0,
            completedAt: record.timeInfo?.completedAt,
            tronTxHash: record.tronTxHash,
          });
        }
      }

      // 按创建时间倒序排序
      records.sort((a, b) => b.createdAt - a.createdAt);

      return records;
    } catch (error) {
      console.error('[BridgeService] Get history error:', error);
      throw error;
    }
  }

  /**
   * 查询单个兑换记录
   * @param swapId 兑换记录ID
   * @returns 兑换记录
   */
  async getSwapRecord(swapId: number): Promise<SwapRecord | null> {
    const api = this.getApi();

    try {
      const record = await api.query.swap.makerSwapRecords(swapId);

      if (record.isEmpty) {
        return null;
      }

      const data = record.toJSON() as SwapRecordJson;

      return {
        id: swapId,
        buyer: data.buyer ?? '',
        makerId: parseNumber(data.makerId),
        dustAmount: BigInt(data.dustAmount ?? 0),
        usdtAmount: BigInt(data.usdtAmount ?? 0),
        buyerTronAddress: data.buyerTronAddress ?? '',
        makerTronAddress: data.makerTronAddress ?? '',
        status: parseSwapStatus(data.status),
        createdAt: data.timeInfo?.createdAt ?? 0,
        completedAt: data.timeInfo?.completedAt,
        tronTxHash: data.tronTxHash,
      };
    } catch (error) {
      console.error('[BridgeService] Get record error:', error);
      throw error;
    }
  }

  /**
   * 订阅兑换记录状态变化
   * @param swapId 兑换记录ID
   * @param callback 状态变化回调
   * @returns 取消订阅函数
   */
  async subscribeToSwap(
    swapId: number,
    callback: (record: SwapRecord) => void
  ): Promise<() => void> {
    const api = this.getApi();

    const unsub = await api.query.swap.makerSwapRecords(swapId, (record) => {
      if (!record.isEmpty) {
        const data = record.toJSON() as SwapRecordJson;
        callback({
          id: swapId,
          buyer: data.buyer ?? '',
          makerId: parseNumber(data.makerId),
          dustAmount: BigInt(data.dustAmount ?? 0),
          usdtAmount: BigInt(data.usdtAmount ?? 0),
          buyerTronAddress: data.buyerTronAddress ?? '',
          makerTronAddress: data.makerTronAddress ?? '',
          status: parseSwapStatus(data.status),
          createdAt: data.timeInfo?.createdAt ?? 0,
          completedAt: data.timeInfo?.completedAt,
          tronTxHash: data.tronTxHash,
        });
      }
    });

    return unsub;
  }

  /**
   * 解析兑换状态（内部使用，保留向后兼容）
   * @deprecated 使用 parseSwapStatus 替代
   */
  private parseSwapStatus(status: unknown): SwapStatus {
    return parseSwapStatus(status) as SwapStatus;
  }

  /**
   * 获取当前 DUST 价格
   * @returns DUST 价格（USDT，精度 10^6）
   */
  async getDustPrice(): Promise<number> {
    const api = this.getApi();

    try {
      // 从 tradingPricing pallet 获取价格
      const coldStartExited = await api.query.tradingPricing.coldStartExited();

      if (!coldStartExited.isTrue) {
        // 冷启动期间，获取默认价格
        const defaultPrice = await api.query.tradingPricing.defaultPrice();
        const priceValue = defaultPrice.toNumber();
        return priceValue > 1000 ? priceValue / 1_000_000 : 0.1;
      }

      const stats = await api.query.tradingPricing.marketStats();
      const data = stats.toJSON() as MarketStatsJson;
      return (data?.weightedPrice ?? 100000) / 1_000_000;
    } catch (error) {
      console.error('[BridgeService] Get price error:', error);
      return 0.10; // 返回默认价格
    }
  }

  /**
   * 获取用户的 DUST 余额
   * @param account 用户地址
   * @returns DUST 余额（最小单位，精度 10^12）
   */
  async getDustBalance(account: string): Promise<bigint> {
    const api = this.getApi();

    try {
      const balance = await api.query.system.account(account);
      const data = balance.toJSON() as AccountDataJson;

      return BigInt(data.data?.free ?? 0);
    } catch (error) {
      console.error('[BridgeService] Get balance error:', error);
      throw error;
    }
  }

  /**
   * 格式化 DUST 数量为可读字符串
   * @param amount DUST 数量（最小单位）
   * @returns 格式化后的字符串
   */
  static formatDustAmount(amount: bigint): string {
    return (Number(amount) / 1e12).toFixed(4);
  }

  /**
   * 格式化 USDT 数量为可读字符串
   * @param amount USDT 数量（最小单位，精度 10^6）
   * @returns 格式化后的字符串
   */
  static formatUsdtAmount(amount: bigint): string {
    return (Number(amount) / 1e6).toFixed(2);
  }
}

// 导出单例
export const bridgeService = new BridgeService();
