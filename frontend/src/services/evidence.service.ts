/**
 * 证据存证服务
 * 处理证据提交、验证、撤销等
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
 * 证据状态
 */
export enum EvidenceStatus {
  Pending = 'Pending',     // 待验证
  Verified = 'Verified',   // 已验证
  Revoked = 'Revoked',     // 已撤销
  Expired = 'Expired',     // 已过期
}

/**
 * 证据类型
 */
export enum EvidenceType {
  Screenshot = 'Screenshot',   // 截图
  Document = 'Document',       // 文档
  Video = 'Video',             // 视频
  Audio = 'Audio',             // 音频
  Transaction = 'Transaction', // 交易记录
  Other = 'Other',             // 其他
}

/**
 * 证据信息
 */
export interface Evidence {
  id: number;
  submitter: string;
  evidenceType: EvidenceType;
  contentCid: string;
  contentHash: string;
  description: string;
  status: EvidenceStatus;
  submittedAt: number;
  verifiedAt?: number;
  revokedAt?: number;
  relatedDisputeId?: number;
}

/**
 * 证据存证服务类
 */
export class EvidenceService {
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

  // ===== 证据管理 =====

  /**
   * 提交证据
   * @param evidenceType 证据类型
   * @param contentCid 证据内容的 IPFS CID
   * @param description 证据描述
   * @param relatedDisputeId 相关争议ID（可选）
   * @param onStatusChange 状态回调
   * @returns 证据ID
   */
  async submitEvidence(
    evidenceType: EvidenceType,
    contentCid: string,
    description: string,
    relatedDisputeId?: number,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备提交证据...');

    const typeEnum = evidenceType === EvidenceType.Screenshot
      ? { Screenshot: null }
      : evidenceType === EvidenceType.Document
        ? { Document: null }
        : evidenceType === EvidenceType.Video
          ? { Video: null }
          : evidenceType === EvidenceType.Audio
            ? { Audio: null }
            : evidenceType === EvidenceType.Transaction
              ? { Transaction: null }
              : { Other: null };

    const tx = api.tx.evidence.submitEvidence(
      typeEnum,
      contentCid,
      description,
      relatedDisputeId ?? null
    );

    onStatusChange?.('等待签名...');

    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    const evidenceEvent = events.find(
      ({ event }: any) =>
        event.section === 'evidence' &&
        event.method === 'EvidenceSubmitted'
    );

    if (!evidenceEvent) {
      throw new Error('未找到证据提交事件');
    }

    const evidenceId = evidenceEvent.event.data[0].toString();
    return parseInt(evidenceId, 10);
  }

  /**
   * 撤销证据
   * @param evidenceId 证据ID
   * @param reason 撤销原因（可选）
   * @param onStatusChange 状态回调
   */
  async revokeEvidence(
    evidenceId: number,
    reason?: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = await getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备撤销证据...');

    const tx = api.tx.evidence.revokeEvidence(evidenceId, reason || null);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  // ===== 查询方法 =====

  /**
   * 获取证据详情
   * @param evidenceId 证据ID
   * @returns 证据信息
   */
  async getEvidence(evidenceId: number): Promise<Evidence | null> {
    const api = this.getApi();

    try {
      const evidence = await api.query.evidence.evidences(evidenceId);

      if (evidence.isEmpty) {
        return null;
      }

      const data = evidence.toJSON() as any;

      return {
        id: evidenceId,
        submitter: data.submitter,
        evidenceType: this.parseEvidenceType(data.evidenceType),
        contentCid: data.contentCid,
        contentHash: data.contentHash,
        description: data.description,
        status: this.parseEvidenceStatus(data.status),
        submittedAt: data.submittedAt,
        verifiedAt: data.verifiedAt,
        revokedAt: data.revokedAt,
        relatedDisputeId: data.relatedDisputeId,
      };
    } catch (error) {
      console.error('[EvidenceService] Get evidence error:', error);
      return null;
    }
  }

  /**
   * 获取用户提交的证据列表
   * @param account 用户地址
   * @returns 证据列表
   */
  async getUserEvidences(account: string): Promise<Evidence[]> {
    const api = this.getApi();

    try {
      const entries = await api.query.evidence.evidences.entries();
      const evidences: Evidence[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as any;

        if (data.submitter === account) {
          evidences.push({
            id: parseInt(key.args[0].toString(), 10),
            submitter: data.submitter,
            evidenceType: this.parseEvidenceType(data.evidenceType),
            contentCid: data.contentCid,
            contentHash: data.contentHash,
            description: data.description,
            status: this.parseEvidenceStatus(data.status),
            submittedAt: data.submittedAt,
            verifiedAt: data.verifiedAt,
            revokedAt: data.revokedAt,
            relatedDisputeId: data.relatedDisputeId,
          });
        }
      }

      // 按提交时间倒序排序
      evidences.sort((a, b) => b.submittedAt - a.submittedAt);

      return evidences;
    } catch (error) {
      console.error('[EvidenceService] Get user evidences error:', error);
      return [];
    }
  }

  /**
   * 获取争议相关的证据列表
   * @param disputeId 争议ID
   * @returns 证据列表
   */
  async getDisputeEvidences(disputeId: number): Promise<Evidence[]> {
    const api = this.getApi();

    try {
      const entries = await api.query.evidence.evidences.entries();
      const evidences: Evidence[] = [];

      for (const [key, value] of entries) {
        const data = value.toJSON() as any;

        if (data.relatedDisputeId === disputeId) {
          evidences.push({
            id: parseInt(key.args[0].toString(), 10),
            submitter: data.submitter,
            evidenceType: this.parseEvidenceType(data.evidenceType),
            contentCid: data.contentCid,
            contentHash: data.contentHash,
            description: data.description,
            status: this.parseEvidenceStatus(data.status),
            submittedAt: data.submittedAt,
            verifiedAt: data.verifiedAt,
            revokedAt: data.revokedAt,
            relatedDisputeId: data.relatedDisputeId,
          });
        }
      }

      // 按提交时间排序
      evidences.sort((a, b) => a.submittedAt - b.submittedAt);

      return evidences;
    } catch (error) {
      console.error('[EvidenceService] Get dispute evidences error:', error);
      return [];
    }
  }

  /**
   * 验证证据哈希
   * @param evidenceId 证据ID
   * @param contentHash 内容哈希
   * @returns 是否匹配
   */
  async verifyEvidenceHash(evidenceId: number, contentHash: string): Promise<boolean> {
    const evidence = await this.getEvidence(evidenceId);
    if (!evidence) {
      return false;
    }
    return evidence.contentHash === contentHash;
  }

  // ===== 辅助方法 =====

  private parseEvidenceType(type: any): EvidenceType {
    if (typeof type === 'string') {
      return type as EvidenceType;
    }
    if (type && typeof type === 'object') {
      const key = Object.keys(type)[0];
      return key as EvidenceType;
    }
    return EvidenceType.Other;
  }

  private parseEvidenceStatus(status: any): EvidenceStatus {
    if (typeof status === 'string') {
      return status as EvidenceStatus;
    }
    if (status && typeof status === 'object') {
      const key = Object.keys(status)[0];
      return key as EvidenceStatus;
    }
    return EvidenceStatus.Pending;
  }
}

// 导出单例
export const evidenceService = new EvidenceService();
