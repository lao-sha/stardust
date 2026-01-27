/**
 * 星尘玄鉴 - Trading 服务层
 * 封装与区块链交互的 API
 */

import { ApiPromise } from '@polkadot/api';
import CryptoJS from 'crypto-js';
import { getApi } from '@/lib/api';
import { signAndSend, getCurrentSignerAddress } from '@/lib/signer';
import type { Maker, Order, MarketStats, BuyerCreditInfo, KycStatus } from '@/stores/trading.store';
import type { ChainEvent, DispatchError } from '@/types/substrate.types';
import {
  parseOtcOrderState,
  parseDisputeStatus,
  parseDisputeResolution,
  parseBoolean,
  parseString,
} from '@/types/type-guards';
import { createErrorHandler } from '@/lib/error-handler';
import { APIConnectionError, TransactionError, NetworkError } from '@/lib/errors';
import { createLogger } from '@/lib/logger';

// 创建模块级日志器和错误处理器
const log = createLogger('TradingService');
const errorHandler = createErrorHandler('TradingService');

/**
 * Trading Service
 * 提供与 pallet-trading 交互的方法
 */
export class TradingService {
  /**
   * 获取 API 实例
   */
  private getApi(): ApiPromise {
    try {
      return getApi();
    } catch (error) {
      throw new APIConnectionError('API 未初始化，请先连接到区块链节点', error);
    }
  }

  // ===== 做市商相关 =====

  /**
   * 获取所有活跃做市商
   */
  async getMakers(): Promise<Maker[]> {
    const api = this.getApi();

    try {
      // 查询所有做市商申请
      const entries = await api.query.maker.makerApplications.entries();

      const makers: Maker[] = [];

      for (const [key, value] of entries) {
        const makerId = key.args[0].toNumber();
        const app = value.unwrap();

        // 只返回已激活且未暂停的做市商
        if (app.status.isActive && !app.servicePaused.isTrue) {
          makers.push({
            id: makerId,
            owner: app.owner.toString(),
            tronAddress: app.tronAddress.toHuman() as string,
            buyPremiumBps: app.buyPremiumBps.toNumber(),
            sellPremiumBps: app.sellPremiumBps.toNumber(),
            minAmount: app.minAmount.toBigInt(),
            servicePaused: app.servicePaused.isTrue,
            usersServed: app.usersServed.toNumber(),
            maskedFullName: app.maskedFullName.toHuman() as string,
            wechatId: app.wechatId.toHuman() as string,
            rating: this.calculateMakerRating(app.usersServed.toNumber()),
          });
        }
      }

      // 按评分排序
      makers.sort((a, b) => b.rating - a.rating);

      return makers;
    } catch (error) {
      errorHandler.handle(error, 'getMakers');
      throw error instanceof APIConnectionError ? error : new NetworkError('获取做市商列表失败', error);
    }
  }

  /**
   * 获取做市商详情
   */
  async getMaker(makerId: number): Promise<Maker | null> {
    const api = this.getApi();

    try {
      const app = await api.query.maker.makerApplications(makerId);

      if (app.isNone) {
        return null;
      }

      const data = app.unwrap();

      return {
        id: makerId,
        owner: data.owner.toString(),
        tronAddress: data.tronAddress.toHuman() as string,
        buyPremiumBps: data.buyPremiumBps.toNumber(),
        sellPremiumBps: data.sellPremiumBps.toNumber(),
        minAmount: data.minAmount.toBigInt(),
        servicePaused: data.servicePaused.isTrue,
        usersServed: data.usersServed.toNumber(),
        maskedFullName: data.maskedFullName.toHuman() as string,
        wechatId: data.wechatId.toHuman() as string,
        rating: this.calculateMakerRating(data.usersServed.toNumber()),
      };
    } catch (error) {
      log.error('Get maker error:', error);
      throw error;
    }
  }

  /**
   * 计算做市商评分
   */
  private calculateMakerRating(usersServed: number): number {
    // 简单的评分算法：基于服务用户数
    if (usersServed >= 1000) return 5.0;
    if (usersServed >= 500) return 4.9;
    if (usersServed >= 200) return 4.8;
    if (usersServed >= 100) return 4.7;
    if (usersServed >= 50) return 4.6;
    return 4.5;
  }

  // ===== 订单相关 =====

  /**
   * 创建首购订单
   */
  async createFirstPurchase(
    accountAddress: string,
    makerId: number,
    paymentCommit: string,
    contactCommit: string,
    onStatusChange?: (status: string) => void
  ): Promise<number> {
    const api = this.getApi();

    try {
      const tx = api.tx.otcOrder.createFirstPurchase(
        makerId,
        paymentCommit,
        contactCommit
      );

      const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

      // 解析事件获取订单 ID
      for (const { event } of events) {
        if (api.events.otcOrder.FirstPurchaseCreated.is(event)) {
          const [orderId] = event.data;
          log.info('First purchase created, order ID:', orderId.toString());
          return orderId.toNumber();
        }
      }

      throw new Error('Order ID not found in events');
    } catch (error) {
      log.error('Create first purchase error:', error);
      throw error;
    }
  }

  /**
   * 创建普通订单
   */
  async createOrder(
    accountAddress: string,
    makerId: number,
    dustAmount: bigint,
    paymentCommit: string,
    contactCommit: string,
    onStatusChange?: (status: string) => void
  ): Promise<number> {
    const api = this.getApi();

    try {
      const tx = api.tx.otcOrder.createOrder(
        makerId,
        dustAmount.toString(),
        paymentCommit,
        contactCommit
      );

      const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

      // 解析事件获取订单 ID
      for (const { event } of events) {
        if (api.events.otcOrder.OrderCreated.is(event)) {
          const [orderId] = event.data;
          log.info('Order created, order ID:', orderId.toString());
          return orderId.toNumber();
        }
      }

      throw new Error('Order ID not found in events');
    } catch (error) {
      log.error('Create order error:', error);
      throw error;
    }
  }

  /**
   * 标记已付款
   */
  async markPaid(
    accountAddress: string,
    orderId: number,
    tronTxHash?: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.otcOrder.markPaid(orderId, tronTxHash || null);
      await signAndSend(api, tx, accountAddress, onStatusChange);
      log.info('Mark paid success');
    } catch (error) {
      log.error('Mark paid error:', error);
      throw error;
    }
  }

  /**
   * 取消订单
   */
  async cancelOrder(
    accountAddress: string,
    orderId: number,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.otcOrder.cancelOrder(orderId);
      await signAndSend(api, tx, accountAddress, onStatusChange);
      log.info('Cancel order success');
    } catch (error) {
      log.error('Cancel order error:', error);
      throw error;
    }
  }

  /**
   * 发起争议（买家或做市商调用）
   *
   * 当交易出现问题时，任一方都可以发起争议。
   * 发起后进入仲裁流程，等待对方响应。
   *
   * @param accountAddress 发起者地址
   * @param orderId 订单ID
   * @param reason 争议原因描述
   * @param evidenceCid 证据的 IPFS CID（可选）
   * @param onStatusChange 状态回调
   */
  async dispute(
    accountAddress: string,
    orderId: number,
    reason: string,
    evidenceCid?: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      // 生成证据哈希（如果有证据）
      const evidenceHash = evidenceCid
        ? '0x' + CryptoJS.SHA256(evidenceCid).toString()
        : null;

      // 使用新的 initiateDispute 方法
      const tx = api.tx.otcOrder.initiateDispute(orderId, reason, evidenceHash);
      await signAndSend(api, tx, accountAddress, onStatusChange);
      log.info('Dispute initiated:', orderId);
    } catch (error) {
      log.error('Dispute error:', error);
      throw error;
    }
  }

  /**
   * 响应争议（被争议方调用）
   *
   * 当对方发起争议后，被争议方需要在规定时间内响应。
   * 如果不响应，可能会自动判定对方胜诉。
   *
   * @param accountAddress 响应者地址
   * @param orderId 订单ID
   * @param response 响应内容
   * @param evidenceCid 证据的 IPFS CID（可选）
   * @param onStatusChange 状态回调
   */
  async respondDispute(
    accountAddress: string,
    orderId: number,
    response: string,
    evidenceCid?: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      // 生成证据哈希（如果有证据）
      const evidenceHash = evidenceCid
        ? '0x' + CryptoJS.SHA256(evidenceCid).toString()
        : null;

      const tx = api.tx.otcOrder.respondDispute(orderId, response, evidenceHash);
      await signAndSend(api, tx, accountAddress, onStatusChange);
      log.info('Dispute response submitted:', orderId);
    } catch (error) {
      log.error('Respond dispute error:', error);
      throw error;
    }
  }

  /**
   * 获取争议详情
   *
   * @param orderId 订单ID
   * @returns 争议信息或 null
   */
  async getDispute(orderId: number): Promise<{
    initiator: string;
    reason: string;
    initiatorEvidence?: string;
    respondentResponse?: string;
    respondentEvidence?: string;
    status: 'Initiated' | 'Responded' | 'Resolved';
    deadline: number;
    resolution?: 'BuyerWins' | 'MakerWins' | 'Split';
  } | null> {
    const api = this.getApi();

    try {
      const dispute = await api.query.otcOrder.disputes(orderId);

      if (dispute.isNone) {
        return null;
      }

      const data = dispute.unwrap();

      return {
        initiator: data.initiator.toString(),
        reason: parseString(data.reason.toHuman()),
        initiatorEvidence: data.initiatorEvidence.isSome
          ? data.initiatorEvidence.unwrap().toHex()
          : undefined,
        respondentResponse: data.respondentResponse.isSome
          ? parseString(data.respondentResponse.unwrap().toHuman())
          : undefined,
        respondentEvidence: data.respondentEvidence.isSome
          ? data.respondentEvidence.unwrap().toHex()
          : undefined,
        status: parseDisputeStatus(data.status.toString()),
        deadline: data.deadline.toNumber(),
        resolution: data.resolution.isSome
          ? parseDisputeResolution(data.resolution.unwrap().toString())
          : undefined,
      };
    } catch (error) {
      log.error('Get dispute error:', error);
      return null;
    }
  }

  /**
   * 获取订单详情
   */
  async getOrder(orderId: number): Promise<Order | null> {
    const api = this.getApi();

    try {
      const order = await api.query.otcOrder.orders(orderId);

      if (order.isNone) {
        return null;
      }

      const data = order.unwrap();

      return {
        id: orderId,
        makerId: data.makerId.toNumber(),
        maker: data.maker.toString(),
        taker: data.taker.toString(),
        price: data.price.toBigInt(),
        qty: data.qty.toBigInt(),
        amount: data.amount.toBigInt(),
        createdAt: data.createdAt.toNumber(),
        expireAt: data.expireAt.toNumber(),
        makerTronAddress: parseString(data.makerTronAddress.toHuman()),
        state: parseOtcOrderState(data.state.toString()),
        isFirstPurchase: parseBoolean(data.isFirstPurchase),
      };
    } catch (error) {
      log.error('Get order error:', error);
      throw error;
    }
  }

  /**
   * 订阅订单状态
   */
  async subscribeToOrder(
    orderId: number,
    callback: (order: Order) => void
  ): Promise<() => void> {
    const api = this.getApi();

    const unsub = await api.query.otcOrder.orders(orderId, (order) => {
      if (order.isSome) {
        const data = order.unwrap();
        callback({
          id: orderId,
          makerId: data.makerId.toNumber(),
          maker: data.maker.toString(),
          taker: data.taker.toString(),
          price: data.price.toBigInt(),
          qty: data.qty.toBigInt(),
          amount: data.amount.toBigInt(),
          createdAt: data.createdAt.toNumber(),
          expireAt: data.expireAt.toNumber(),
          makerTronAddress: parseString(data.makerTronAddress.toHuman()),
          state: parseOtcOrderState(data.state.toString()),
          isFirstPurchase: parseBoolean(data.isFirstPurchase),
        });
      }
    });

    return unsub;
  }

  /**
   * 获取买家订单历史
   */
  async getOrderHistory(buyer: string): Promise<Order[]> {
    const api = this.getApi();

    try {
      const orderIds = await api.query.otcOrder.buyerOrders(buyer);
      const orders: Order[] = [];

      for (const orderId of orderIds) {
        const order = await this.getOrder(orderId.toNumber());
        if (order) {
          orders.push(order);
        }
      }

      // 按创建时间倒序排序
      orders.sort((a, b) => b.createdAt - a.createdAt);

      return orders;
    } catch (error) {
      log.error('Get order history error:', error);
      throw error;
    }
  }

  // ===== 价格相关 =====

  /**
   * 获取市场统计
   */
  async getMarketStats(): Promise<MarketStats> {
    const api = this.getApi();

    try {
      // 尝试获取市场统计
      const stats = await api.query.tradingPricing.marketStats();
      
      // 获取默认价格（用于冷启动期间）
      const defaultPrice = await api.query.tradingPricing.defaultPrice();
      const defaultPriceValue = defaultPrice.toNumber();
      
      // 检查是否在冷启动期间
      const coldStartExited = await api.query.tradingPricing.coldStartExited();
      
      let weightedPrice: number;
      
      if (!coldStartExited.isTrue) {
        // 冷启动期间，使用默认价格或配置的初始价格
        // 默认价格精度是 10^6，所以 100000 = 0.1 USDT
        weightedPrice = defaultPriceValue > 1 ? defaultPriceValue / 1_000_000 : 0.1;
      } else {
        weightedPrice = stats.weightedPrice.toNumber() / 1_000_000;
      }

      return {
        otcPrice: stats.otcPrice.toNumber() / 1_000_000 || weightedPrice,
        bridgePrice: stats.bridgePrice.toNumber() / 1_000_000 || weightedPrice,
        weightedPrice: weightedPrice,
        simpleAvgPrice: stats.simpleAvgPrice.toNumber() / 1_000_000 || weightedPrice,
        otcVolume: stats.otcVolume.toBigInt(),
        bridgeVolume: stats.bridgeVolume.toBigInt(),
        totalVolume: stats.totalVolume.toBigInt(),
      };
    } catch (error) {
      log.error('Get market stats error:', error);
      // 返回默认值
      return {
        otcPrice: 0.1,
        bridgePrice: 0.1,
        weightedPrice: 0.1,
        simpleAvgPrice: 0.1,
        otcVolume: BigInt(0),
        bridgeVolume: BigInt(0),
        totalVolume: BigInt(0),
      };
    }
  }

  /**
   * 获取 DUST 价格
   */
  async getDustPrice(): Promise<number> {
    try {
      const api = this.getApi();
      
      // 尝试获取加权市场价格
      const coldStartExited = await api.query.tradingPricing.coldStartExited();
      
      if (!coldStartExited.isTrue) {
        // 冷启动期间，获取默认价格
        const defaultPrice = await api.query.tradingPricing.defaultPrice();
        const priceValue = defaultPrice.toNumber();
        // 如果默认价格太小（如 1），使用 0.1 作为初始价格
        return priceValue > 1000 ? priceValue / 1_000_000 : 0.1;
      }
      
      const stats = await this.getMarketStats();
      return stats.weightedPrice;
    } catch (error) {
      log.error('Get dust price error:', error);
      return 0.1; // 默认价格
    }
  }

  // ===== 信用相关 =====

  /**
   * 获取买家信用信息
   */
  async getBuyerCredit(buyer: string): Promise<BuyerCreditInfo> {
    const api = this.getApi();

    try {
      const credit = await api.query.credit.buyerCredits(buyer);

      if (credit.isNone) {
        // 新用户默认信用
        return {
          riskScore: 500,
          level: '新用户',
          maxAmount: 10,
          concurrentOrders: 0,
          maxConcurrentOrders: 1,
          completedOrders: 0,
          trend: 'stable',
        };
      }

      const data = credit.unwrap();
      const riskScore = data.riskScore.toNumber();
      const completedOrders = data.completedOrders.toNumber();

      return {
        riskScore,
        level: this.getCreditLevel(riskScore),
        maxAmount: data.maxAmount.toNumber() / 1_000_000,
        concurrentOrders: data.concurrentOrders.toNumber(),
        maxConcurrentOrders: data.maxConcurrentOrders.toNumber(),
        completedOrders,
        trend: this.calculateCreditTrend(riskScore, completedOrders),
      };
    } catch (error) {
      log.error('Get buyer credit error:', error);
      throw error;
    }
  }

  /**
   * 计算信用趋势
   * 基于风险分数和完成订单数判断趋势
   */
  private calculateCreditTrend(
    riskScore: number,
    completedOrders: number
  ): 'up' | 'down' | 'stable' {
    // 新用户（订单少于3个）趋势为稳定
    if (completedOrders < 3) {
      return 'stable';
    }

    // 风险分数越低越好
    // 低风险（<300）且有足够订单 -> 上升趋势
    if (riskScore < 300 && completedOrders >= 10) {
      return 'up';
    }

    // 高风险（>600）-> 下降趋势
    if (riskScore > 600) {
      return 'down';
    }

    // 中等风险但订单多 -> 可能上升
    if (riskScore < 500 && completedOrders >= 20) {
      return 'up';
    }

    return 'stable';
  }

  /**
   * 获取信用等级
   */
  private getCreditLevel(riskScore: number): string {
    if (riskScore <= 200) return '高信任';
    if (riskScore <= 400) return '中信任';
    if (riskScore <= 600) return '低信任';
    return '极低信任';
  }

  /**
   * 检查是否完成首购
   */
  async hasCompletedFirstPurchase(buyer: string): Promise<boolean> {
    const api = this.getApi();

    try {
      const hasCompleted = await api.query.otcOrder.firstPurchaseCompleted(buyer);
      return hasCompleted.isTrue;
    } catch (error) {
      log.error('Check first purchase error:', error);
      return false;
    }
  }

  // ===== KYC 相关 =====

  /**
   * 检查 KYC 状态
   */
  async checkKycStatus(account: string): Promise<{
    status: KycStatus;
    failureReason: string | null;
  }> {
    const api = this.getApi();

    try {
      // 获取 KYC 配置
      const config = await api.query.otcOrder.kycConfig();

      // 如果 KYC 未启用
      if (!config.enabled.isTrue) {
        return {
          status: 'Skipped' as KycStatus,
          failureReason: null,
        };
      }

      // 检查是否为豁免账户
      const isExempt = await api.query.otcOrder.kycExemptAccounts(account);
      if (isExempt.isSome) {
        return {
          status: 'Exempted' as KycStatus,
          failureReason: null,
        };
      }

      // 检查身份认证
      const identity = await api.query.identity.identityOf(account);
      if (identity.isNone) {
        return {
          status: 'Failed' as KycStatus,
          failureReason: 'IdentityNotSet',
        };
      }

      // 检查认证等级（judgements）
      const identityData = identity.unwrap();
      const judgements = identityData.judgements;
      
      // 获取所需的最低认证等级
      const requiredLevel = config.requiredLevel?.toNumber?.() ?? 1;
      
      // 检查是否有有效的认证判定
      let hasValidJudgement = false;
      let highestLevel = 0;
      
      for (const [, judgement] of judgements) {
        // 判定类型：Unknown, FeePaid, Reasonable, KnownGood, OutOfDate, LowQuality, Erroneous
        const judgementType = judgement.toString();
        
        if (judgementType === 'Reasonable') {
          highestLevel = Math.max(highestLevel, 1);
          hasValidJudgement = true;
        } else if (judgementType === 'KnownGood') {
          highestLevel = Math.max(highestLevel, 2);
          hasValidJudgement = true;
        } else if (judgementType === 'LowQuality' || judgementType === 'Erroneous') {
          // 负面判定
          return {
            status: 'Failed' as KycStatus,
            failureReason: 'QualityIssue',
          };
        }
      }

      if (!hasValidJudgement) {
        return {
          status: 'Failed' as KycStatus,
          failureReason: 'NoValidJudgement',
        };
      }

      if (highestLevel < requiredLevel) {
        return {
          status: 'Failed' as KycStatus,
          failureReason: 'InsufficientLevel',
        };
      }

      return {
        status: 'Passed' as KycStatus,
        failureReason: null,
      };
    } catch (error) {
      log.error('Check KYC error:', error);
      throw error;
    }
  }

  // ===== 工具方法 =====

  /**
   * 生成支付承诺哈希
   */
  static generatePaymentCommit(
    realName: string,
    idCard: string,
    phone: string
  ): string {
    const data = `${realName}|${idCard}|${phone}`;
    return '0x' + CryptoJS.SHA256(data).toString();
  }

  /**
   * 生成联系方式承诺哈希
   */
  static generateContactCommit(wechat: string, phone: string): string {
    const data = `${wechat}|${phone}`;
    return '0x' + CryptoJS.SHA256(data).toString();
  }

  /**
   * 计算预计获得的 DUST 数量
   */
  static calculateDustAmount(
    usdAmount: number,
    dustPrice: number,
    premiumBps: number
  ): bigint {
    // 计算实际价格（含溢价）
    const actualPrice = dustPrice * (1 + premiumBps / 10000);
    // 计算 DUST 数量（精度 10^12）
    const dustAmount = (usdAmount / actualPrice) * 1e12;
    return BigInt(Math.floor(dustAmount));
  }

  /**
   * 格式化 DUST 数量
   */
  static formatDustAmount(amount: bigint): string {
    return (Number(amount) / 1e12).toFixed(4);
  }

  /**
   * 格式化 USD 金额
   */
  static formatUsdAmount(amount: bigint): string {
    return (Number(amount) / 1e6).toFixed(2);
  }
}

// 导出单例
export const tradingService = new TradingService();
