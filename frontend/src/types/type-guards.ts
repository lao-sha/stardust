/**
 * 星尘玄鉴 - 类型守卫
 * 
 * 提供运行时类型验证，确保链上数据符合预期格式
 */

import type {
  AccountInfo,
  AccountData,
  RawOtcOrder,
  OtcOrderState,
  RawDispute,
  DisputeStatus,
  DisputeResolution,
  RawMakerApplication,
  MakerStatus,
  RawProvider,
  RawServicePackage,
  RawMarketOrder,
  MarketOrderStatus,
  RawReview,
  RawMarketStats,
  RawBuyerCredit,
  RawSwapRecord,
  SwapStatus,
  RawContact,
  FriendStatus,
} from './substrate.types';

// ==================== 基础类型守卫 ====================

/**
 * 检查是否为有效的 Substrate 地址
 */
export function isValidSubstrateAddress(value: unknown): value is string {
  if (typeof value !== 'string') return false;
  // SS58 地址格式：以数字开头，长度 47-48
  return /^[1-9A-HJ-NP-Za-km-z]{47,48}$/.test(value);
}

/**
 * 检查是否为有效的区块哈希
 */
export function isValidBlockHash(value: unknown): value is string {
  if (typeof value !== 'string') return false;
  return /^0x[a-fA-F0-9]{64}$/.test(value);
}

/**
 * 检查是否为非负整数
 */
export function isNonNegativeInteger(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value >= 0;
}

/**
 * 检查是否为有效的余额字符串
 */
export function isValidBalanceString(value: unknown): value is string {
  if (typeof value !== 'string') return false;
  return /^\d+$/.test(value.replace(/,/g, ''));
}

// ==================== 枚举类型守卫 ====================

const OTC_ORDER_STATES: OtcOrderState[] = [
  'Created', 'Paid', 'Released', 'Cancelled', 'Disputed', 'Resolved', 'Expired'
];

/**
 * 检查是否为有效的 OTC 订单状态
 */
export function isOtcOrderState(value: unknown): value is OtcOrderState {
  if (typeof value === 'string') {
    return OTC_ORDER_STATES.includes(value as OtcOrderState);
  }
  if (typeof value === 'object' && value !== null) {
    const keys = Object.keys(value);
    return keys.length === 1 && OTC_ORDER_STATES.includes(keys[0] as OtcOrderState);
  }
  return false;
}

/**
 * 解析枚举值（处理 { EnumVariant: null } 格式）
 */
export function parseEnumValue<T extends string>(
  value: unknown,
  validValues: readonly T[]
): T | null {
  if (typeof value === 'string' && validValues.includes(value as T)) {
    return value as T;
  }
  if (typeof value === 'object' && value !== null) {
    const keys = Object.keys(value);
    if (keys.length === 1 && validValues.includes(keys[0] as T)) {
      return keys[0] as T;
    }
  }
  return null;
}

const DISPUTE_STATUSES: DisputeStatus[] = ['Initiated', 'Responded', 'Resolved'];
const DISPUTE_RESOLUTIONS: DisputeResolution[] = ['BuyerWins', 'MakerWins', 'Split'];
const MAKER_STATUSES: MakerStatus[] = ['Pending', 'Active', 'Suspended', 'Exited'];
const MARKET_ORDER_STATUSES: MarketOrderStatus[] = [
  'Pending', 'Accepted', 'Completed', 'Cancelled', 'Refunded', 'Disputed'
];
const SWAP_STATUSES: SwapStatus[] = ['Pending', 'Completed', 'Timeout', 'Reported', 'Refunded'];
const FRIEND_STATUSES: FriendStatus[] = ['None', 'Pending', 'Accepted', 'Blocked'];

// ==================== 复合类型守卫 ====================

/**
 * 检查是否为有效的账户数据
 */
export function isAccountData(value: unknown): value is AccountData {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.free === 'string' &&
    typeof obj.reserved === 'string' &&
    typeof obj.frozen === 'string'
  );
}

/**
 * 检查是否为有效的账户信息
 */
export function isAccountInfo(value: unknown): value is AccountInfo {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    isNonNegativeInteger(obj.nonce) &&
    isNonNegativeInteger(obj.consumers) &&
    isNonNegativeInteger(obj.providers) &&
    isNonNegativeInteger(obj.sufficients) &&
    isAccountData(obj.data)
  );
}

/**
 * 检查是否为有效的 OTC 订单
 */
export function isRawOtcOrder(value: unknown): value is RawOtcOrder {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    isNonNegativeInteger(obj.makerId) &&
    typeof obj.maker === 'string' &&
    typeof obj.taker === 'string' &&
    (typeof obj.price === 'string' || typeof obj.price === 'number') &&
    (typeof obj.qty === 'string' || typeof obj.qty === 'number') &&
    (typeof obj.amount === 'string' || typeof obj.amount === 'number') &&
    isNonNegativeInteger(obj.createdAt) &&
    isNonNegativeInteger(obj.expireAt) &&
    typeof obj.makerTronAddress === 'string' &&
    isOtcOrderState(obj.state)
  );
}

/**
 * 检查是否为有效的争议信息
 */
export function isRawDispute(value: unknown): value is RawDispute {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.initiator === 'string' &&
    typeof obj.reason === 'string' &&
    isNonNegativeInteger(obj.deadline)
  );
}

/**
 * 检查是否为有效的做市商申请
 */
export function isRawMakerApplication(value: unknown): value is RawMakerApplication {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.owner === 'string' &&
    typeof obj.tronAddress === 'string' &&
    typeof obj.buyPremiumBps === 'number' &&
    typeof obj.sellPremiumBps === 'number' &&
    typeof obj.usersServed === 'number' &&
    typeof obj.maskedFullName === 'string' &&
    typeof obj.wechatId === 'string'
  );
}

/**
 * 检查是否为有效的解卦师信息
 */
export function isRawProvider(value: unknown): value is RawProvider {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.name === 'string' &&
    typeof obj.bio === 'string' &&
    typeof obj.divinationTypes === 'number' &&
    typeof obj.specialties === 'number' &&
    typeof obj.tier === 'number' &&
    typeof obj.totalOrders === 'number' &&
    typeof obj.completedOrders === 'number' &&
    typeof obj.averageRating === 'number' &&
    typeof obj.isActive === 'boolean' &&
    isNonNegativeInteger(obj.registeredAt)
  );
}

/**
 * 检查是否为有效的服务套餐
 */
export function isRawServicePackage(value: unknown): value is RawServicePackage {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.providerId === 'string' &&
    typeof obj.name === 'string' &&
    typeof obj.description === 'string' &&
    typeof obj.divinationType === 'number' &&
    typeof obj.deliveryDays === 'number' &&
    typeof obj.maxFollowUps === 'number' &&
    typeof obj.isActive === 'boolean'
  );
}

/**
 * 检查是否为有效的市场订单
 */
export function isRawMarketOrder(value: unknown): value is RawMarketOrder {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    isNonNegativeInteger(obj.id) &&
    typeof obj.clientId === 'string' &&
    typeof obj.providerId === 'string' &&
    isNonNegativeInteger(obj.packageId) &&
    typeof obj.questionCid === 'string' &&
    isNonNegativeInteger(obj.createdAt)
  );
}

/**
 * 检查是否为有效的评价
 */
export function isRawReview(value: unknown): value is RawReview {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    isNonNegativeInteger(obj.orderId) &&
    typeof obj.clientId === 'string' &&
    typeof obj.providerId === 'string' &&
    typeof obj.accuracy === 'number' &&
    typeof obj.attitude === 'number' &&
    typeof obj.speed === 'number' &&
    typeof obj.value === 'number' &&
    typeof obj.isAnonymous === 'boolean' &&
    isNonNegativeInteger(obj.createdAt)
  );
}

/**
 * 检查是否为有效的市场统计
 */
export function isRawMarketStats(value: unknown): value is RawMarketStats {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.otcPrice === 'number' &&
    typeof obj.bridgePrice === 'number' &&
    typeof obj.weightedPrice === 'number' &&
    typeof obj.simpleAvgPrice === 'number'
  );
}

/**
 * 检查是否为有效的买家信用
 */
export function isRawBuyerCredit(value: unknown): value is RawBuyerCredit {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.riskScore === 'number' &&
    typeof obj.maxAmount === 'number' &&
    typeof obj.concurrentOrders === 'number' &&
    typeof obj.maxConcurrentOrders === 'number' &&
    typeof obj.completedOrders === 'number'
  );
}

/**
 * 检查是否为有效的兑换记录
 */
export function isRawSwapRecord(value: unknown): value is RawSwapRecord {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.buyer === 'string' &&
    isNonNegativeInteger(obj.makerId) &&
    typeof obj.buyerTronAddress === 'string' &&
    typeof obj.makerTronAddress === 'string'
  );
}

/**
 * 检查是否为有效的联系人
 */
export function isRawContact(value: unknown): value is RawContact {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    Array.isArray(obj.groups) &&
    isNonNegativeInteger(obj.addedAt)
  );
}

// ==================== 解析辅助函数 ====================

/**
 * 安全解析布尔值（处理 { isTrue: boolean } 格式）
 */
export function parseBoolean(value: unknown): boolean {
  if (typeof value === 'boolean') return value;
  if (typeof value === 'object' && value !== null) {
    const obj = value as Record<string, unknown>;
    if ('isTrue' in obj) return Boolean(obj.isTrue);
  }
  return false;
}

/**
 * 安全解析 BigInt
 */
export function parseBigInt(value: unknown): bigint {
  if (typeof value === 'bigint') return value;
  if (typeof value === 'number') return BigInt(Math.floor(value));
  if (typeof value === 'string') {
    const cleaned = value.replace(/,/g, '');
    return BigInt(cleaned || '0');
  }
  return BigInt(0);
}

/**
 * 安全解析数字
 */
export function parseNumber(value: unknown, defaultValue: number = 0): number {
  if (typeof value === 'number') return value;
  if (typeof value === 'string') {
    const num = parseFloat(value.replace(/,/g, ''));
    return isNaN(num) ? defaultValue : num;
  }
  return defaultValue;
}

/**
 * 安全解析字符串
 */
export function parseString(value: unknown, defaultValue: string = ''): string {
  if (typeof value === 'string') return value;
  if (value === null || value === undefined) return defaultValue;
  return String(value);
}

/**
 * 安全解析可选字符串
 */
export function parseOptionalString(value: unknown): string | undefined {
  if (typeof value === 'string' && value.length > 0) return value;
  return undefined;
}

/**
 * 解析 bitmap 为数组
 */
export function parseBitmapToArray(bitmap: number): number[] {
  const result: number[] = [];
  for (let i = 0; i < 32; i++) {
    if (bitmap & (1 << i)) {
      result.push(i);
    }
  }
  return result;
}

// ==================== 导出解析枚举的便捷函数 ====================

export const parseOtcOrderState = (value: unknown): OtcOrderState =>
  parseEnumValue(value, OTC_ORDER_STATES) ?? 'Created';

export const parseDisputeStatus = (value: unknown): DisputeStatus =>
  parseEnumValue(value, DISPUTE_STATUSES) ?? 'Initiated';

export const parseDisputeResolution = (value: unknown): DisputeResolution | undefined =>
  parseEnumValue(value, DISPUTE_RESOLUTIONS) ?? undefined;

export const parseMakerStatus = (value: unknown): MakerStatus =>
  parseEnumValue(value, MAKER_STATUSES) ?? 'Pending';

export const parseMarketOrderStatus = (value: unknown): MarketOrderStatus =>
  parseEnumValue(value, MARKET_ORDER_STATUSES) ?? 'Pending';

export const parseSwapStatus = (value: unknown): SwapStatus =>
  parseEnumValue(value, SWAP_STATUSES) ?? 'Pending';

export const parseFriendStatus = (value: unknown): FriendStatus =>
  parseEnumValue(value, FRIEND_STATUSES) ?? 'None';
