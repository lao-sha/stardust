/**
 * 星尘玄鉴 - 交易状态管理
 * 管理做市商、订单、价格、信用等交易相关状态
 */

import { create } from 'zustand';
import { tradingService } from '@/services/trading.service';
import { useWalletStore } from './wallet.store';

// ===== 类型定义 =====

/**
 * 做市商信息
 */
export interface Maker {
  id: number;
  owner: string;
  tronAddress: string;
  buyPremiumBps: number;   // 买入溢价 (基点, -500~500)
  sellPremiumBps: number;  // 卖出溢价
  minAmount: bigint;
  servicePaused: boolean;
  usersServed: number;
  maskedFullName: string;
  wechatId: string;
  rating: number;          // 前端计算的评分
  creditScore?: number;    // 信用分 (800-1000)
  creditLevel?: string;    // 信用等级
  avgResponseTime?: number; // 平均响应时间(秒)
  completionRate?: number;  // 完成率(%)
  isOnline?: boolean;       // 在线状态
}

/**
 * 订单状态
 */
export enum OrderState {
  Created = 'Created',
  Paid = 'Paid',
  Released = 'Released',
  Cancelled = 'Cancelled',
  Disputed = 'Disputed',
  Resolved = 'Resolved',
  Expired = 'Expired',
}

/**
 * 订单信息
 */
export interface Order {
  id: number;
  makerId: number;
  maker: string;
  taker: string;
  price: bigint;           // 单价 (USDT/DUST, 精度 10^6)
  qty: bigint;             // DUST 数量
  amount: bigint;          // USDT 金额
  createdAt: number;       // 创建时间 (毫秒)
  expireAt: number;        // 超时时间 (毫秒)
  makerTronAddress: string;
  state: OrderState;
  isFirstPurchase: boolean;
  tronTxHash?: string;     // TRON 交易哈希
}

/**
 * 市场统计
 */
export interface MarketStats {
  otcPrice: number;        // OTC 均价
  bridgePrice: number;     // Bridge 均价
  weightedPrice: number;   // 加权平均价
  simpleAvgPrice: number;  // 简单平均价
  otcVolume: bigint;       // OTC 交易量
  bridgeVolume: bigint;    // Bridge 交易量
  totalVolume: bigint;     // 总交易量
  priceChange24h?: number; // 24h 涨跌幅
}

/**
 * 买家信用信息
 */
export interface BuyerCreditInfo {
  riskScore: number;       // 风险分 (0-1000)
  level: string;           // 信用等级
  maxAmount: number;       // 最大交易金额 (USD)
  concurrentOrders: number; // 当前并发订单数
  maxConcurrentOrders: number; // 最大并发订单数
  completedOrders: number; // 已完成订单数
  trend: 'up' | 'down' | 'stable'; // 信用趋势
}

/**
 * KYC 验证结果
 */
export enum KycStatus {
  Passed = 'Passed',
  Failed = 'Failed',
  Skipped = 'Skipped',
  Exempted = 'Exempted',
}

/**
 * KYC 失败原因
 */
export enum KycFailureReason {
  IdentityNotSet = 'IdentityNotSet',
  NoValidJudgement = 'NoValidJudgement',
  InsufficientLevel = 'InsufficientLevel',
  QualityIssue = 'QualityIssue',
}

// ===== Store 定义 =====

interface TradingState {
  // 做市商
  makers: Maker[];
  loadingMakers: boolean;
  selectedMaker: Maker | null;
  makerError: string | null;

  // 订单
  currentOrder: Order | null;
  orderHistory: Order[];
  loadingOrder: boolean;
  orderError: string | null;

  // 首购状态
  isFirstPurchase: boolean;
  hasCompletedFirstPurchase: boolean;
  checkingFirstPurchase: boolean;

  // 价格
  dustPrice: number;
  marketStats: MarketStats | null;
  loadingPrice: boolean;

  // 信用
  buyerCredit: BuyerCreditInfo | null;
  loadingCredit: boolean;

  // KYC
  kycStatus: KycStatus | null;
  kycFailureReason: KycFailureReason | null;
  checkingKyc: boolean;

  // Actions - 做市商
  fetchMakers: () => Promise<void>;
  selectMaker: (makerId: number) => void;
  clearSelectedMaker: () => void;

  // Actions - 订单
  createFirstPurchase: (makerId: number, paymentCommit: string, contactCommit: string, onStatusChange?: (status: string) => void) => Promise<number>;
  createOrder: (makerId: number, dustAmount: bigint, paymentCommit: string, contactCommit: string, onStatusChange?: (status: string) => void) => Promise<number>;
  markPaid: (orderId: number, tronTxHash?: string, onStatusChange?: (status: string) => void) => Promise<void>;
  cancelOrder: (orderId: number, onStatusChange?: (status: string) => void) => Promise<void>;
  dispute: (orderId: number, reason: string, evidenceCid?: string, onStatusChange?: (status: string) => void) => Promise<void>;
  fetchOrder: (orderId: number) => Promise<void>;
  fetchOrderHistory: () => Promise<void>;
  subscribeToOrder: (orderId: number) => () => void;

  // Actions - 价格
  fetchMarketStats: () => Promise<void>;
  fetchDustPrice: () => Promise<void>;

  // Actions - 信用
  fetchBuyerCredit: () => Promise<void>;
  checkFirstPurchaseStatus: () => Promise<void>;

  // Actions - KYC
  checkKycStatus: () => Promise<void>;

  // Actions - 清理
  clearError: () => void;
  reset: () => void;
}

/**
 * Trading Store
 */
export const useTradingStore = create<TradingState>()((set, get) => ({
  // 初始状态
  makers: [],
  loadingMakers: false,
  selectedMaker: null,
  makerError: null,

  currentOrder: null,
  orderHistory: [],
  loadingOrder: false,
  orderError: null,

  isFirstPurchase: true,
  hasCompletedFirstPurchase: false,
  checkingFirstPurchase: false,

  dustPrice: 0.1,  // 默认初始价格 0.1 USDT/DUST
  marketStats: null,
  loadingPrice: false,

  buyerCredit: null,
  loadingCredit: false,

  kycStatus: null,
  kycFailureReason: null,
  checkingKyc: false,

  // ===== 做市商相关 =====

  /**
   * 获取做市商列表
   */
  fetchMakers: async () => {
    try {
      set({ loadingMakers: true, makerError: null });

      // 调用 API 获取做市商列表
      const makers = await tradingService.getMakers();

      set({ makers, loadingMakers: false });
    } catch (error) {
      console.error('[Trading] Fetch makers error:', error);
      set({
        makerError: '获取做市商列表失败',
        loadingMakers: false,
      });
    }
  },

  /**
   * 选择做市商
   */
  selectMaker: (makerId: number) => {
    const maker = get().makers.find(m => m.id === makerId);
    if (maker) {
      set({ selectedMaker: maker });
    }
  },

  /**
   * 清除选中的做市商
   */
  clearSelectedMaker: () => {
    set({ selectedMaker: null });
  },

  // ===== 订单相关 =====

  /**
   * 创建首购订单
   */
  createFirstPurchase: async (
    makerId: number,
    paymentCommit: string,
    contactCommit: string,
    onStatusChange?: (status: string) => void
  ): Promise<number> => {
    try {
      set({ loadingOrder: true, orderError: null });

      // 获取当前账户
      const address = useWalletStore.getState().address;
      if (!address) {
        throw new Error('No account selected');
      }

      // 调用 API 创建首购订单
      const orderId = await tradingService.createFirstPurchase(
        address,
        makerId,
        paymentCommit,
        contactCommit,
        onStatusChange
      );

      console.log('[Trading] First purchase created:', orderId);
      set({ loadingOrder: false });

      return orderId;
    } catch (error) {
      console.error('[Trading] Create first purchase error:', error);
      const errorMessage = error instanceof Error ? error.message : '创建首购订单失败';
      set({
        orderError: errorMessage,
        loadingOrder: false,
      });
      throw error;
    }
  },

  /**
   * 创建普通订单
   */
  createOrder: async (
    makerId: number,
    dustAmount: bigint,
    paymentCommit: string,
    contactCommit: string,
    onStatusChange?: (status: string) => void
  ): Promise<number> => {
    try {
      set({ loadingOrder: true, orderError: null });

      // 获取当前账户
      const address = useWalletStore.getState().address;
      if (!address) {
        throw new Error('No account selected');
      }

      // 调用 API 创建订单
      const orderId = await tradingService.createOrder(
        address,
        makerId,
        dustAmount,
        paymentCommit,
        contactCommit,
        onStatusChange
      );

      console.log('[Trading] Order created:', orderId);
      set({ loadingOrder: false });

      return orderId;
    } catch (error) {
      console.error('[Trading] Create order error:', error);
      const errorMessage = error instanceof Error ? error.message : '创建订单失败';
      set({
        orderError: errorMessage,
        loadingOrder: false,
      });
      throw error;
    }
  },

  /**
   * 标记已付款
   */
  markPaid: async (
    orderId: number,
    tronTxHash?: string,
    onStatusChange?: (status: string) => void
  ) => {
    try {
      set({ loadingOrder: true, orderError: null });

      // 获取当前账户
      const address = useWalletStore.getState().address;
      if (!address) {
        throw new Error('No account selected');
      }

      // 调用 API 标记已付款
      await tradingService.markPaid(
        address,
        orderId,
        tronTxHash,
        onStatusChange
      );

      console.log('[Trading] Order marked as paid:', orderId);
      set({ loadingOrder: false });
    } catch (error) {
      console.error('[Trading] Mark paid error:', error);
      const errorMessage = error instanceof Error ? error.message : '标记付款失败';
      set({
        orderError: errorMessage,
        loadingOrder: false,
      });
      throw error;
    }
  },

  /**
   * 取消订单
   */
  cancelOrder: async (
    orderId: number,
    onStatusChange?: (status: string) => void
  ) => {
    try {
      set({ loadingOrder: true, orderError: null });

      // 获取当前账户
      const address = useWalletStore.getState().address;
      if (!address) {
        throw new Error('No account selected');
      }

      // 调用 API 取消订单
      await tradingService.cancelOrder(
        address,
        orderId,
        onStatusChange
      );

      console.log('[Trading] Order cancelled:', orderId);
      set({ loadingOrder: false });
    } catch (error) {
      console.error('[Trading] Cancel order error:', error);
      const errorMessage = error instanceof Error ? error.message : '取消订单失败';
      set({
        orderError: errorMessage,
        loadingOrder: false,
      });
      throw error;
    }
  },

  /**
   * 申请仲裁
   */
  dispute: async (
    orderId: number,
    reason: string,
    evidenceCid?: string,
    onStatusChange?: (status: string) => void
  ) => {
    try {
      set({ loadingOrder: true, orderError: null });

      // 获取当前账户
      const address = useWalletStore.getState().address;
      if (!address) {
        throw new Error('No account selected');
      }

      // 调用 API 申请仲裁
      await tradingService.dispute(
        address,
        orderId,
        reason,
        evidenceCid,
        onStatusChange
      );

      console.log('[Trading] Dispute submitted:', orderId);
      set({ loadingOrder: false });
    } catch (error) {
      console.error('[Trading] Dispute error:', error);
      const errorMessage = error instanceof Error ? error.message : '申请仲裁失败';
      set({
        orderError: errorMessage,
        loadingOrder: false,
      });
      throw error;
    }
  },

  /**
   * 获取订单详情
   */
  fetchOrder: async (orderId: number) => {
    try {
      set({ loadingOrder: true, orderError: null });

      const order = await tradingService.getOrder(orderId);

      if (!order) {
        throw new Error('订单不存在');
      }

      set({ currentOrder: order, loadingOrder: false });
    } catch (error) {
      console.error('[Trading] Fetch order error:', error);
      const errorMessage = error instanceof Error ? error.message : '获取订单详情失败';
      set({
        orderError: errorMessage,
        loadingOrder: false,
      });
    }
  },

  /**
   * 获取订单历史
   */
  fetchOrderHistory: async () => {
    try {
      const address = useWalletStore.getState().address;
      if (!address) {
        set({ orderHistory: [] });
        return;
      }

      const orders = await tradingService.getOrderHistory(address);
      set({ orderHistory: orders });
    } catch (error) {
      console.error('[Trading] Fetch order history error:', error);
      set({ orderHistory: [] });
    }
  },

  /**
   * 订阅订单状态
   */
  subscribeToOrder: (orderId: number) => {
    let unsubscribe: (() => void) | null = null;
    let isCancelled = false;

    // 异步订阅
    tradingService.subscribeToOrder(orderId, (order) => {
      if (!isCancelled) {
        set({ currentOrder: order });
      }
    }).then((unsub) => {
      if (!isCancelled) {
        unsubscribe = unsub;
      } else {
        // 如果已经取消，立即清理
        unsub();
      }
    }).catch((error) => {
      console.error('[Trading] Subscribe to order error:', error);
    });

    // 返回取消订阅函数
    return () => {
      isCancelled = true;
      if (unsubscribe) {
        unsubscribe();
      }
      console.log('[Trading] Unsubscribe from order:', orderId);
    };
  },

  // ===== 价格相关 =====

  /**
   * 获取市场统计
   */
  fetchMarketStats: async () => {
    try {
      set({ loadingPrice: true });

      const stats = await tradingService.getMarketStats();

      set({ 
        marketStats: stats, 
        dustPrice: stats.weightedPrice || 0.1, 
        loadingPrice: false 
      });
    } catch (error) {
      console.error('[Trading] Fetch market stats error:', error);
      // 设置默认价格
      set({ 
        dustPrice: 0.1,
        marketStats: {
          otcPrice: 0.1,
          bridgePrice: 0.1,
          weightedPrice: 0.1,
          simpleAvgPrice: 0.1,
          otcVolume: BigInt(0),
          bridgeVolume: BigInt(0),
          totalVolume: BigInt(0),
        },
        loadingPrice: false 
      });
    }
  },

  /**
   * 获取 DUST 价格
   */
  fetchDustPrice: async () => {
    try {
      set({ loadingPrice: true });

      const price = await tradingService.getDustPrice();

      set({ dustPrice: price || 0.1, loadingPrice: false });
    } catch (error) {
      console.error('[Trading] Fetch dust price error:', error);
      set({ dustPrice: 0.1, loadingPrice: false });
    }
  },

  // ===== 信用相关 =====

  /**
   * 获取买家信用信息
   */
  fetchBuyerCredit: async () => {
    try {
      set({ loadingCredit: true });

      const address = useWalletStore.getState().address;
      if (!address) {
        set({ loadingCredit: false });
        return;
      }

      const credit = await tradingService.getBuyerCredit(address);

      set({ buyerCredit: credit, loadingCredit: false });
    } catch (error) {
      console.error('[Trading] Fetch buyer credit error:', error);
      set({ loadingCredit: false });
    }
  },

  /**
   * 检查首购状态
   */
  checkFirstPurchaseStatus: async () => {
    try {
      set({ checkingFirstPurchase: true });

      const address = useWalletStore.getState().address;
      if (!address) {
        set({ checkingFirstPurchase: false });
        return;
      }

      const hasCompleted = await tradingService.hasCompletedFirstPurchase(address);

      set({
        hasCompletedFirstPurchase: hasCompleted,
        isFirstPurchase: !hasCompleted,
        checkingFirstPurchase: false,
      });
    } catch (error) {
      console.error('[Trading] Check first purchase error:', error);
      set({ checkingFirstPurchase: false });
    }
  },

  // ===== KYC 相关 =====

  /**
   * 检查 KYC 状态
   */
  checkKycStatus: async () => {
    try {
      set({ checkingKyc: true });

      const address = useWalletStore.getState().address;
      if (!address) {
        set({ checkingKyc: false });
        return;
      }

      const result = await tradingService.checkKycStatus(address);

      set({
        kycStatus: result.status as KycStatus,
        kycFailureReason: result.failureReason as KycFailureReason | null,
        checkingKyc: false,
      });
    } catch (error) {
      console.error('[Trading] Check KYC error:', error);
      set({ checkingKyc: false });
    }
  },

  // ===== 工具方法 =====

  /**
   * 清除错误
   */
  clearError: () => {
    set({
      makerError: null,
      orderError: null,
    });
  },

  /**
   * 重置状态
   */
  reset: () => {
    set({
      makers: [],
      selectedMaker: null,
      currentOrder: null,
      orderHistory: [],
      marketStats: null,
      buyerCredit: null,
      kycStatus: null,
      kycFailureReason: null,
    });
  },
}));
