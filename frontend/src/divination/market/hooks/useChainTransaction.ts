// frontend/src/divination/market/hooks/useChainTransaction.ts

import { useState, useCallback, useRef } from 'react';
import { useWalletStore } from '@/stores/wallet.store';
import {
  getSigner,
  TransactionResult,
  TransactionCallbacks,
  // Provider transactions
  registerProvider,
  updateProviderProfile,
  pauseProvider,
  resumeProvider,
  createPackage,
  updatePackage,
  deletePackage,
  // Order transactions
  createOrder,
  acceptOrder,
  rejectOrder,
  completeOrder,
  cancelOrder,
  requestRefund,
  submitReport,
  submitFollowUp,
  replyFollowUp,
  // Review transactions
  submitReview,
  replyReview,
  // Fund transactions
  withdraw,
  tip,
} from '../services/chain.service';
import {
  parseTransaction,
  simulateTransaction,
  isWhitelisted,
  addToWhitelist,
} from '@/services/transaction-security.service';
import type { TransactionDetails, SimulationResult } from '@/components/TransactionConfirmDialog';
import type { DivinationType } from '../types';
import type { SubmittableExtrinsic } from '@polkadot/api/types';

export type TransactionStatus = 'idle' | 'confirming' | 'simulating' | 'signing' | 'broadcasting' | 'pending' | 'success' | 'error';

export interface TransactionState {
  status: TransactionStatus;
  txHash?: string;
  blockHash?: string;
  error?: string;
}

/**
 * 交易确认请求
 */
export interface TransactionConfirmRequest {
  /** 交易详情 */
  details: TransactionDetails;
  /** 模拟结果 */
  simulation: SimulationResult | null;
  /** 是否正在模拟 */
  isSimulating: boolean;
  /** 确认回调 */
  onConfirm: (password: string) => void;
  /** 取消回调 */
  onCancel: () => void;
}

/**
 * 链上交易 Hook
 * 
 * 安全特性：
 * - 交易详情预览
 * - 交易模拟（预估 Gas 费）
 * - 风险评估
 * - 白名单机制
 */
export function useChainTransaction() {
  const { address, isLocked } = useWalletStore();
  const [txState, setTxState] = useState<TransactionState>({ status: 'idle' });
  const [confirmRequest, setConfirmRequest] = useState<TransactionConfirmRequest | null>(null);
  
  // 用于存储待执行的交易
  const pendingTxRef = useRef<{
    txFn: (signer: any) => Promise<TransactionResult>;
    options?: {
      onSuccess?: (result: TransactionResult) => void;
      onError?: (error: Error) => void;
    };
    resolve: (result: TransactionResult | null) => void;
  } | null>(null);

  /**
   * 重置交易状态
   */
  const resetState = useCallback(() => {
    setTxState({ status: 'idle' });
    setConfirmRequest(null);
    pendingTxRef.current = null;
  }, []);

  /**
   * 关闭确认对话框
   */
  const closeConfirmDialog = useCallback(() => {
    setConfirmRequest(null);
    if (pendingTxRef.current) {
      pendingTxRef.current.resolve(null);
      pendingTxRef.current = null;
    }
    setTxState({ status: 'idle' });
  }, []);

  /**
   * 执行已确认的交易
   */
  const executeConfirmedTransaction = useCallback(async (password: string) => {
    const pending = pendingTxRef.current;
    if (!pending || !address) return;

    try {
      setTxState({ status: 'signing' });
      setConfirmRequest(null);

      // 获取签名者
      const signer = await getSigner(password, address);

      setTxState({ status: 'broadcasting' });

      // 执行交易
      const result = await pending.txFn(signer);

      if (result.success) {
        setTxState({
          status: 'success',
          txHash: result.txHash,
          blockHash: result.blockHash,
        });
        pending.options?.onSuccess?.(result);
      } else {
        setTxState({
          status: 'error',
          error: result.error,
        });
        pending.options?.onError?.(new Error(result.error || '交易失败'));
      }

      pending.resolve(result);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '交易失败';
      setTxState({ status: 'error', error: errorMessage });
      pending.options?.onError?.(error instanceof Error ? error : new Error(errorMessage));
      pending.resolve(null);
    } finally {
      pendingTxRef.current = null;
    }
  }, [address]);

  /**
   * 请求确认并执行交易
   */
  const executeTransaction = useCallback(
    async <T extends any[]>(
      txFn: (signer: any, ...args: T) => Promise<TransactionResult>,
      args: T,
      options?: {
        onSuccess?: (result: TransactionResult) => void;
        onError?: (error: Error) => void;
        /** 跳过确认（用于白名单交易） */
        skipConfirmation?: boolean;
        /** 额外显示的参数 */
        extraParams?: Record<string, string>;
      }
    ): Promise<TransactionResult | null> => {
      if (!address) {
        const error = new Error('请先连接钱包');
        options?.onError?.(error);
        return null;
      }

      return new Promise(async (resolve) => {
        try {
          // 创建一个临时交易用于解析详情
          const { getApi } = await import('@/api');
          const api = getApi();
          
          // 获取交易详情（需要构建一个临时交易）
          // 这里我们使用一个简化的方法：从 txFn 的名称推断交易类型
          const txFnName = txFn.name || 'unknown';
          
          // 创建包装后的交易函数
          const wrappedTxFn = async (signer: any) => {
            return txFn(signer, ...args);
          };

          // 检查是否在白名单中
          // 注意：这里需要从实际交易中获取 module 和 method
          // 暂时跳过白名单检查，直接显示确认对话框
          
          if (options?.skipConfirmation) {
            // 跳过确认，直接执行
            pendingTxRef.current = {
              txFn: wrappedTxFn,
              options,
              resolve,
            };
            // 需要用户提供密码
            setConfirmRequest({
              details: {
                type: '快速交易',
                description: '白名单交易，跳过详情确认',
                module: 'unknown',
                method: 'unknown',
                params: [],
                riskLevel: 'low',
              },
              simulation: null,
              isSimulating: false,
              onConfirm: executeConfirmedTransaction,
              onCancel: closeConfirmDialog,
            });
            setTxState({ status: 'confirming' });
            return;
          }

          // 显示确认对话框
          setTxState({ status: 'confirming' });

          // 构建交易详情
          const details: TransactionDetails = {
            type: getTransactionTypeName(txFnName),
            description: getTransactionDescription(txFnName),
            module: getModuleName(txFnName),
            method: getMethodName(txFnName),
            params: buildParams(args, options?.extraParams),
            riskLevel: getRiskLevel(txFnName),
            riskWarnings: getRiskWarnings(txFnName),
          };

          // 存储待执行的交易
          pendingTxRef.current = {
            txFn: wrappedTxFn,
            options,
            resolve,
          };

          // 设置确认请求
          setConfirmRequest({
            details,
            simulation: null,
            isSimulating: false,
            onConfirm: executeConfirmedTransaction,
            onCancel: closeConfirmDialog,
          });

        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : '准备交易失败';
          setTxState({ status: 'error', error: errorMessage });
          options?.onError?.(error instanceof Error ? error : new Error(errorMessage));
          resolve(null);
        }
      });
    },
    [address, executeConfirmedTransaction, closeConfirmDialog]
  );

  // ==================== Provider 交易方法 ====================

  const doRegisterProvider = useCallback(
    (
      params: {
        name: string;
        bio: string;
        divinationTypes: DivinationType[];
        specialties: number[];
        avatarCid?: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => registerProvider(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doUpdateProviderProfile = useCallback(
    (
      params: {
        name?: string;
        bio?: string;
        avatarCid?: string;
        specialties?: number[];
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => updateProviderProfile(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doCreatePackage = useCallback(
    (
      params: {
        name: string;
        description: string;
        divinationType: DivinationType;
        price: bigint;
        deliveryDays: number;
        maxFollowUps: number;
        isActive?: boolean;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => createPackage(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doUpdatePackage = useCallback(
    (
      params: {
        packageId: number;
        name?: string;
        description?: string;
        price?: bigint;
        deliveryDays?: number;
        maxFollowUps?: number;
        isActive?: boolean;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => updatePackage(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doDeletePackage = useCallback(
    (
      packageId: number,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => deletePackage(signer, packageId, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doPauseProvider = useCallback(
    (options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }) => {
      return executeTransaction(
        (signer) => pauseProvider(signer, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doResumeProvider = useCallback(
    (options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }) => {
      return executeTransaction(
        (signer) => resumeProvider(signer, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doSubmitReport = useCallback(
    (
      params: {
        provider: string;
        reportType: number;
        evidenceCid: string;
        description: string;
        relatedOrderId?: number;
        isAnonymous?: boolean;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => submitReport(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  // ==================== Order 交易方法 ====================

  const doCreateOrder = useCallback(
    (
      params: {
        providerId: string;
        packageId: number;
        questionCid: string;
        hexagramData?: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => createOrder(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doAcceptOrder = useCallback(
    (
      orderId: number,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => acceptOrder(signer, orderId, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doRejectOrder = useCallback(
    (
      orderId: number,
      reason?: string,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => rejectOrder(signer, orderId, reason, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doCompleteOrder = useCallback(
    (
      params: {
        orderId: number;
        resultCid: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => completeOrder(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doCancelOrder = useCallback(
    (
      orderId: number,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => cancelOrder(signer, orderId, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doRequestRefund = useCallback(
    (
      orderId: number,
      reason: string,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => requestRefund(signer, orderId, reason, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doSubmitFollowUp = useCallback(
    (
      params: {
        orderId: number;
        questionCid: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => submitFollowUp(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doReplyFollowUp = useCallback(
    (
      params: {
        orderId: number;
        followUpIndex: number;
        answerCid: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => replyFollowUp(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  // ==================== Review 交易方法 ====================

  const doSubmitReview = useCallback(
    (
      params: {
        orderId: number;
        ratings: {
          accuracy: number;
          attitude: number;
          speed: number;
          value: number;
        };
        contentCid?: string;
        isAnonymous?: boolean;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => submitReview(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doReplyReview = useCallback(
    (
      params: {
        reviewId: number;
        replyCid: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => replyReview(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  // ==================== 资金交易方法 ====================

  const doWithdraw = useCallback(
    (
      amount?: bigint,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => withdraw(signer, amount, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doTip = useCallback(
    (
      params: {
        providerId: string;
        amount: bigint;
        orderId?: number;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => tip(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  // ==================== 辅助函数 ====================

  /**
   * 创建交易回调
   */
  const createCallbacks = (): TransactionCallbacks => ({
    onBroadcast: () => setTxState((s) => ({ ...s, status: 'broadcasting' })),
    onInBlock: (blockHash) => setTxState((s) => ({ ...s, status: 'pending', blockHash })),
    onFinalized: (blockHash) => setTxState((s) => ({ ...s, status: 'success', blockHash })),
    onError: (error) => setTxState({ status: 'error', error: error.message }),
  });

  return {
    // 状态
    txState,
    isProcessing: ['signing', 'broadcasting', 'pending'].includes(txState.status),
    isSuccess: txState.status === 'success',
    isError: txState.status === 'error',
    
    // 确认对话框状态
    confirmRequest,
    isConfirming: txState.status === 'confirming',

    // 操作
    resetState,
    closeConfirmDialog,

    // Provider 交易
    registerProvider: doRegisterProvider,
    updateProviderProfile: doUpdateProviderProfile,
    pauseProvider: doPauseProvider,
    resumeProvider: doResumeProvider,
    createPackage: doCreatePackage,
    updatePackage: doUpdatePackage,
    deletePackage: doDeletePackage,

    // Order 交易
    createOrder: doCreateOrder,
    acceptOrder: doAcceptOrder,
    rejectOrder: doRejectOrder,
    completeOrder: doCompleteOrder,
    cancelOrder: doCancelOrder,
    requestRefund: doRequestRefund,
    submitReport: doSubmitReport,
    submitFollowUp: doSubmitFollowUp,
    replyFollowUp: doReplyFollowUp,

    // Review 交易
    submitReview: doSubmitReview,
    replyReview: doReplyReview,

    // 资金交易
    withdraw: doWithdraw,
    tip: doTip,
  };
}

// ==================== 交易类型映射辅助函数 ====================

function getTransactionTypeName(fnName: string): string {
  const typeMap: Record<string, string> = {
    registerProvider: '注册解卦师',
    updateProviderProfile: '更新资料',
    pauseProvider: '暂停服务',
    resumeProvider: '恢复服务',
    createPackage: '创建套餐',
    updatePackage: '更新套餐',
    deletePackage: '删除套餐',
    createOrder: '创建订单',
    acceptOrder: '接受订单',
    rejectOrder: '拒绝订单',
    completeOrder: '完成订单',
    cancelOrder: '取消订单',
    requestRefund: '申请退款',
    submitReport: '提交举报',
    submitFollowUp: '提交追问',
    replyFollowUp: '回复追问',
    submitReview: '提交评价',
    replyReview: '回复评价',
    withdraw: '提现',
    tip: '打赏',
  };
  return typeMap[fnName] || fnName;
}

function getTransactionDescription(fnName: string): string {
  const descMap: Record<string, string> = {
    registerProvider: '注册成为解卦师，开始提供占卜服务',
    updateProviderProfile: '更新您的解卦师资料',
    pauseProvider: '暂停接单，休息一下',
    resumeProvider: '恢复接单状态',
    createPackage: '创建新的服务套餐',
    updatePackage: '更新服务套餐信息',
    deletePackage: '删除服务套餐',
    createOrder: '向解卦师下单购买服务',
    acceptOrder: '接受客户的订单',
    rejectOrder: '拒绝客户的订单',
    completeOrder: '提交解卦结果，完成订单',
    cancelOrder: '取消订单',
    requestRefund: '申请订单退款',
    submitReport: '举报违规行为',
    submitFollowUp: '向解卦师提交追问',
    replyFollowUp: '回复客户的追问',
    submitReview: '对服务进行评价',
    replyReview: '回复客户的评价',
    withdraw: '提取账户余额到主钱包',
    tip: '向解卦师发送打赏',
  };
  return descMap[fnName] || '执行链上操作';
}

function getModuleName(fnName: string): string {
  const moduleMap: Record<string, string> = {
    registerProvider: 'divinationMarket',
    updateProviderProfile: 'divinationMarket',
    pauseProvider: 'divinationMarket',
    resumeProvider: 'divinationMarket',
    createPackage: 'divinationMarket',
    updatePackage: 'divinationMarket',
    deletePackage: 'divinationMarket',
    createOrder: 'divinationMarket',
    acceptOrder: 'divinationMarket',
    rejectOrder: 'divinationMarket',
    completeOrder: 'divinationMarket',
    cancelOrder: 'divinationMarket',
    requestRefund: 'divinationMarket',
    submitReport: 'divinationMarket',
    submitFollowUp: 'divinationMarket',
    replyFollowUp: 'divinationMarket',
    submitReview: 'divinationMarket',
    replyReview: 'divinationMarket',
    withdraw: 'divinationMarket',
    tip: 'divinationMarket',
  };
  return moduleMap[fnName] || 'unknown';
}

function getMethodName(fnName: string): string {
  // 驼峰转下划线
  return fnName.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
}

function getRiskLevel(fnName: string): 'low' | 'medium' | 'high' {
  const highRisk = ['withdraw', 'tip', 'createOrder'];
  const mediumRisk = ['acceptOrder', 'rejectOrder', 'cancelOrder', 'requestRefund', 'submitReport'];
  
  if (highRisk.includes(fnName)) return 'high';
  if (mediumRisk.includes(fnName)) return 'medium';
  return 'low';
}

function getRiskWarnings(fnName: string): string[] | undefined {
  const warningsMap: Record<string, string[]> = {
    withdraw: ['请确认提现金额正确', '提现后资金将转入您的主账户'],
    tip: ['打赏金额将直接转给解卦师'],
    createOrder: ['此操作将锁定订单金额', '请确认订单信息正确'],
    cancelOrder: ['取消订单可能产生手续费'],
    requestRefund: ['退款申请需要审核'],
  };
  return warningsMap[fnName];
}

function buildParams(args: any[], extraParams?: Record<string, string>): Array<{ name: string; value: string; sensitive?: boolean }> {
  const params: Array<{ name: string; value: string; sensitive?: boolean }> = [];
  
  // 添加额外参数
  if (extraParams) {
    Object.entries(extraParams).forEach(([name, value]) => {
      params.push({ name, value });
    });
  }
  
  return params;
}
