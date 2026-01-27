/**
 * 星尘玄鉴 - 交易安全服务
 * 
 * 功能：
 * - 交易详情解析
 * - 交易模拟（预估 Gas 费和结果）
 * - 风险评估
 * - 交易白名单管理
 */

import { getApi } from '@/lib/api';
import type { SubmittableExtrinsic } from '@polkadot/api/types';
import type { TransactionDetails, TransactionParam, SimulationResult } from '@/components/TransactionConfirmDialog';

// ==================== 类型定义 ====================

/**
 * 交易类型配置
 */
interface TransactionTypeConfig {
  type: string;
  description: string;
  riskLevel: 'low' | 'medium' | 'high';
  riskWarnings?: string[];
  /** 是否涉及资金转移 */
  involvesTransfer?: boolean;
  /** 是否需要额外确认 */
  requiresExtraConfirmation?: boolean;
}

/**
 * 白名单条目
 */
interface WhitelistEntry {
  module: string;
  method: string;
  /** 过期时间（毫秒时间戳），null 表示永不过期 */
  expiresAt: number | null;
  /** 添加时间 */
  addedAt: number;
}

// ==================== 交易类型配置 ====================

const TRANSACTION_TYPES: Record<string, Record<string, TransactionTypeConfig>> = {
  // 占卜市场
  divinationMarket: {
    registerProvider: {
      type: '注册解卦师',
      description: '注册成为解卦师，开始提供占卜服务',
      riskLevel: 'low',
    },
    updateProviderProfile: {
      type: '更新资料',
      description: '更新解卦师个人资料',
      riskLevel: 'low',
    },
    createPackage: {
      type: '创建服务套餐',
      description: '创建新的占卜服务套餐',
      riskLevel: 'low',
    },
    createOrder: {
      type: '创建订单',
      description: '向解卦师下单购买占卜服务',
      riskLevel: 'medium',
      involvesTransfer: true,
      riskWarnings: ['此操作将锁定订单金额'],
    },
    cancelOrder: {
      type: '取消订单',
      description: '取消未完成的订单',
      riskLevel: 'low',
    },
    completeOrder: {
      type: '完成订单',
      description: '提交解卦结果，完成订单',
      riskLevel: 'low',
    },
    submitReview: {
      type: '提交评价',
      description: '对解卦师服务进行评价',
      riskLevel: 'low',
    },
    withdraw: {
      type: '提现',
      description: '提取账户余额',
      riskLevel: 'high',
      involvesTransfer: true,
      requiresExtraConfirmation: true,
      riskWarnings: ['请确认提现金额正确', '提现后资金将转入您的主账户'],
    },
    tip: {
      type: '打赏',
      description: '向解卦师发送打赏',
      riskLevel: 'medium',
      involvesTransfer: true,
    },
  },
  // OTC 交易
  otcOrder: {
    createFirstPurchase: {
      type: '首购订单',
      description: '创建首次购买订单',
      riskLevel: 'medium',
      involvesTransfer: true,
      riskWarnings: ['首购订单有特殊限制'],
    },
    createOrder: {
      type: '创建 OTC 订单',
      description: '创建 OTC 交易订单',
      riskLevel: 'medium',
      involvesTransfer: true,
    },
    markPaid: {
      type: '标记已付款',
      description: '确认已完成链下付款',
      riskLevel: 'medium',
      riskWarnings: ['请确保已完成实际付款'],
    },
    cancelOrder: {
      type: '取消订单',
      description: '取消 OTC 订单',
      riskLevel: 'low',
    },
    initiateDispute: {
      type: '发起争议',
      description: '对订单发起争议仲裁',
      riskLevel: 'medium',
      riskWarnings: ['争议将进入仲裁流程'],
    },
  },
  // 做市商
  maker: {
    applyMaker: {
      type: '申请做市商',
      description: '申请成为做市商',
      riskLevel: 'high',
      involvesTransfer: true,
      requiresExtraConfirmation: true,
      riskWarnings: ['需要质押保证金', '请仔细阅读做市商协议'],
    },
    updatePremium: {
      type: '更新溢价',
      description: '更新买卖溢价设置',
      riskLevel: 'low',
    },
    withdrawDeposit: {
      type: '提取保证金',
      description: '提取做市商保证金',
      riskLevel: 'high',
      involvesTransfer: true,
      requiresExtraConfirmation: true,
      riskWarnings: ['提取保证金将影响做市商状态'],
    },
  },
  // 兑换
  swap: {
    makerSwap: {
      type: '做市商兑换',
      description: '通过做市商兑换 DUST',
      riskLevel: 'medium',
      involvesTransfer: true,
    },
    cancelSwap: {
      type: '取消兑换',
      description: '取消兑换订单',
      riskLevel: 'low',
    },
    disputeSwap: {
      type: '兑换争议',
      description: '对兑换订单发起争议',
      riskLevel: 'medium',
    },
  },
  // 系统转账
  balances: {
    transfer: {
      type: '转账',
      description: '向其他账户转账 DUST',
      riskLevel: 'high',
      involvesTransfer: true,
      requiresExtraConfirmation: true,
      riskWarnings: ['请仔细核对接收地址', '转账不可撤销'],
    },
    transferKeepAlive: {
      type: '转账（保留最小余额）',
      description: '转账并保留账户最小余额',
      riskLevel: 'high',
      involvesTransfer: true,
      requiresExtraConfirmation: true,
      riskWarnings: ['请仔细核对接收地址', '转账不可撤销'],
    },
  },
};

// ==================== 白名单存储 ====================

const WHITELIST_STORAGE_KEY = 'stardust_tx_whitelist';

/**
 * 加载白名单
 */
function loadWhitelist(): WhitelistEntry[] {
  try {
    const data = localStorage.getItem(WHITELIST_STORAGE_KEY);
    if (!data) return [];
    
    const entries: WhitelistEntry[] = JSON.parse(data);
    const now = Date.now();
    
    // 过滤掉已过期的条目
    return entries.filter(entry => 
      entry.expiresAt === null || entry.expiresAt > now
    );
  } catch {
    return [];
  }
}

/**
 * 保存白名单
 */
function saveWhitelist(entries: WhitelistEntry[]): void {
  localStorage.setItem(WHITELIST_STORAGE_KEY, JSON.stringify(entries));
}

// ==================== 服务类 ====================

/**
 * 交易安全服务
 */
class TransactionSecurityService {
  private whitelist: WhitelistEntry[] = [];

  constructor() {
    this.whitelist = loadWhitelist();
  }

  /**
   * 解析交易详情
   */
  parseTransaction(
    tx: SubmittableExtrinsic<'promise'>,
    extraParams?: Record<string, string>
  ): TransactionDetails {
    const { method, section } = tx.method;
    const methodName = method;
    const moduleName = section;

    // 获取交易类型配置
    const config = TRANSACTION_TYPES[moduleName]?.[methodName] || {
      type: `${moduleName}.${methodName}`,
      description: '执行链上操作',
      riskLevel: 'medium' as const,
    };

    // 解析参数
    const params: TransactionParam[] = [];
    const args = tx.method.args;
    const argNames = tx.meta.args;

    argNames.forEach((argMeta, index) => {
      const argName = argMeta.name.toString();
      const argValue = args[index];
      
      if (argValue !== undefined) {
        let displayValue = '';
        
        try {
          // 尝试转换为人类可读格式
          const humanValue = argValue.toHuman();
          displayValue = typeof humanValue === 'object' 
            ? JSON.stringify(humanValue) 
            : String(humanValue);
        } catch {
          displayValue = String(argValue);
        }

        params.push({
          name: this.formatParamName(argName),
          value: displayValue,
          sensitive: this.isSensitiveParam(argName),
        });
      }
    });

    // 添加额外参数
    if (extraParams) {
      Object.entries(extraParams).forEach(([name, value]) => {
        params.push({ name, value });
      });
    }

    // 提取金额和接收方
    let amount: string | undefined;
    let recipient: string | undefined;

    if (config.involvesTransfer) {
      // 尝试从参数中提取金额
      const amountParam = params.find(p => 
        ['amount', 'value', 'price', 'dustAmount'].includes(p.name.toLowerCase())
      );
      if (amountParam) {
        amount = this.formatAmount(amountParam.value);
      }

      // 尝试从参数中提取接收方
      const recipientParam = params.find(p => 
        ['dest', 'to', 'recipient', 'providerId', 'target'].includes(p.name.toLowerCase())
      );
      if (recipientParam) {
        recipient = recipientParam.value;
      }
    }

    return {
      type: config.type,
      description: config.description,
      module: moduleName,
      method: methodName,
      params,
      amount,
      recipient,
      riskLevel: config.riskLevel,
      riskWarnings: config.riskWarnings,
    };
  }

  /**
   * 模拟交易
   */
  async simulateTransaction(
    tx: SubmittableExtrinsic<'promise'>,
    senderAddress: string
  ): Promise<SimulationResult> {
    try {
      const api = getApi();

      // 获取交易费用信息
      const paymentInfo = await tx.paymentInfo(senderAddress);
      const estimatedFee = this.formatAmount(paymentInfo.partialFee.toString());

      // 尝试进行 dry run（如果支持）
      let resultDescription: string | undefined;
      
      try {
        // 某些链支持 dry run
        const dryRunResult = await (api.rpc as any).system?.dryRun?.(
          tx.toHex(),
          senderAddress
        );
        
        if (dryRunResult) {
          const result = dryRunResult.toHuman();
          if (result?.Ok) {
            resultDescription = '交易模拟成功';
          } else if (result?.Err) {
            return {
              success: false,
              estimatedFee,
              error: `模拟失败: ${JSON.stringify(result.Err)}`,
            };
          }
        }
      } catch {
        // dry run 不可用，跳过
      }

      return {
        success: true,
        estimatedFee,
        resultDescription: resultDescription || '预估成功',
      };
    } catch (error) {
      return {
        success: false,
        estimatedFee: '未知',
        error: error instanceof Error ? error.message : '模拟失败',
      };
    }
  }

  /**
   * 检查交易是否在白名单中
   */
  isWhitelisted(module: string, method: string): boolean {
    // 刷新白名单（移除过期条目）
    this.whitelist = loadWhitelist();
    
    return this.whitelist.some(
      entry => entry.module === module && entry.method === method
    );
  }

  /**
   * 添加到白名单
   */
  addToWhitelist(
    module: string,
    method: string,
    durationMs?: number
  ): void {
    // 移除已存在的相同条目
    this.whitelist = this.whitelist.filter(
      entry => !(entry.module === module && entry.method === method)
    );

    // 添加新条目
    this.whitelist.push({
      module,
      method,
      expiresAt: durationMs ? Date.now() + durationMs : null,
      addedAt: Date.now(),
    });

    saveWhitelist(this.whitelist);
  }

  /**
   * 从白名单移除
   */
  removeFromWhitelist(module: string, method: string): void {
    this.whitelist = this.whitelist.filter(
      entry => !(entry.module === module && entry.method === method)
    );
    saveWhitelist(this.whitelist);
  }

  /**
   * 获取白名单列表
   */
  getWhitelist(): WhitelistEntry[] {
    this.whitelist = loadWhitelist();
    return [...this.whitelist];
  }

  /**
   * 清空白名单
   */
  clearWhitelist(): void {
    this.whitelist = [];
    saveWhitelist([]);
  }

  /**
   * 检查是否需要额外确认
   */
  requiresExtraConfirmation(module: string, method: string): boolean {
    const config = TRANSACTION_TYPES[module]?.[method];
    return config?.requiresExtraConfirmation ?? false;
  }

  /**
   * 获取风险等级
   */
  getRiskLevel(module: string, method: string): 'low' | 'medium' | 'high' {
    const config = TRANSACTION_TYPES[module]?.[method];
    return config?.riskLevel ?? 'medium';
  }

  // ==================== 私有方法 ====================

  /**
   * 格式化参数名称
   */
  private formatParamName(name: string): string {
    // 驼峰转空格分隔
    return name
      .replace(/([A-Z])/g, ' $1')
      .replace(/^./, str => str.toUpperCase())
      .trim();
  }

  /**
   * 判断是否为敏感参数
   */
  private isSensitiveParam(name: string): boolean {
    const sensitiveNames = [
      'address', 'dest', 'to', 'recipient', 'target',
      'providerId', 'clientId', 'owner', 'account',
    ];
    return sensitiveNames.some(s => name.toLowerCase().includes(s.toLowerCase()));
  }

  /**
   * 格式化金额
   */
  private formatAmount(value: string): string {
    try {
      // 假设精度为 12 位（DUST）
      const num = BigInt(value.replace(/,/g, ''));
      const whole = num / BigInt(1e12);
      const fraction = num % BigInt(1e12);
      const fractionStr = fraction.toString().padStart(12, '0').slice(0, 4);
      return `${whole}.${fractionStr}`;
    } catch {
      return value;
    }
  }
}

// 导出单例
export const transactionSecurityService = new TransactionSecurityService();

// 导出便捷方法
export const parseTransaction = (
  tx: SubmittableExtrinsic<'promise'>,
  extraParams?: Record<string, string>
) => transactionSecurityService.parseTransaction(tx, extraParams);

export const simulateTransaction = (
  tx: SubmittableExtrinsic<'promise'>,
  senderAddress: string
) => transactionSecurityService.simulateTransaction(tx, senderAddress);

export const isWhitelisted = (module: string, method: string) =>
  transactionSecurityService.isWhitelisted(module, method);

export const addToWhitelist = (
  module: string,
  method: string,
  durationMs?: number
) => transactionSecurityService.addToWhitelist(module, method, durationMs);
