/**
 * 星尘玄鉴 - Matchmaking 服务层
 * 封装婚恋模块与区块链交互的 API
 * 
 * 支持功能：
 * - 用户资料管理（创建/更新/删除）
 * - 择偶条件设置
 * - 照片上传与管理
 * - 互动操作（喜欢/超级喜欢/跳过/屏蔽）
 * - 合婚请求管理
 * - 八字绑定与性格同步
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
 * 性别枚举
 */
export enum Gender {
  Male = 'Male',
  Female = 'Female',
}

/**
 * 隐私模式枚举
 */
export enum MatchmakingPrivacyMode {
  Public = 0,      // 公开：所有人可见
  MembersOnly = 1, // 仅会员可见
  MatchedOnly = 2, // 仅匹配后可见
}

/**
 * 互动类型枚举
 */
export enum InteractionType {
  Like = 'Like',
  SuperLike = 'SuperLike',
  Pass = 'Pass',
  Block = 'Block',
}

/**
 * 合婚请求状态
 */
export enum MatchRequestStatus {
  Pending = 'Pending',
  Authorized = 'Authorized',
  Rejected = 'Rejected',
  Cancelled = 'Cancelled',
  Completed = 'Completed',
}

/**
 * 用户资料接口
 */
export interface UserProfile {
  owner: string;
  nickname?: string;
  gender: Gender;
  birthYear: number;
  location: string;
  height: number;
  education: number;
  occupation: string;
  income: number;
  housingStatus: number;
  bio?: string;
  avatarCid?: string;
  photoCids: string[];
  baziChartId?: number;
  privacyMode: MatchmakingPrivacyMode;
  createdAt: number;
  updatedAt: number;
  membershipExpiry?: number;
}

/**
 * 择偶条件接口
 */
export interface MatchPreferences {
  minAge: number;
  maxAge: number;
  minHeight: number;
  maxHeight: number;
  minEducation: number;
  locations: string[];
  minIncome?: number;
  housingRequired: boolean;
}

/**
 * 合婚请求接口
 */
export interface MatchRequest {
  id: number;
  partyA: string;
  partyB: string;
  requester: string; // alias for partyA
  target: string; // alias for partyB
  status: MatchRequestStatus;
  createdAt: number;
  authorizedAt?: number;
  reportCid?: string;
}

/**
 * Matchmaking Service
 * 提供与 pallet-matchmaking 交互的方法
 */
export class MatchmakingService {
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

  // ===== Profile 模块 =====

  /**
   * 创建用户资料
   * 需要支付 50 USDT 等值的 DUST 作为保证金
   */
  async createProfile(
    gender: Gender,
    birthYear: number,
    location: string,
    height: number,
    education: number,
    occupation: string,
    income: number,
    housingStatus: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.createProfile) {
      throw new Error('Matchmaking pallet not available');
    }
    if (!api.tx.matchmakingProfile?.createProfile) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.createProfile(
      gender === Gender.Male ? { Male: null } : { Female: null },
      birthYear,
      location,
      height,
      education,
      occupation,
      income,
      housingStatus
    );

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 更新用户资料
   */
  async updateProfile(
    location?: string,
    height?: number,
    education?: number,
    occupation?: string,
    income?: number,
    housingStatus?: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.updateProfile) {
      throw new Error('Matchmaking pallet not available');
    }
    if (!api.tx.matchmakingProfile?.updateProfile) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.updateProfile(
      location ?? null,
      height ?? null,
      education ?? null,
      occupation ?? null,
      income ?? null,
      housingStatus ?? null
    );

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 更新择偶条件
   */
  async updatePreferences(
    preferences: MatchPreferences,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.updatePreferences) { throw new Error("Matchmaking pallet not available"); }
    if (!api.tx.matchmakingProfile?.updatePreferences) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.updatePreferences(
      preferences.minAge,
      preferences.maxAge,
      preferences.minHeight,
      preferences.maxHeight,
      preferences.minEducation,
      preferences.locations,
      preferences.minIncome ?? null,
      preferences.housingRequired
    );

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 绑定八字命盘
   */
  async linkBazi(
    baziChartId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.linkBazi) { throw new Error('Matchmaking pallet not available'); }
    if (!api.tx.matchmakingProfile?.linkBazi) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.linkBazi(baziChartId);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 更新隐私模式
   */
  async updatePrivacyMode(
    mode: MatchmakingPrivacyMode,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    const modeEnum = mode === MatchmakingPrivacyMode.Public
      ? { Public: null }
      : mode === MatchmakingPrivacyMode.MembersOnly
        ? { MembersOnly: null }
        : { MatchedOnly: null };

    if (!api.tx.matchmakingProfile?.updatePrivacyMode) { throw new Error('Matchmaking pallet not available'); }
    if (!api.tx.matchmakingProfile?.updatePrivacyMode) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.updatePrivacyMode(modeEnum);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 删除用户资料
   * 将取消固定所有照片并释放保证金
   */
  async deleteProfile(onStatusChange?: StatusCallback): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.deleteProfile) { throw new Error('Matchmaking pallet not available'); }
    if (!api.tx.matchmakingProfile?.deleteProfile) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.deleteProfile();

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 支付月费
   */
  async payMonthlyFee(
    months: number,
    referrer?: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.payMonthlyFee) { throw new Error('Matchmaking pallet not available'); }
    if (!api.tx.matchmakingProfile?.payMonthlyFee) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.payMonthlyFee(months, referrer ?? null);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 更新用户自填性格
   */
  async updateUserPersonality(
    personalityTags: number[],
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.updateUserPersonality) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.updateUserPersonality(personalityTags);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 从八字同步性格
   */
  async syncBaziPersonality(onStatusChange?: StatusCallback): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.syncBaziPersonality) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.syncBaziPersonality();

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 上传照片
   */
  async uploadPhoto(
    cid: string,
    isAvatar: boolean,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.uploadPhoto) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.uploadPhoto(cid, isAvatar);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 批量上传照片
   */
  async uploadPhotosBatch(
    cids: string[],
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingProfile?.uploadPhotosBatch) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingProfile.uploadPhotosBatch(cids);

    await signAndSend(api, tx, address, onStatusChange);
  }

  // ===== Interaction 模块 =====

  /**
   * 初始化隐私盐值
   */
  async initializeSalt(onStatusChange?: StatusCallback): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingInteraction?.initializeSalt) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.initializeSalt();

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 点赞
   */
  async like(
    target: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingInteraction?.like) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.like(target);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 超级喜欢（付费）
   */
  async superLike(
    target: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingInteraction?.superLike) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.superLike(target);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 跳过
   */
  async pass(
    target: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingInteraction?.pass) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.pass(target);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 屏蔽用户
   */
  async blockUser(
    target: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingInteraction?.blockUser) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.blockUser(target);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 取消屏蔽
   */
  async unblockUser(
    target: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingInteraction?.unblockUser) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.unblockUser(target);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 验证互动关系
   */
  async verifyInteraction(
    target: string,
    interactionType: InteractionType,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    const typeEnum = interactionType === InteractionType.Like
      ? { Like: null }
      : interactionType === InteractionType.SuperLike
        ? { SuperLike: null }
        : interactionType === InteractionType.Pass
          ? { Pass: null }
          : { Block: null };

    if (!api.tx.matchmakingInteraction?.verifyInteraction) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.verifyInteraction(target, typeEnum);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 标记超级喜欢已查看
   */
  async markSuperLikeViewed(
    sender: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingInteraction?.markSuperLikeViewed) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.markSuperLikeViewed(sender);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 发起婚恋聊天（带配额检查）
   * 
   * 权限规则：
   * - 已匹配用户可发起聊天（消耗每日配额）
   * - 收到超级喜欢后可发起聊天（不消耗配额）
   * - 已有会话可继续聊天（不消耗配额）
   * - 被动回复不消耗配额
   */
  async initiateMatchmakingChat(
    receiver: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingInteraction?.initiateMatchmakingChat) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.initiateMatchmakingChat(receiver);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 查看用户资料（消耗查看配额）
   * 
   * 配额规则：
   * - 免费用户：每日限制查看次数
   * - 会员用户：更多查看次数或无限制
   * - 同一天重复查看同一用户不消耗配额
   * - 查看自己的资料不消耗配额
   */
  async viewProfile(
    target: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingInteraction?.viewProfile) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingInteraction.viewProfile(target);

    await signAndSend(api, tx, address, onStatusChange);
  }

  // ===== Matching 模块 =====

  /**
   * 创建合婚请求
   */
  async createMatchRequest(
    partyB: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingMatching?.createRequest) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingMatching.createRequest(partyB);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 授权合婚请求
   */
  async authorizeMatchRequest(
    requestId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingMatching?.authorizeRequest) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingMatching.authorizeRequest(requestId);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 拒绝合婚请求
   */
  async rejectMatchRequest(
    requestId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingMatching?.rejectRequest) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingMatching.rejectRequest(requestId);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 取消合婚请求
   */
  async cancelMatchRequest(
    requestId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingMatching?.cancelRequest) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingMatching.cancelRequest(requestId);

    await signAndSend(api, tx, address, onStatusChange);
  }

  /**
   * 生成合婚报告
   */
  async generateMatchReport(
    requestId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const address = await getCurrentSignerAddress();

    if (!address) {
      throw new Error('No signer address available');
    }

    if (!api.tx.matchmakingMatching?.generateReport) { throw new Error("Matchmaking pallet not available"); }
    const tx = api.tx.matchmakingMatching.generateReport(requestId);

    await signAndSend(api, tx, address, onStatusChange);
  }

  // ===== 查询方法 =====

  /**
   * 获取用户资料
   */
  async getProfile(address: string): Promise<UserProfile | null> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingProfile?.profiles) {
        return null;
      }
      const result = await api.query.matchmakingProfile.profiles(address);

      if ((result as any).isNone || result.isEmpty) {
        return null;
      }

      const profile = (result as any).unwrap ? (result as any).unwrap() : result.toJSON() as any;

      return {
        owner: profile.owner.toString(),
        gender: profile.gender.isMale ? Gender.Male : Gender.Female,
        birthYear: profile.birthYear.toNumber(),
        location: profile.location.toHuman() as string,
        height: profile.height.toNumber(),
        education: profile.education.toNumber(),
        occupation: profile.occupation.toHuman() as string,
        income: profile.income.toNumber(),
        housingStatus: profile.housingStatus.toNumber(),
        avatarCid: profile.avatarCid.isSome ? profile.avatarCid.unwrap().toHuman() as string : undefined,
        photoCids: profile.photoCids.map((cid: any) => cid.toHuman() as string),
        baziChartId: profile.baziChartId.isSome ? profile.baziChartId.unwrap().toNumber() : undefined,
        privacyMode: profile.privacyMode.isPublic
          ? MatchmakingPrivacyMode.Public
          : profile.privacyMode.isMembersOnly
            ? MatchmakingPrivacyMode.MembersOnly
            : MatchmakingPrivacyMode.MatchedOnly,
        createdAt: profile.createdAt.toNumber(),
        updatedAt: profile.updatedAt.toNumber(),
        membershipExpiry: profile.membershipExpiry.isSome
          ? profile.membershipExpiry.unwrap().toNumber()
          : undefined,
      };
    } catch (error) {
      console.error('[MatchmakingService] Get profile error:', error);
      return null;
    }
  }

  /**
   * 获取择偶条件
   */
  async getPreferences(address: string): Promise<MatchPreferences | null> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingProfile?.preferences) {
        return null;
      }
      const result = await api.query.matchmakingProfile.preferences(address);

      if ((result as any).isNone || result.isEmpty) {
        return null;
      }

      const prefs = (result as any).unwrap ? (result as any).unwrap() : result.toJSON() as any;

      return {
        minAge: prefs.minAge.toNumber(),
        maxAge: prefs.maxAge.toNumber(),
        minHeight: prefs.minHeight.toNumber(),
        maxHeight: prefs.maxHeight.toNumber(),
        minEducation: prefs.minEducation.toNumber(),
        locations: prefs.locations.map((loc: any) => loc.toHuman() as string),
        minIncome: prefs.minIncome.isSome ? prefs.minIncome.unwrap().toNumber() : undefined,
        housingRequired: prefs.housingRequired.isTrue,
      };
    } catch (error) {
      console.error('[MatchmakingService] Get preferences error:', error);
      return null;
    }
  }

  /**
   * 获取合婚请求
   */
  async getMatchRequest(requestId: number): Promise<MatchRequest | null> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingMatching?.requests) {
        return null;
      }
      const result = await api.query.matchmakingMatching.requests(requestId);

      if ((result as any).isNone || result.isEmpty) {
        return null;
      }

      const request = (result as any).unwrap ? (result as any).unwrap() : result.toJSON() as any;

      let status: MatchRequestStatus;
      if (request.status.isPending) {
        status = MatchRequestStatus.Pending;
      } else if (request.status.isAuthorized) {
        status = MatchRequestStatus.Authorized;
      } else if (request.status.isRejected) {
        status = MatchRequestStatus.Rejected;
      } else if (request.status.isCancelled) {
        status = MatchRequestStatus.Cancelled;
      } else {
        status = MatchRequestStatus.Completed;
      }

      const partyAStr = request.partyA?.toString?.() || request.partyA;
      const partyBStr = request.partyB?.toString?.() || request.partyB;
      return {
        id: requestId,
        partyA: partyAStr,
        partyB: partyBStr,
        requester: partyAStr,
        target: partyBStr,
        status,
        createdAt: request.createdAt?.toNumber?.() || request.createdAt,
        authorizedAt: request.authorizedAt?.isSome
          ? request.authorizedAt.unwrap().toNumber()
          : undefined,
        reportCid: request.reportCid?.isSome
          ? request.reportCid.unwrap().toHuman() as string
          : undefined,
      };
    } catch (error) {
      console.error('[MatchmakingService] Get match request error:', error);
      return null;
    }
  }

  /**
   * 获取用户的合婚请求列表
   */
  async getUserMatchRequests(address: string): Promise<number[]> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingMatching?.userRequests) {
        return [];
      }
      const result = await api.query.matchmakingMatching.userRequests(address);
      return (result as any).map ? (result as any).map((id: any) => id.toNumber?.() || id) : [];
    } catch (error) {
      console.error('[MatchmakingService] Get user match requests error:', error);
      return [];
    }
  }

  /**
   * 检查是否已匹配
   */
  async isMatched(userA: string, userB: string): Promise<boolean> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingInteraction?.matches) {
        return false;
      }
      const result = await api.query.matchmakingInteraction.matches([userA, userB]);
      return (result as any).isSome || !result.isEmpty;
    } catch (error) {
      console.error('[MatchmakingService] Check match error:', error);
      return false;
    }
  }

  /**
   * 获取剩余配额（点赞、超级喜欢、查看）
   */
  async getRemainingQuota(address: string): Promise<{ likes: number; superLikes: number; views: number }> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingInteraction?.dailyQuotas) {
        return { likes: 0, superLikes: 0, views: 0 };
      }
      const result = await api.query.matchmakingInteraction.dailyQuotas(address);
      
      if ((result as any).isNone || result.isEmpty) {
        // 如果没有配额记录，返回默认最大值
        // 这些值应该与链端配置一致
        return { likes: 10, superLikes: 3, views: 50 };
      }

      const quota = result.toJSON() as any;
      
      // 获取配置的最大值（这里使用默认值，实际应该从链端查询）
      const maxLikes = 10;
      const maxSuperLikes = 3;
      const maxViews = 50;
      
      return {
        likes: Math.max(0, maxLikes - (quota.likesUsed || 0)),
        superLikes: Math.max(0, maxSuperLikes - (quota.superLikesUsed || 0)),
        views: Math.max(0, maxViews - (quota.viewsUsed || 0)),
      };
    } catch (error) {
      console.error('[MatchmakingService] Get remaining quota error:', error);
      return { likes: 0, superLikes: 0, views: 0 };
    }
  }

  /**
   * 获取剩余聊天发起配额
   */
  async getRemainingChatQuota(address: string): Promise<{ remaining: number; total: number }> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingInteraction?.chatInitiationQuotas) {
        return { remaining: 0, total: 0 };
      }
      const result = await api.query.matchmakingInteraction.chatInitiationQuotas(address);
      
      if ((result as any).isNone || result.isEmpty) {
        // 如果没有配额记录，返回默认最大值
        const defaultLimit = 3; // 免费用户默认值
        return { remaining: defaultLimit, total: defaultLimit };
      }

      const quota = result.toJSON() as any;
      const defaultLimit = 3; // 免费用户默认值
      
      return {
        remaining: Math.max(0, defaultLimit - (quota.chatsInitiated || 0)),
        total: defaultLimit,
      };
    } catch (error) {
      console.error('[MatchmakingService] Get remaining chat quota error:', error);
      return { remaining: 0, total: 0 };
    }
  }

  /**
   * 获取查看过我的用户列表
   */
  async getProfileViewers(address: string): Promise<Array<{ viewer: string; viewedAt: number }>> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingInteraction?.profileViewers) {
        return [];
      }
      const result = await api.query.matchmakingInteraction.profileViewers(address);
      
      if ((result as any).isNone || result.isEmpty) {
        return [];
      }

      const viewers = result.toJSON() as any[];
      return viewers.map((item: any) => ({
        viewer: item[0],
        viewedAt: item[1],
      }));
    } catch (error) {
      console.error('[MatchmakingService] Get profile viewers error:', error);
      return [];
    }
  }

  /**
   * 获取收到的超级喜欢列表
   */
  async getSuperLikesReceived(address: string): Promise<Array<{ senderHash: string; timestamp: number; viewed: boolean }>> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingInteraction?.superLikesReceived) {
        return [];
      }
      const result = await api.query.matchmakingInteraction.superLikesReceived(address);
      
      if ((result as any).isNone || result.isEmpty) {
        return [];
      }

      const superLikes = result.toJSON() as any[];
      return superLikes.map((item: any) => ({
        senderHash: item.senderHash,
        timestamp: item.timestamp,
        viewed: item.viewed,
      }));
    } catch (error) {
      console.error('[MatchmakingService] Get super likes received error:', error);
      return [];
    }
  }

  /**
   * 获取未查看的超级喜欢数量
   */
  async getUnviewedSuperLikesCount(address: string): Promise<number> {
    const superLikes = await this.getSuperLikesReceived(address);
    return superLikes.filter(sl => !sl.viewed).length;
  }

  /**
   * 获取用户匹配列表
   */
  async getUserMatches(address: string): Promise<string[]> {
    const api = this.getApi();

    try {
      if (!api.query.matchmakingInteraction?.matches) {
        return [];
      }
      const entries = await api.query.matchmakingInteraction.matches.entries();
      const matches: string[] = [];

      for (const [key, value] of entries) {
        if ((value as any).isSome || !value.isEmpty) {
          const [userA, userB] = (key.args[0] as any).toHuman() as [string, string];
          if (userA === address) {
            matches.push(userB);
          } else if (userB === address) {
            matches.push(userA);
          }
        }
      }

      return matches;
    } catch (error) {
      console.error('[MatchmakingService] Get user matches error:', error);
      return [];
    }
  }
}

// 导出单例实例
export const matchmakingService = new MatchmakingService();
