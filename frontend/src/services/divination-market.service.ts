/**
 * 占卜市场服务 - 处理解卦师注册、订单管理、评价等
 */

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/lib/api';
import { signAndSend, getCurrentSignerAddress } from '@/lib/signer';
import { u8aToHex } from '@polkadot/util';
import type {
  RawProvider,
  RawServicePackage,
  RawMarketOrder,
  RawReview,
} from '@/types/substrate.types';
import { parseMarketOrderStatus, parseString, parseNumber } from '@/types/type-guards';

/** 链上数据的 JSON 表示 */
interface ProviderJson {
  id?: number;
  account?: string;
  name?: string;
  bio?: string;
  specialties?: number;
  supportedTypes?: number;
  deposit?: string | number;
  status?: string;
  rating?: number;
  totalOrders?: number;
  completedOrders?: number;
  createdAt?: number;
}

/** 链上订单数据的 JSON 表示 */
interface OrderJson {
  id?: number;
  customer?: string;
  providerId?: number;
  packageId?: number;
  questionCid?: string;
  answerCid?: string;
  amount?: string | number;
  status?: string;
  createdAt?: number;
  completedAt?: number;
}

/** 链上套餐数据的 JSON 表示 */
interface PackageJson {
  id?: number;
  providerId?: number;
  name?: string;
  description?: string;
  price?: string | number;
  duration?: number;
  isActive?: boolean;
}

/** 链上评价数据的 JSON 表示 */
interface ReviewJson {
  orderId?: number;
  customer?: string;
  providerId?: number;
  rating?: number;
  comment?: string;
  createdAt?: number;
}

/**
 * 签名状态回调
 */
export type StatusCallback = (status: string) => void;

/**
 * 解卦师状态
 */
export enum ProviderStatus {
  Pending = 'Pending',     // 待审核
  Active = 'Active',       // 已激活
  Paused = 'Paused',       // 已暂停
  Suspended = 'Suspended', // 被封禁
  Deactivated = 'Deactivated', // 已注销
}

/**
 * 订单状态
 */
export enum OrderStatus {
  Pending = 'Pending',       // 待接单
  Paid = 'Paid',             // 已支付
  Accepted = 'Accepted',     // 已接单
  Completed = 'Completed',   // 已完成
  Cancelled = 'Cancelled',   // 已取消
  Disputed = 'Disputed',     // 有争议
}

/**
 * 解卦师信息
 */
export interface Provider {
  id: number;
  account: string;
  name: string;
  bio: string;
  specialties: number;      // 位标志
  supportedTypes: number;   // 位标志
  deposit: bigint;
  status: ProviderStatus;
  rating: number;
  totalOrders: number;
  completedOrders: number;
  createdAt: number;
}

/**
 * 订单信息
 */
export interface Order {
  id: number;
  customer: string;
  providerId: number;
  packageId: number;
  questionCid: string;
  answerCid?: string;
  amount: bigint;
  status: OrderStatus;
  createdAt: number;
  completedAt?: number;
}

/**
 * 套餐信息
 */
export interface Package {
  id: number;
  providerId: number;
  name: string;
  description: string;
  price: bigint;
  duration: number; // 预计完成时间（秒）
  isActive: boolean;
}

/**
 * 评价信息
 */
export interface Review {
  orderId: number;
  customer: string;
  providerId: number;
  rating: number;
  comment: string;
  createdAt: number;
}

/**
 * 占卜市场服务类
 */
export class DivinationMarketService {
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

  // ===== 解卦师相关 =====

  /**
   * 注册为解卦师
   * @param name 名称
   * @param bio 简介
   * @param specialties 专长（位标志）
   * @param supportedTypes 支持的占卜类型（位标志）
   * @param deposit 保证金数量
   * @param onStatusChange 状态回调
   * @returns 解卦师ID
   */
  async registerProvider(
    name: string,
    bio: string,
    specialties: number,
    supportedTypes: number,
    deposit: bigint,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    // 创建交易
    const tx = api.tx.divinationMarket.registerProvider(
      name,
      bio,
      specialties,
      supportedTypes,
      deposit.toString()
    );

    onStatusChange?.('等待签名...');

    // 签名并发送交易
    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    // 从事件中提取解卦师ID
    const providerEvent = events.find(
      ({ event }: any) =>
        event.section === 'divinationMarket' &&
        event.method === 'ProviderRegistered'
    );

    if (!providerEvent) {
      throw new Error('未找到解卦师注册事件');
    }

    const providerId = providerEvent.event.data[1].toString();
    return parseInt(providerId, 10);
  }

  /**
   * 更新解卦师信息
   * @param name 名称
   * @param bio 简介
   * @param specialties 专长
   * @param supportedTypes 支持的占卜类型
   * @param onStatusChange 状态回调
   */
  async updateProvider(
    name: string,
    bio: string,
    specialties: number,
    supportedTypes: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.updateProvider(
      name,
      bio,
      specialties,
      supportedTypes
    );

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 注销解卦师（暂停服务）
   * @param onStatusChange 状态回调
   */
  async deactivateProvider(onStatusChange?: StatusCallback): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.deactivateProvider();

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 重新激活解卦师
   * @param onStatusChange 状态回调
   */
  async reactivateProvider(onStatusChange?: StatusCallback): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.reactivateProvider();

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 暂停服务（解卦师暂时停止接单）
   * @param onStatusChange 状态回调
   */
  async pauseProvider(onStatusChange?: StatusCallback): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.pauseProvider();

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 恢复服务（解卦师恢复接单）
   * @param onStatusChange 状态回调
   */
  async resumeProvider(onStatusChange?: StatusCallback): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.resumeProvider();

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 获取解卦师信息
   * @param providerId 解卦师ID
   * @returns 解卦师信息
   */
  async getProvider(providerId: number): Promise<Provider | null> {
    const api = this.getApi();

    try {
      const provider = await api.query.divinationMarket.providers(providerId);

      if (provider.isEmpty) {
        return null;
      }

      const data = provider.toJSON() as ProviderJson;

      return {
        id: parseNumber(data.id),
        account: parseString(data.account),
        name: parseString(data.name),
        bio: parseString(data.bio),
        specialties: parseNumber(data.specialties),
        supportedTypes: parseNumber(data.supportedTypes),
        deposit: BigInt(data.deposit ?? 0),
        status: (data.status as ProviderStatus) ?? ProviderStatus.Pending,
        rating: parseNumber(data.rating),
        totalOrders: parseNumber(data.totalOrders),
        completedOrders: parseNumber(data.completedOrders),
        createdAt: parseNumber(data.createdAt),
      };
    } catch (error) {
      console.error('[DivinationMarketService] Get provider error:', error);
      throw error;
    }
  }

  /**
   * 根据账户地址获取解卦师信息
   * @param account 账户地址
   * @returns 解卦师信息
   */
  async getProviderByAccount(account: string): Promise<Provider | null> {
    const api = this.getApi();

    try {
      const entries = await api.query.divinationMarket.providers.entries();

      for (const [key, value] of entries) {
        const data = value.toJSON() as ProviderJson;

        if (data.account === account) {
          return {
            id: parseNumber(data.id),
            account: parseString(data.account),
            name: parseString(data.name),
            bio: parseString(data.bio),
            specialties: parseNumber(data.specialties),
            supportedTypes: parseNumber(data.supportedTypes),
            deposit: BigInt(data.deposit ?? 0),
            status: (data.status as ProviderStatus) ?? ProviderStatus.Pending,
            rating: parseNumber(data.rating),
            totalOrders: parseNumber(data.totalOrders),
            completedOrders: parseNumber(data.completedOrders),
            createdAt: parseNumber(data.createdAt),
          };
        }
      }

      return null;
    } catch (error) {
      console.error('[DivinationMarketService] Get provider by account error:', error);
      return null;
    }
  }

  /**
   * 获取所有活跃的解卦师
   * @returns 解卦师列表
   */
  async getActiveProviders(): Promise<Provider[]> {
    const api = this.getApi();

    try {
      const entries = await api.query.divinationMarket.providers.entries();
      const providers: Provider[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as ProviderJson;

        if (data.status === 'Active') {
          providers.push({
            id: parseNumber(data.id),
            account: parseString(data.account),
            name: parseString(data.name),
            bio: parseString(data.bio),
            specialties: parseNumber(data.specialties),
            supportedTypes: parseNumber(data.supportedTypes),
            deposit: BigInt(data.deposit ?? 0),
            status: ProviderStatus.Active,
            rating: parseNumber(data.rating),
            totalOrders: parseNumber(data.totalOrders),
            completedOrders: parseNumber(data.completedOrders),
            createdAt: parseNumber(data.createdAt),
          });
        }
      }

      // 按评分排序
      providers.sort((a, b) => b.rating - a.rating);

      return providers;
    } catch (error) {
      console.error('[DivinationMarketService] Get providers error:', error);
      throw error;
    }
  }

  /**
   * 获取解卦师的订单列表
   * @param providerId 解卦师ID
   * @returns 订单列表
   */
  async getProviderOrders(providerId: number): Promise<Order[]> {
    const api = this.getApi();

    try {
      if (!api.query.divinationMarket?.orders) {
        return [];
      }
      const entries = await api.query.divinationMarket.orders.entries();
      const orders: Order[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as OrderJson;

        if (data.providerId === providerId) {
          orders.push({
            id: parseInt(key.args[0]?.toString() || '0', 10),
            customer: parseString(data.customer),
            providerId: parseNumber(data.providerId),
            packageId: parseNumber(data.packageId),
            questionCid: parseString(data.questionCid),
            answerCid: data.answerCid,
            amount: BigInt(data.amount ?? 0),
            status: (data.status as OrderStatus) ?? OrderStatus.Pending,
            createdAt: parseNumber(data.createdAt),
            completedAt: data.completedAt,
          });
        }
      }

      // 按创建时间倒序排序
      orders.sort((a, b) => b.createdAt - a.createdAt);

      return orders;
    } catch (error) {
      console.error('[DivinationMarketService] Get provider orders error:', error);
      return [];
    }
  }

  // ===== 套餐相关 =====

  /**
   * 创建套餐
   * @param name 套餐名称
   * @param description 描述
   * @param price 价格
   * @param duration 预计完成时间
   * @param onStatusChange 状态回调
   * @returns 套餐ID
   */
  async createPackage(
    name: string,
    description: string,
    price: bigint,
    duration: number,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.createPackage(
      name,
      description,
      price.toString(),
      duration
    );

    onStatusChange?.('等待签名...');

    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    const packageEvent = events.find(
      ({ event }: any) =>
        event.section === 'divinationMarket' &&
        event.method === 'PackageCreated'
    );

    if (!packageEvent) {
      throw new Error('未找到套餐创建事件');
    }

    const packageId = packageEvent.event.data[1].toString();
    return parseInt(packageId, 10);
  }

  /**
   * 获取解卦师的套餐列表
   * @param providerId 解卦师ID
   * @returns 套餐列表
   */
  async getProviderPackages(providerId: number): Promise<Package[]> {
    const api = this.getApi();

    try {
      const entries = await api.query.divinationMarket.packages.entries();
      const packages: Package[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as PackageJson;

        if (data.providerId === providerId) {
          packages.push({
            id: parseNumber(data.id),
            providerId: parseNumber(data.providerId),
            name: parseString(data.name),
            description: parseString(data.description),
            price: BigInt(data.price ?? 0),
            duration: parseNumber(data.duration),
            isActive: data.isActive ?? false,
          });
        }
      }

      return packages;
    } catch (error) {
      console.error('[DivinationMarketService] Get packages error:', error);
      throw error;
    }
  }

  /**
   * 更新套餐信息
   * @param packageId 套餐ID
   * @param name 新名称（可选）
   * @param description 新描述（可选）
   * @param price 新价格（可选）
   * @param duration 新预计时间（可选）
   * @param onStatusChange 状态回调
   */
  async updatePackage(
    packageId: number,
    name?: string,
    description?: string,
    price?: bigint,
    duration?: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.updatePackage(
      packageId,
      name || null,
      description || null,
      price?.toString() || null,
      duration || null
    );

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 停用套餐
   * @param packageId 套餐ID
   * @param onStatusChange 状态回调
   */
  async deactivatePackage(
    packageId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.deactivatePackage(packageId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 删除套餐
   * @param packageId 套餐ID
   * @param onStatusChange 状态回调
   */
  async removePackage(
    packageId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.removePackage(packageId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 重新激活套餐
   * @param packageId 套餐ID
   * @param onStatusChange 状态回调
   */
  async reactivatePackage(
    packageId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.reactivatePackage(packageId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  // ===== 订单相关 =====

  /**
   * 创建订单
   * @param providerId 解卦师ID
   * @param packageId 套餐ID
   * @param question 问题内容
   * @param amount 支付金额
   * @param onStatusChange 状态回调
   * @returns 订单ID
   */
  async createOrder(
    providerId: number,
    packageId: number,
    question: string,
    amount: bigint,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    // 将问题内容转换为CID
    const questionBytes = new TextEncoder().encode(question);
    const questionCid = u8aToHex(questionBytes);

    const tx = api.tx.divinationMarket.createOrder(
      providerId,
      packageId,
      questionCid,
      amount.toString()
    );

    onStatusChange?.('等待签名...');

    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    const orderEvent = events.find(
      ({ event }: any) =>
        event.section === 'divinationMarket' &&
        event.method === 'OrderCreated'
    );

    if (!orderEvent) {
      throw new Error('未找到订单创建事件');
    }

    const orderId = orderEvent.event.data[1].toString();
    return parseInt(orderId, 10);
  }

  /**
   * 提交解答（解卦师完成订单）
   * @param orderId 订单ID
   * @param answer 解答内容
   * @param onStatusChange 状态回调
   */
  async submitAnswer(
    orderId: number,
    answer: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const answerBytes = new TextEncoder().encode(answer);
    const answerCid = u8aToHex(answerBytes);

    const tx = api.tx.divinationMarket.submitAnswer(orderId, answerCid);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 接受订单（解卦师调用）
   * @param orderId 订单ID
   * @param onStatusChange 状态回调
   */
  async acceptOrder(
    orderId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.acceptOrder(orderId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 拒绝订单（解卦师调用）
   * @param orderId 订单ID
   * @param reason 拒绝原因（可选）
   * @param onStatusChange 状态回调
   */
  async rejectOrder(
    orderId: number,
    reason?: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.rejectOrder(orderId, reason || null);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 提交解卦结果（解卦师调用）
   * @param orderId 订单ID
   * @param answerCid 解答内容的 IPFS CID
   * @param onStatusChange 状态回调
   */
  async submitInterpretation(
    orderId: number,
    answerCid: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.submitInterpretation(orderId, answerCid);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 确认解卦结果（用户调用）
   * @param orderId 订单ID
   * @param onStatusChange 状态回调
   */
  async confirmInterpretation(
    orderId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.confirmInterpretation(orderId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 更新解卦结果（解卦师调用，用于修改已提交的解答）
   * @param orderId 订单ID
   * @param newAnswerCid 新解答内容的 IPFS CID
   * @param onStatusChange 状态回调
   */
  async updateInterpretation(
    orderId: number,
    newAnswerCid: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.updateInterpretation(orderId, newAnswerCid);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 取消订单（用户或解卦师调用）
   * @param orderId 订单ID
   * @param reason 取消原因（可选）
   * @param onStatusChange 状态回调
   */
  async cancelOrder(
    orderId: number,
    reason?: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.cancelOrder(orderId, reason || null);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 确认完成订单（用户调用）
   *
   * 用户在收到解答后调用此方法确认完成，资金将释放给解卦师
   *
   * @param orderId 订单ID
   * @param onStatusChange 状态回调
   */
  async completeOrder(
    orderId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.completeOrder(orderId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 发起订单争议
   *
   * 当用户对解答不满意或解卦师未按时完成时，可以发起争议
   *
   * @param orderId 订单ID
   * @param reason 争议原因
   * @param evidenceCid 证据的 IPFS CID（可选）
   * @param onStatusChange 状态回调
   */
  async disputeOrder(
    orderId: number,
    reason: string,
    evidenceCid?: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.disputeOrder(
      orderId,
      reason,
      evidenceCid || null
    );

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 订阅订单状态变化
   * @param orderId 订单ID
   * @param callback 状态变化回调
   * @returns 取消订阅函数
   */
  async subscribeToOrder(
    orderId: number,
    callback: (order: Order) => void
  ): Promise<() => void> {
    const api = this.getApi();

    const unsub = await api.query.divinationMarket.orders(orderId, (order) => {
      if (!order.isEmpty) {
        const data = order.toJSON() as OrderJson;
        callback({
          id: parseNumber(data.id),
          customer: parseString(data.customer),
          providerId: parseNumber(data.providerId),
          packageId: parseNumber(data.packageId),
          questionCid: parseString(data.questionCid),
          answerCid: data.answerCid,
          amount: BigInt(data.amount ?? 0),
          status: (data.status as OrderStatus) ?? OrderStatus.Pending,
          createdAt: parseNumber(data.createdAt),
          completedAt: data.completedAt,
        });
      }
    });

    return unsub;
  }

  /**
   * 获取订单详情
   * @param orderId 订单ID
   * @returns 订单信息
   */
  async getOrder(orderId: number): Promise<Order | null> {
    const api = this.getApi();

    try {
      const order = await api.query.divinationMarket.orders(orderId);

      if (order.isEmpty) {
        return null;
      }

      const data = order.toJSON() as OrderJson;

      return {
        id: parseNumber(data.id),
        customer: parseString(data.customer),
        providerId: parseNumber(data.providerId),
        packageId: parseNumber(data.packageId),
        questionCid: parseString(data.questionCid),
        answerCid: data.answerCid,
        amount: BigInt(data.amount ?? 0),
        status: (data.status as OrderStatus) ?? OrderStatus.Pending,
        createdAt: parseNumber(data.createdAt),
        completedAt: data.completedAt,
      };
    } catch (error) {
      console.error('[DivinationMarketService] Get order error:', error);
      throw error;
    }
  }

  /**
   * 获取用户的订单列表
   * @param account 用户地址
   * @returns 订单列表
   */
  async getUserOrders(account: string): Promise<Order[]> {
    const api = this.getApi();

    try {
      const entries = await api.query.divinationMarket.orders.entries();
      const orders: Order[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as OrderJson;

        if (data.customer === account) {
          orders.push({
            id: parseNumber(data.id),
            customer: parseString(data.customer),
            providerId: parseNumber(data.providerId),
            packageId: parseNumber(data.packageId),
            questionCid: parseString(data.questionCid),
            answerCid: data.answerCid,
            amount: BigInt(data.amount ?? 0),
            status: (data.status as OrderStatus) ?? OrderStatus.Pending,
            createdAt: parseNumber(data.createdAt),
            completedAt: data.completedAt,
          });
        }
      }

      // 按时间倒序排序
      orders.sort((a, b) => b.createdAt - a.createdAt);

      return orders;
    } catch (error) {
      console.error('[DivinationMarketService] Get user orders error:', error);
      throw error;
    }
  }

  // ===== 评价相关 =====

  /**
   * 提交评价
   * @param orderId 订单ID
   * @param rating 评分 (1-5)
   * @param comment 评价内容
   * @param onStatusChange 状态回调
   */
  async submitReview(
    orderId: number,
    rating: number,
    comment: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    if (rating < 1 || rating > 5) {
      throw new Error('Rating must be between 1 and 5');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.submitReview(orderId, rating, comment);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 提现（解卦师提取已完成订单的收入）
   * @param amount 提现金额（可选，不传则提取全部）
   * @param onStatusChange 状态回调
   */
  async withdraw(amount?: bigint, onStatusChange?: StatusCallback): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = amount
      ? api.tx.divinationMarket.withdraw(amount.toString())
      : api.tx.divinationMarket.withdrawAll();

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 申请提现（解卦师申请提取收入，需要等待冷却期）
   * @param amount 提现金额
   * @param onStatusChange 状态回调
   */
  async requestWithdrawal(
    amount: bigint,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.requestWithdrawal(amount.toString());

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 提交追问（用户对解答有疑问时提交追问）
   * @param orderId 订单ID
   * @param questionCid 追问内容的 IPFS CID
   * @param onStatusChange 状态回调
   */
  async submitFollowUp(
    orderId: number,
    questionCid: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.submitFollowUp(orderId, questionCid);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 回复追问（解卦师回复用户的追问）
   * @param orderId 订单ID
   * @param followUpId 追问ID
   * @param answerCid 回复内容的 IPFS CID
   * @param onStatusChange 状态回调
   */
  async replyFollowUp(
    orderId: number,
    followUpId: number,
    answerCid: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.replyFollowUp(orderId, followUpId, answerCid);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 回复评价（解卦师回复用户的评价）
   * @param reviewId 评价ID
   * @param reply 回复内容
   * @param onStatusChange 状态回调
   */
  async replyReview(
    reviewId: number,
    reply: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.replyReview(reviewId, reply);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 打赏（额外打赏解卦师）
   * @param providerId 解卦师ID
   * @param amount 打赏金额
   * @param orderId 相关订单ID（可选）
   * @param onStatusChange 状态回调
   */
  async tip(
    providerId: number,
    amount: bigint,
    orderId?: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.divinationMarket.tip(
      providerId,
      amount.toString(),
      orderId || null
    );

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 获取解卦师的评价列表
   * @param providerId 解卦师ID
   * @returns 评价列表
   */
  async getProviderReviews(providerId: number): Promise<Review[]> {
    const api = this.getApi();

    try {
      const entries = await api.query.divinationMarket.reviews.entries();
      const reviews: Review[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as ReviewJson;

        if (data.providerId === providerId) {
          reviews.push({
            orderId: parseNumber(data.orderId),
            customer: parseString(data.customer),
            providerId: parseNumber(data.providerId),
            rating: parseNumber(data.rating),
            comment: parseString(data.comment),
            createdAt: parseNumber(data.createdAt),
          });
        }
      }

      // 按时间倒序排序
      reviews.sort((a, b) => b.createdAt - a.createdAt);

      return reviews;
    } catch (error) {
      console.error('[DivinationMarketService] Get reviews error:', error);
      throw error;
    }
  }

  // ===== 收益相关 =====

  /**
   * 获取解卦师收益信息
   * @param providerId 解卦师ID
   * @returns 收益信息
   */
  async getProviderEarnings(providerId: number): Promise<{
    availableBalance: bigint;
    totalEarnings: bigint;
    pendingWithdrawal: bigint;
    lastWithdrawalAt?: number;
  }> {
    const api = this.getApi();

    try {
      // 查询解卦师收益存储
      const earnings = await api.query.divinationMarket.providerEarnings(providerId);

      if (earnings.isEmpty) {
        return {
          availableBalance: BigInt(0),
          totalEarnings: BigInt(0),
          pendingWithdrawal: BigInt(0),
        };
      }

      const data = earnings.toJSON() as {
        availableBalance?: string | number;
        totalEarnings?: string | number;
        pendingWithdrawal?: string | number;
        lastWithdrawalAt?: number;
      };

      return {
        availableBalance: BigInt(data.availableBalance ?? 0),
        totalEarnings: BigInt(data.totalEarnings ?? 0),
        pendingWithdrawal: BigInt(data.pendingWithdrawal ?? 0),
        lastWithdrawalAt: data.lastWithdrawalAt,
      };
    } catch (error) {
      console.error('[DivinationMarketService] Get earnings error:', error);
      // 返回默认值
      return {
        availableBalance: BigInt(0),
        totalEarnings: BigInt(0),
        pendingWithdrawal: BigInt(0),
      };
    }
  }

  /**
   * 获取解卦师提现历史
   * @param providerId 解卦师ID
   * @returns 提现记录列表
   */
  async getWithdrawalHistory(providerId: number): Promise<Array<{
    id: number;
    amount: bigint;
    status: 'pending' | 'completed' | 'failed';
    createdAt: number;
    completedAt?: number;
    txHash?: string;
  }>> {
    const api = this.getApi();

    try {
      // 查询提现记录
      const entries = await api.query.divinationMarket.withdrawalRecords?.entries?.();

      if (!entries) {
        return [];
      }

      const records: Array<{
        id: number;
        amount: bigint;
        status: 'pending' | 'completed' | 'failed';
        createdAt: number;
        completedAt?: number;
        txHash?: string;
      }> = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as {
          providerId?: number;
          amount?: string | number;
          status?: string;
          createdAt?: number;
          completedAt?: number;
          txHash?: string;
        };

        if (data.providerId === providerId) {
          records.push({
            id: parseInt(key.args[0]?.toString() || '0', 10),
            amount: BigInt(data.amount ?? 0),
            status: (data.status?.toLowerCase() as 'pending' | 'completed' | 'failed') ?? 'pending',
            createdAt: data.createdAt ?? 0,
            completedAt: data.completedAt,
            txHash: data.txHash,
          });
        }
      }

      // 按时间倒序排序
      records.sort((a, b) => b.createdAt - a.createdAt);

      return records;
    } catch (error) {
      console.error('[DivinationMarketService] Get withdrawal history error:', error);
      return [];
    }
  }
}

// 导出单例
export const divinationMarketService = new DivinationMarketService();
