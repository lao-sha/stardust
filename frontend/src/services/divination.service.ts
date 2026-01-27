/**
 * 占卜服务 - 处理占卜结果的链上存储和查询
 *
 * 支持功能：
 * - 基础占卜结果存储
 * - 八字命盘创建（公开/加密/多方授权）
 * - 隐私模式管理
 * - 服务提供者注册
 * - Runtime API 解盘调用
 *
 * 更新日志 (2026-01-19):
 * - 添加隐私模式支持
 * - 添加多方授权加密功能
 * - 添加服务提供者注册功能
 * - 添加 Runtime API 解盘接口
 */

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/lib/api';
import { signAndSend, getCurrentSignerAddress } from '@/lib/signer';
import { u8aToHex, hexToU8a } from '@polkadot/util';

/**
 * 签名状态回调
 */
export type StatusCallback = (status: string) => void;

/**
 * 占卜类型枚举
 */
export enum DivinationType {
  Bazi = 'Bazi',           // 八字
  Ziwei = 'Ziwei',         // 紫微斗数
  Qimen = 'Qimen',         // 奇门遁甲
  Liuyao = 'Liuyao',       // 六爻
  Meihua = 'Meihua',       // 梅花易数
  Tarot = 'Tarot',         // 塔罗
  Daliuren = 'Daliuren',   // 大六壬
  Xiaoliuren = 'Xiaoliuren', // 小六壬
}

/**
 * 隐私模式枚举
 */
export enum PrivacyMode {
  Public = 0,   // 公开模式：所有数据明文
  Partial = 1,  // 部分加密：计算数据明文 + 敏感数据加密（推荐）
  Private = 2,  // 完全加密：所有数据加密
}

/**
 * 访问角色枚举
 */
export enum AccessRole {
  Owner = 'Owner',           // 所有者
  Master = 'Master',         // 命理师
  Family = 'Family',         // 家族成员
  AiService = 'AiService',   // AI 服务
}

/**
 * 访问范围枚举
 */
export enum AccessScope {
  ReadOnly = 'ReadOnly',     // 只读
  CanComment = 'CanComment', // 可评论
  FullAccess = 'FullAccess', // 完全访问
}

/**
 * 服务提供者类型枚举
 */
export enum ServiceProviderType {
  MingLiShi = 0,    // 命理师
  AiService = 1,    // AI 服务
  FamilyMember = 2, // 家族成员
  Research = 3,     // 研究机构
}

/**
 * 占卜记录
 */
export interface DivinationRecord {
  id: number;
  account: string;
  divinationType: DivinationType;
  resultCid: string;
  timestamp: number;
  blockNumber: number;
}

/**
 * 八字命盘信息
 */
export interface BaziChart {
  id: number;
  owner: string;
  name?: string;
  privacyMode: PrivacyMode;
  birthTime?: {
    year: number;
    month: number;
    day: number;
    hour: number;
    minute: number;
  };
  gender?: 'Male' | 'Female';
  createdAt: number;
}

/**
 * 服务提供者信息
 */
export interface ServiceProvider {
  account: string;
  providerType: ServiceProviderType;
  publicKey: string;
  reputation: number;
  registeredAt: number;
  isActive: boolean;
}

/**
 * 授权条目信息
 */
export interface AuthorizationEntry {
  account: string;
  role: AccessRole;
  scope: AccessScope;
  grantedAt: number;
  expiresAt: number;
}

/**
 * 占卜服务类
 */
export class DivinationService {
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
   * 将占卜结果存储到链上
   * @param divinationType 占卜类型
   * @param resultData 占卜结果数据（将被序列化为JSON）
   * @param onStatusChange 状态变化回调
   * @returns 占卜记录ID
   */
  async storeDivinationResult(
    divinationType: DivinationType,
    resultData: any,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    // 将结果数据序列化为JSON
    const resultJson = JSON.stringify(resultData);
    const resultBytes = new TextEncoder().encode(resultJson);
    const resultCid = u8aToHex(resultBytes);

    onStatusChange?.('准备交易...');

    // 创建交易
    const tx = api.tx.divination.storeDivinationResult(
      divinationType,
      resultCid
    );

    onStatusChange?.('等待签名...');

    // 签名并发送交易
    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    // 从事件中提取占卜记录ID
    const divinationEvent = events.find(
      ({ event }: any) =>
        event.section === 'divination' &&
        event.method === 'DivinationStored'
    );

    if (!divinationEvent) {
      throw new Error('未找到占卜存储事件');
    }

    // 提取记录ID（假设事件数据格式为 [account, id, type, cid]）
    const recordId = divinationEvent.event.data[1].toString();

    return parseInt(recordId, 10);
  }

  /**
   * 查询用户的占卜历史记录
   * @param account 用户地址
   * @param divinationType 占卜类型（可选，不传则查询所有类型）
   * @returns 占卜记录列表
   */
  async getDivinationHistory(
    account: string,
    divinationType?: DivinationType
  ): Promise<DivinationRecord[]> {
    const api = this.getApi();

    try {
      // 查询链上存储
      const entries = await api.query.divination.divinationRecords.entries();

      const records: DivinationRecord[] = [];

      for (const [key, value] of entries) {
        const record = value.toJSON() as any;

        // 过滤用户和类型
        if (record.account === account) {
          if (!divinationType || record.divinationType === divinationType) {
            records.push({
              id: record.id,
              account: record.account,
              divinationType: record.divinationType,
              resultCid: record.resultCid,
              timestamp: record.timestamp,
              blockNumber: record.blockNumber,
            });
          }
        }
      }

      // 按时间倒序排序
      records.sort((a, b) => b.timestamp - a.timestamp);

      return records;
    } catch (error) {
      console.error('[DivinationService] Get history error:', error);
      throw error;
    }
  }

  /**
   * 查询单个占卜记录
   * @param recordId 记录ID
   * @returns 占卜记录
   */
  async getDivinationRecord(recordId: number): Promise<DivinationRecord | null> {
    const api = this.getApi();

    try {
      const record = await api.query.divination.divinationRecords(recordId);

      if (record.isEmpty) {
        return null;
      }

      const data = record.toJSON() as any;

      return {
        id: data.id,
        account: data.account,
        divinationType: data.divinationType,
        resultCid: data.resultCid,
        timestamp: data.timestamp,
        blockNumber: data.blockNumber,
      };
    } catch (error) {
      console.error('[DivinationService] Get record error:', error);
      throw error;
    }
  }

  /**
   * 解析占卜结果数据
   * @param resultCid 结果CID（十六进制字符串）
   * @returns 解析后的结果数据
   */
  parseResultData<T = any>(resultCid: string): T {
    // 移除 0x 前缀
    const hex = resultCid.startsWith('0x') ? resultCid.slice(2) : resultCid;

    // 将十六进制转换为字节数组
    const bytes = new Uint8Array(
      hex.match(/.{1,2}/g)?.map((byte) => parseInt(byte, 16)) || []
    );

    // 解码为字符串
    const json = new TextDecoder().decode(bytes);

    // 解析JSON
    return JSON.parse(json) as T;
  }

  /**
   * 获取占卜统计信息
   * @param account 用户地址
   * @returns 统计信息
   */
  async getDivinationStats(account: string): Promise<{
    total: number;
    byType: Record<DivinationType, number>;
  }> {
    const records = await this.getDivinationHistory(account);

    const stats = {
      total: records.length,
      byType: {} as Record<DivinationType, number>,
    };

    // 初始化计数器
    Object.values(DivinationType).forEach((type) => {
      stats.byType[type] = 0;
    });

    // 统计各类型数量
    records.forEach((record) => {
      stats.byType[record.divinationType]++;
    });

    return stats;
  }

  // 本地删除记录存储键
  private static readonly DELETED_RECORDS_KEY = 'stardust_deleted_divination_records';

  /**
   * 删除占卜记录（软删除，仅本地标记）
   * 注意：链上数据无法删除，此方法仅用于本地隐藏
   * @param recordId 记录ID
   */
  async markRecordAsDeleted(recordId: number): Promise<void> {
    try {
      const AsyncStorage = await this.getAsyncStorage();
      const existingData = await AsyncStorage.getItem(DivinationService.DELETED_RECORDS_KEY);
      const deletedIds: number[] = existingData ? JSON.parse(existingData) : [];
      
      if (!deletedIds.includes(recordId)) {
        deletedIds.push(recordId);
        await AsyncStorage.setItem(
          DivinationService.DELETED_RECORDS_KEY,
          JSON.stringify(deletedIds)
        );
      }
      
      console.log('标记记录为已删除:', recordId);
    } catch (error) {
      console.error('标记删除记录失败:', error);
      throw error;
    }
  }

  /**
   * 检查记录是否已被标记为删除
   * @param recordId 记录ID
   */
  async isRecordDeleted(recordId: number): Promise<boolean> {
    try {
      const AsyncStorage = await this.getAsyncStorage();
      const existingData = await AsyncStorage.getItem(DivinationService.DELETED_RECORDS_KEY);
      const deletedIds: number[] = existingData ? JSON.parse(existingData) : [];
      return deletedIds.includes(recordId);
    } catch (error) {
      console.error('检查删除状态失败:', error);
      return false;
    }
  }

  /**
   * 获取所有已删除的记录ID
   */
  async getDeletedRecordIds(): Promise<number[]> {
    try {
      const AsyncStorage = await this.getAsyncStorage();
      const existingData = await AsyncStorage.getItem(DivinationService.DELETED_RECORDS_KEY);
      return existingData ? JSON.parse(existingData) : [];
    } catch (error) {
      console.error('获取删除记录列表失败:', error);
      return [];
    }
  }

  /**
   * 恢复已删除的记录
   * @param recordId 记录ID
   */
  async restoreDeletedRecord(recordId: number): Promise<void> {
    try {
      const AsyncStorage = await this.getAsyncStorage();
      const existingData = await AsyncStorage.getItem(DivinationService.DELETED_RECORDS_KEY);
      const deletedIds: number[] = existingData ? JSON.parse(existingData) : [];
      
      const index = deletedIds.indexOf(recordId);
      if (index > -1) {
        deletedIds.splice(index, 1);
        await AsyncStorage.setItem(
          DivinationService.DELETED_RECORDS_KEY,
          JSON.stringify(deletedIds)
        );
      }
      
      console.log('恢复已删除记录:', recordId);
    } catch (error) {
      console.error('恢复删除记录失败:', error);
      throw error;
    }
  }

  /**
   * 动态导入 AsyncStorage（避免循环依赖）
   */
  private async getAsyncStorage() {
    const { default: AsyncStorage } = await import('@react-native-async-storage/async-storage');
    return AsyncStorage;
  }

  /**
   * 临时计算八字（免费试算，不保存到链上）
   * @param birthYear 出生年份
   * @param birthMonth 出生月份 (1-12)
   * @param birthDay 出生日期 (1-31)
   * @param birthHour 出生小时 (0-23)
   * @param birthMinute 出生分钟 (0-59)
   * @param gender 性别 ('male' | 'female')
   * @param calendarType 日历类型 ('solar' | 'lunar')
   * @returns 八字命盘数据（JSON 格式）
   */
  async calculateBaziTemp(
    birthYear: number,
    birthMonth: number,
    birthDay: number,
    birthHour: number,
    birthMinute: number,
    gender: 'male' | 'female',
    calendarType: 'solar' | 'lunar' = 'solar'
  ): Promise<any> {
    const api = this.getApi();

    // 准备参数
    const inputType = calendarType === 'solar' ? 0 : 1; // 0=公历, 1=农历
    const params = [birthYear, birthMonth, birthDay, birthHour, birthMinute];
    const genderValue = gender === 'male' ? 0 : 1; // 0=男, 1=女
    const zishiMode = 2; // 2=现代派

    try {
      // 调用 Runtime API（免费，不上链）
      const result = await api.call.baziChartApi.calculateBaziTempUnified(
        inputType,
        params,
        genderValue,
        zishiMode
      );

      if (!result) {
        throw new Error('计算失败，请检查输入参数');
      }

      // 解析 JSON 字符串
      return JSON.parse(result.toString());
    } catch (error) {
      console.error('[DivinationService] Calculate bazi temp error:', error);
      throw error;
    }
  }

  /**
   * 创建八字命盘并保存到链上
   * @param name 命盘名称（可选）
   * @param birthYear 出生年份
   * @param birthMonth 出生月份 (1-12)
   * @param birthDay 出生日期 (1-31)
   * @param birthHour 出生小时 (0-23)
   * @param birthMinute 出生分钟 (0-59)
   * @param gender 性别 ('male' | 'female')
   * @param calendarType 日历类型 ('solar' | 'lunar')
   * @param onStatusChange 状态变化回调
   * @returns 命盘ID
   */
  async createBaziChart(
    name: string | null,
    birthYear: number,
    birthMonth: number,
    birthDay: number,
    birthHour: number,
    birthMinute: number,
    gender: 'male' | 'female',
    calendarType: 'solar' | 'lunar' = 'solar',
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    // 构造输入类型
    let input;
    if (calendarType === 'solar') {
      input = {
        Solar: {
          year: birthYear,
          month: birthMonth,
          day: birthDay,
          hour: birthHour,
          minute: birthMinute,
        }
      };
    } else {
      input = {
        Lunar: {
          year: birthYear,
          month: birthMonth,
          day: birthDay,
          isLeapMonth: false,
          hour: birthHour,
          minute: birthMinute,
        }
      };
    }

    // 构造其他参数
    const nameParam = name ? new TextEncoder().encode(name) : null;
    const genderParam = gender === 'male' ? 'Male' : 'Female';
    const zishiModeParam = 'Modern';
    const longitudeParam = null; // 不使用真太阳时修正

    onStatusChange?.('等待签名...');

    // 创建交易
    const tx = api.tx.bazi.createBaziChart(
      nameParam,
      input,
      genderParam,
      zishiModeParam,
      longitudeParam
    );

    // 签名并发送交易
    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    // 从事件中提取命盘ID
    const baziEvent = events.find(
      ({ event }: any) =>
        event.section === 'bazi' &&
        event.method === 'BaziChartCreated'
    );

    if (!baziEvent) {
      throw new Error('未找到八字创建事件');
    }

    // 提取命盘ID（假设事件数据格式为 [owner, chart_id, birth_time]）
    const chartId = baziEvent.event.data[1].toString();

    return parseInt(chartId, 10);
  }

  // ===== 隐私模式相关 =====

  /**
   * 创建带隐私模式的八字命盘
   *
   * @param privacyMode 隐私模式 (0=Public, 1=Partial, 2=Private)
   * @param name 命盘名称（可选）
   * @param birthYear 出生年份
   * @param birthMonth 出生月份 (1-12)
   * @param birthDay 出生日期 (1-31)
   * @param birthHour 出生小时 (0-23)
   * @param birthMinute 出生分钟 (0-59)
   * @param gender 性别 ('male' | 'female')
   * @param calendarType 日历类型 ('solar' | 'lunar')
   * @param encryptedData 加密的敏感数据（Partial/Private 模式必填）
   * @param dataHash 数据哈希（用于验证）
   * @param ownerKeyBackup 所有者密钥包（92 bytes）
   * @param onStatusChange 状态变化回调
   * @returns 命盘ID
   */
  async createBaziChartEncrypted(
    privacyMode: PrivacyMode,
    name: string | null,
    birthYear: number,
    birthMonth: number,
    birthDay: number,
    birthHour: number,
    birthMinute: number,
    gender: 'male' | 'female',
    calendarType: 'solar' | 'lunar' = 'solar',
    encryptedData?: Uint8Array,
    dataHash?: Uint8Array,
    ownerKeyBackup?: Uint8Array,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    // 验证参数
    if (privacyMode !== PrivacyMode.Public && (!encryptedData || !dataHash || !ownerKeyBackup)) {
      throw new Error('Partial/Private 模式需要提供加密数据、数据哈希和密钥包');
    }

    onStatusChange?.('准备交易...');

    // 构造输入类型
    let input = null;
    if (privacyMode !== PrivacyMode.Private) {
      if (calendarType === 'solar') {
        input = {
          Solar: {
            year: birthYear,
            month: birthMonth,
            day: birthDay,
            hour: birthHour,
            minute: birthMinute,
          }
        };
      } else {
        input = {
          Lunar: {
            year: birthYear,
            month: birthMonth,
            day: birthDay,
            isLeapMonth: false,
            hour: birthHour,
            minute: birthMinute,
          }
        };
      }
    }

    const nameParam = name ? new TextEncoder().encode(name) : null;
    const genderParam = privacyMode !== PrivacyMode.Private
      ? (gender === 'male' ? 'Male' : 'Female')
      : null;
    const zishiModeParam = privacyMode !== PrivacyMode.Private ? 'Modern' : null;

    onStatusChange?.('等待签名...');

    // 创建交易
    const tx = api.tx.bazi.createBaziChartEncrypted(
      privacyMode,
      nameParam,
      input,
      genderParam,
      zishiModeParam,
      null, // longitude
      encryptedData || null,
      dataHash ? Array.from(dataHash) : null,
      ownerKeyBackup ? Array.from(ownerKeyBackup) : null
    );

    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    const baziEvent = events.find(
      ({ event }: any) =>
        event.section === 'bazi' &&
        (event.method === 'BaziChartCreatedWithPrivacy' || event.method === 'BaziChartCreated')
    );

    if (!baziEvent) {
      throw new Error('未找到八字创建事件');
    }

    const chartId = baziEvent.event.data[1].toString();
    return parseInt(chartId, 10);
  }

  // ===== 加密公钥管理 =====

  /**
   * 注册加密公钥
   *
   * 用户需要先注册 X25519 加密公钥才能：
   * 1. 创建多方授权加密命盘
   * 2. 被授权访问他人的加密命盘
   *
   * @param publicKey X25519 公钥（32 bytes）
   * @param onStatusChange 状态回调
   */
  async registerEncryptionKey(
    publicKey: Uint8Array,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    if (publicKey.length !== 32) {
      throw new Error('公钥必须是 32 bytes');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.bazi.registerEncryptionKey(Array.from(publicKey));

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 更新加密公钥
   *
   * @param newPublicKey 新的 X25519 公钥（32 bytes）
   * @param onStatusChange 状态回调
   */
  async updateEncryptionKey(
    newPublicKey: Uint8Array,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    if (newPublicKey.length !== 32) {
      throw new Error('公钥必须是 32 bytes');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.bazi.updateEncryptionKey(Array.from(newPublicKey));

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 获取用户的加密公钥
   *
   * @param account 用户地址
   * @returns 公钥（32 bytes）或 null
   */
  async getUserEncryptionKey(account: string): Promise<Uint8Array | null> {
    const api = this.getApi();

    try {
      const key = await api.query.bazi.userEncryptionKeys(account);

      if (key.isEmpty) {
        return null;
      }

      return new Uint8Array(key.toU8a());
    } catch (error) {
      console.error('[DivinationService] Get encryption key error:', error);
      return null;
    }
  }

  // ===== 多方授权功能 =====

  /**
   * 授权访问命盘
   *
   * @param chartId 命盘ID
   * @param grantee 被授权账户地址
   * @param encryptedKey 用被授权方公钥加密的 DataKey
   * @param role 授权角色
   * @param scope 访问范围
   * @param expiresAt 过期区块号（0 = 永久有效）
   * @param onStatusChange 状态回调
   */
  async grantChartAccess(
    chartId: number,
    grantee: string,
    encryptedKey: Uint8Array,
    role: AccessRole,
    scope: AccessScope,
    expiresAt: number = 0,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.bazi.grantChartAccess(
      chartId,
      grantee,
      Array.from(encryptedKey),
      role,
      scope,
      expiresAt
    );

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 撤销访问权限
   *
   * @param chartId 命盘ID
   * @param revokee 被撤销账户地址
   * @param onStatusChange 状态回调
   */
  async revokeChartAccess(
    chartId: number,
    revokee: string,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.bazi.revokeChartAccess(chartId, revokee);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 撤销所有访问权限（紧急情况）
   *
   * @param chartId 命盘ID
   * @param onStatusChange 状态回调
   */
  async revokeAllChartAccess(
    chartId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.bazi.revokeAllChartAccess(chartId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 获取命盘的授权列表
   *
   * @param chartId 命盘ID
   * @returns 授权列表
   */
  async getChartAuthorizations(chartId: number): Promise<AuthorizationEntry[]> {
    const api = this.getApi();

    try {
      const entries = await api.query.bazi.chartAuthorizations.entries(chartId);
      return entries.map(([key, value]) => {
        const data = value.toJSON() as any;
        return {
          account: key.args[1].toString(),
          role: data.role,
          scope: data.scope,
          grantedAt: data.grantedAt,
          expiresAt: data.expiresAt,
        };
      });
    } catch (error) {
      console.error('[DivinationService] Get chart authorizations error:', error);
      return [];
    }
  }

  // ===== 服务提供者功能 =====

  /**
   * 注册为服务提供者
   *
   * @param providerType 服务类型
   * @param publicKey X25519 公钥（32 bytes）
   * @param onStatusChange 状态回调
   */
  async registerServiceProvider(
    providerType: ServiceProviderType,
    publicKey: Uint8Array,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    if (publicKey.length !== 32) {
      throw new Error('公钥必须是 32 bytes');
    }

    onStatusChange?.('准备交易...');

    const tx = api.tx.bazi.registerProvider(providerType, Array.from(publicKey));

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 获取服务提供者信息
   *
   * @param account 提供者账户
   * @returns 服务提供者信息
   */
  async getServiceProvider(account: string): Promise<ServiceProvider | null> {
    const api = this.getApi();

    try {
      const provider = await api.query.bazi.serviceProviders(account);

      if (provider.isEmpty) {
        return null;
      }

      const data = provider.toJSON() as any;

      return {
        account,
        providerType: data.providerType,
        publicKey: data.publicKey,
        reputation: data.reputation,
        registeredAt: data.registeredAt,
        isActive: data.isActive,
      };
    } catch (error) {
      console.error('[DivinationService] Get provider error:', error);
      return null;
    }
  }

  /**
   * 获取指定类型的活跃服务提供者列表
   *
   * @param providerType 服务类型
   * @returns 提供者账户列表
   */
  async getActiveProvidersByType(providerType: ServiceProviderType): Promise<string[]> {
    const api = this.getApi();

    try {
      const result = await api.call.baziChartApi.getProvidersByTypeFiltered(providerType);
      return result ? result.toJSON() as string[] : [];
    } catch (error) {
      console.error('[DivinationService] Get providers by type error:', error);
      return [];
    }
  }

  // ===== Runtime API 解盘功能 =====

  /**
   * 获取完整解盘结果（免费 Runtime API）
   *
   * @param chartId 命盘ID
   * @returns 完整解盘结果（JSON 格式）
   */
  async getFullInterpretation(chartId: number): Promise<any | null> {
    const api = this.getApi();

    try {
      const result = await api.call.baziChartApi.getFullInterpretation(chartId);

      if (!result) {
        return null;
      }

      return JSON.parse(result.toString());
    } catch (error) {
      console.error('[DivinationService] Get interpretation error:', error);
      return null;
    }
  }

  /**
   * 缓存解盘结果到链上
   *
   * 将解盘结果缓存到链上，后续查询更快。
   * 需要支付少量 gas 费用。
   *
   * @param chartId 命盘ID
   * @param onStatusChange 状态回调
   */
  async cacheInterpretation(
    chartId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备缓存解盘...');

    const tx = api.tx.bazi.cacheInterpretation(chartId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 删除八字命盘
   *
   * @param chartId 命盘ID
   * @param onStatusChange 状态回调
   */
  async deleteBaziChart(
    chartId: number,
    onStatusChange?: StatusCallback
  ): Promise<void> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备删除...');

    const tx = api.tx.bazi.deleteBaziChart(chartId);

    onStatusChange?.('等待签名...');
    await signAndSend(api, tx, accountAddress, onStatusChange);
  }

  /**
   * 获取用户的八字命盘列表
   *
   * @param account 用户地址
   * @returns 命盘ID列表
   */
  async getUserBaziCharts(account: string): Promise<number[]> {
    const api = this.getApi();

    try {
      const chartIds = await api.query.bazi.userCharts(account);
      return chartIds.toJSON() as number[];
    } catch (error) {
      console.error('[DivinationService] Get user charts error:', error);
      return [];
    }
  }

  /**
   * 获取八字命盘详情
   *
   * @param chartId 命盘ID
   * @returns 命盘信息
   */
  async getBaziChart(chartId: number): Promise<BaziChart | null> {
    const api = this.getApi();

    try {
      const chart = await api.query.bazi.chartById(chartId);

      if (chart.isEmpty) {
        return null;
      }

      const data = chart.toJSON() as any;

      return {
        id: chartId,
        owner: data.owner,
        name: data.name,
        privacyMode: data.privacyMode,
        birthTime: data.birthTime,
        gender: data.gender,
        createdAt: data.timestamp,
      };
    } catch (error) {
      console.error('[DivinationService] Get chart error:', error);
      return null;
    }
  }
}

// 导出单例
export const divinationService = new DivinationService();
