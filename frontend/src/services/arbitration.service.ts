/**
 * 仲裁服务
 * 处理争议创建、证据提交、仲裁、申诉等
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
 * 争议状态
 */
export enum DisputeStatus {
  Pending = 'Pending',         // 待处理
  UnderReview = 'UnderReview', // 审核中
  Resolved = 'Resolved',       // 已解决
  Appealed = 'Appealed',       // 已申诉
  Closed = 'Closed',           // 已关闭
}

/**
 * 争议类型
 */
export enum DisputeType {
  Order = 'Order',           // 订单争议
  Swap = 'Swap',             // 兑换争议
  Service = 'Service',       // 服务争议
  Other = 'Other',           // 其他
}

/**
 * 争议信息
 */
export interface Dispute {
  id: number;
  disputeType: DisputeType;
  relatedId: number;         // 相关订单/兑换ID
  plaintiff: string;         // 原告
  defendant: string;         // 被告
  reason: string;
  status: DisputeStatus;
  createdAt: number;
  resolvedAt?: number;
  resolution?: string;
  evidences: Evidence[];
}

/**
 * 证据信息
 */
export interface Evidence {
  id: number;
  disputeId: number;
  submitter: string;
  evidenceCid: string;
  description: string;
  submittedAt: number;
}

/**
 * 仲裁结果
 */
export interface ArbitrationResult {
  disputeId: number;
  winner: string;
  loser: string;
  compensation: bigint;
  penalty: bigint;
  arbitrator: string;
  reason: string;
  arbitratedAt: number;
}

/**
 * 仲裁服务类
 */
export class ArbitrationService {
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

  // ===== 争议管理 =====

  /**
   * 创建争议
   * @param disputeType 争议类型
   * @param relatedId 相关订单/兑换ID
   * @param defendant 被告地址
   * @param reason 争议原因
   * @param evidenceCid 初始证据的 IPFS CID（可选）
   * @param onStatusChange 状态回调
   * @returns 争议ID
   */
  async createDispute(
    disputeType: DisputeType,
    relatedId: number,
    defendant: string,
    reason: string,
    evidenceCid?: string,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备创建争议...');

    const typeEnum = disputeType === DisputeType.Order
      ? { Order: null }
      : disputeType === DisputeType.Swap
        ? { Swap: null }
        : disputeType === DisputeType.Service
          ? { Service: null }
          : { Other: null };

    if (!api.tx.arbitration?.createDispute) {
      throw new Error('Arbitration pallet not available');
    }

    const tx = api.tx.arbitration.createDispute(
      typeEnum,
      relatedId,
      defendant,
      reason,
      evidenceCid || null
    );

    onStatusChange?.('等待签名...');

    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    const disputeEvent = events.find(
      ({ event }: any) =>
        event.section === 'arbitration' &&
        event.method === 'DisputeCreated'
    );

    if (!disputeEvent) {
      throw new Error('未找到争议创建事件');
    }

    const disputeId = disputeEvent.event.data[0].toString();
    return parseInt(disputeId, 10);
  }

  /**
   * 提交证据
   * @param disputeId 争议ID
   * @param evidenceCid 证据的 IPFS CID
   * @param description 证据描述
   * @param onStatusChange 状态回调
   */
  async submitEvidence(
    disputeId: number,
    evidenceCid: string,
    description: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备提交证据...');

    if (!api.tx.arbitration?.submitEvidence) {
      throw new Error('Arbitration pallet not available');
    }
    const tx = api.tx.arbitration.submitEvidence(disputeId, evidenceCid, description);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 申诉
   * @param disputeId 争议ID
   * @param reason 申诉原因
   * @param newEvidenceCid 新证据的 IPFS CID（可选）
   * @param onStatusChange 状态回调
   */
  async appeal(
    disputeId: number,
    reason: string,
    newEvidenceCid?: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备申诉...');

    if (!api.tx.arbitration?.appeal) {
      throw new Error('Arbitration pallet not available');
    }
    const tx = api.tx.arbitration.appeal(disputeId, reason, newEvidenceCid || null);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 关闭争议（仅原告可调用，用于撤回争议）
   * @param disputeId 争议ID
   * @param onStatusChange 状态回调
   */
  async closeDispute(
    disputeId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备关闭争议...');

    if (!api.tx.arbitration?.closeDispute) {
      throw new Error('Arbitration pallet not available');
    }
    const tx = api.tx.arbitration.closeDispute(disputeId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  // ===== 查询方法 =====

  /**
   * 获取争议详情
   * @param disputeId 争议ID
   * @returns 争议信息
   */
  async getDispute(disputeId: number): Promise<Dispute | null> {
    const api = this.getApi();

    try {
      if (!api.query.arbitration?.disputes) {
        throw new Error('Arbitration pallet not available');
      }
      const dispute = await api.query.arbitration.disputes(disputeId);

      if (dispute.isEmpty) {
        return null;
      }

      const data = dispute.toJSON() as any;

      return {
        id: disputeId,
        disputeType: this.parseDisputeType(data.disputeType),
        relatedId: data.relatedId,
        plaintiff: data.plaintiff,
        defendant: data.defendant,
        reason: data.reason,
        status: this.parseDisputeStatus(data.status),
        createdAt: data.createdAt,
        resolvedAt: data.resolvedAt,
        resolution: data.resolution,
        evidences: [],
      };
    } catch (error) {
      console.error('[ArbitrationService] Get dispute error:', error);
      return null;
    }
  }

  /**
   * 获取用户相关的争议列表
   * @param account 用户地址
   * @returns 争议列表
   */
  async getUserDisputes(account: string): Promise<Dispute[]> {
    const api = this.getApi();

    try {
      if (!api.query.arbitration?.disputes) {
        return [];
      }
      const entries = await api.query.arbitration.disputes.entries();
      const disputes: Dispute[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as any;

        if (data.plaintiff === account || data.defendant === account) {
          disputes.push({
            id: parseInt(key.args[0]?.toString() || '0', 10),
            disputeType: this.parseDisputeType(data.disputeType),
            relatedId: data.relatedId,
            plaintiff: data.plaintiff,
            defendant: data.defendant,
            reason: data.reason,
            status: this.parseDisputeStatus(data.status),
            createdAt: data.createdAt,
            resolvedAt: data.resolvedAt,
            resolution: data.resolution,
            evidences: [],
          });
        }
      }

      // 按创建时间倒序排序
      disputes.sort((a, b) => b.createdAt - a.createdAt);

      return disputes;
    } catch (error) {
      console.error('[ArbitrationService] Get user disputes error:', error);
      return [];
    }
  }

  /**
   * 获取争议的证据列表
   * @param disputeId 争议ID
   * @returns 证据列表
   */
  async getDisputeEvidences(disputeId: number): Promise<Evidence[]> {
    const api = this.getApi();

    try {
      if (!api.query.arbitration?.evidences) {
        return [];
      }
      const entries = await api.query.arbitration.evidences.entries(disputeId);
      const evidences: Evidence[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as any;

        evidences.push({
          id: parseInt(key.args[1]?.toString() || '0', 10),
          disputeId,
          submitter: data.submitter,
          evidenceCid: data.evidenceCid,
          description: data.description,
          submittedAt: data.submittedAt,
        });
      }

      // 按提交时间排序
      evidences.sort((a, b) => a.submittedAt - b.submittedAt);

      return evidences;
    } catch (error) {
      console.error('[ArbitrationService] Get evidences error:', error);
      return [];
    }
  }

  /**
   * 获取仲裁结果
   * @param disputeId 争议ID
   * @returns 仲裁结果
   */
  async getArbitrationResult(disputeId: number): Promise<ArbitrationResult | null> {
    const api = this.getApi();

    try {
      if (!api.query.arbitration?.arbitrationResults) {
        return null;
      }
      const result = await api.query.arbitration.arbitrationResults(disputeId);

      if (result.isEmpty) {
        return null;
      }

      const data = result.toJSON() as any;

      return {
        disputeId,
        winner: data.winner,
        loser: data.loser,
        compensation: BigInt(data.compensation || 0),
        penalty: BigInt(data.penalty || 0),
        arbitrator: data.arbitrator,
        reason: data.reason,
        arbitratedAt: data.arbitratedAt,
      };
    } catch (error) {
      console.error('[ArbitrationService] Get result error:', error);
      return null;
    }
  }

  // ===== 辅助方法 =====

  private parseDisputeType(type: any): DisputeType {
    if (typeof type === 'string') {
      return type as DisputeType;
    }
    if (type && typeof type === 'object') {
      const key = Object.keys(type)[0];
      return key as DisputeType;
    }
    return DisputeType.Other;
  }

  private parseDisputeStatus(status: any): DisputeStatus {
    if (typeof status === 'string') {
      return status as DisputeStatus;
    }
    if (status && typeof status === 'object') {
      const key = Object.keys(status)[0];
      return key as DisputeStatus;
    }
    return DisputeStatus.Pending;
  }
}

// 导出单例
export const arbitrationService = new ArbitrationService();
