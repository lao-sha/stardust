/**
 * 星尘玄鉴 - Substrate/Polkadot API 类型定义
 * 
 * 为链上数据提供类型安全的接口定义
 */

// ==================== 基础类型 ====================

/** Substrate 地址 (SS58 格式) */
export type SubstrateAddress = string;

/** 区块哈希 */
export type BlockHash = string;

/** 交易哈希 */
export type TxHash = string;

/** 区块号 */
export type BlockNumber = number;

/** 时间戳（毫秒） */
export type Timestamp = number;

/** 余额（最小单位，精度 10^12） */
export type Balance = bigint;

/** 价格（精度 10^6） */
export type Price = bigint;

// ==================== 账户相关 ====================

/**
 * 账户数据
 */
export interface AccountData {
  free: string;
  reserved: string;
  frozen: string;
}

/**
 * 账户信息
 */
export interface AccountInfo {
  nonce: number;
  consumers: number;
  providers: number;
  sufficients: number;
  data: AccountData;
}

/**
 * 账户余额信息（解析后）
 */
export interface ParsedAccountBalance {
  free: Balance;
  reserved: Balance;
  frozen: Balance;
  total: Balance;
}

// ==================== 交易相关 ====================

/**
 * 交易状态
 */
export type TxStatus = 
  | 'ready'
  | 'broadcast'
  | 'inBlock'
  | 'finalized'
  | 'invalid'
  | 'dropped';

/**
 * 交易结果
 */
export interface TxResult {
  status: TxStatus;
  blockHash?: BlockHash;
  txHash?: TxHash;
  events: ChainEvent[];
  error?: string;
}

/**
 * 链上事件
 */
export interface ChainEvent {
  section: string;
  method: string;
  data: unknown[];
}

/**
 * Dispatch 错误
 */
export interface DispatchError {
  isModule: boolean;
  asModule?: {
    index: number;
    error: Uint8Array;
  };
  toString(): string;
}

// ==================== OTC 交易相关 ====================

/**
 * OTC 订单状态
 */
export type OtcOrderState = 
  | 'Created'
  | 'Paid'
  | 'Released'
  | 'Cancelled'
  | 'Disputed'
  | 'Resolved'
  | 'Expired';

/**
 * OTC 订单（链上原始数据）
 */
export interface RawOtcOrder {
  makerId: number;
  maker: string;
  taker: string;
  price: string;
  qty: string;
  amount: string;
  createdAt: number;
  expireAt: number;
  makerTronAddress: string;
  state: OtcOrderState | { [key: string]: null };
  isFirstPurchase: boolean | { isTrue?: boolean };
}

/**
 * OTC 订单（解析后）
 */
export interface ParsedOtcOrder {
  id: number;
  makerId: number;
  maker: SubstrateAddress;
  taker: SubstrateAddress;
  price: Price;
  qty: Balance;
  amount: Price;
  createdAt: Timestamp;
  expireAt: Timestamp;
  makerTronAddress: string;
  state: OtcOrderState;
  isFirstPurchase: boolean;
}

/**
 * 争议信息（链上原始数据）
 */
export interface RawDispute {
  initiator: string;
  reason: string;
  initiatorEvidence?: string;
  respondentResponse?: string;
  respondentEvidence?: string;
  status: string | { [key: string]: null };
  deadline: number;
  resolution?: string | { [key: string]: null };
}

/**
 * 争议状态
 */
export type DisputeStatus = 'Initiated' | 'Responded' | 'Resolved';

/**
 * 争议解决结果
 */
export type DisputeResolution = 'BuyerWins' | 'MakerWins' | 'Split';

/**
 * 争议信息（解析后）
 */
export interface ParsedDispute {
  initiator: SubstrateAddress;
  reason: string;
  initiatorEvidence?: string;
  respondentResponse?: string;
  respondentEvidence?: string;
  status: DisputeStatus;
  deadline: Timestamp;
  resolution?: DisputeResolution;
}

// ==================== 做市商相关 ====================

/**
 * 做市商申请状态
 */
export type MakerStatus = 'Pending' | 'Active' | 'Suspended' | 'Exited';

/**
 * 做市商申请（链上原始数据）
 */
export interface RawMakerApplication {
  owner: string;
  tronAddress: string;
  buyPremiumBps: number;
  sellPremiumBps: number;
  minAmount: string;
  servicePaused: boolean | { isTrue?: boolean };
  usersServed: number;
  maskedFullName: string;
  wechatId: string;
  status: MakerStatus | { [key: string]: null };
}

/**
 * 做市商信息（解析后）
 */
export interface ParsedMaker {
  id: number;
  owner: SubstrateAddress;
  tronAddress: string;
  buyPremiumBps: number;
  sellPremiumBps: number;
  minAmount: Balance;
  servicePaused: boolean;
  usersServed: number;
  maskedFullName: string;
  wechatId: string;
  status: MakerStatus;
  rating: number;
}

// ==================== 占卜市场相关 ====================

/**
 * 占卜类型
 */
export type DivinationType = 
  | 0  // 八字
  | 1  // 紫微
  | 2  // 奇门
  | 3  // 六爻
  | 4  // 梅花
  | 5  // 塔罗
  | 6  // 大六壬
  | 7; // 小六壬

/**
 * 解卦师等级
 */
export type ProviderTier = 0 | 1 | 2 | 3 | 4;

/**
 * 订单状态
 */
export type MarketOrderStatus = 
  | 'Pending'
  | 'Accepted'
  | 'Completed'
  | 'Cancelled'
  | 'Refunded'
  | 'Disputed';

/**
 * 解卦师信息（链上原始数据）
 */
export interface RawProvider {
  name: string;
  bio: string;
  avatarCid?: string;
  divinationTypes: number;
  specialties: number;
  tier: number;
  totalOrders: number;
  completedOrders: number;
  averageRating: number;
  totalEarnings: string;
  availableBalance: string;
  isActive: boolean;
  registeredAt: number;
}

/**
 * 解卦师信息（解析后）
 */
export interface ParsedProvider {
  id: SubstrateAddress;
  address: SubstrateAddress;
  name: string;
  bio: string;
  avatarCid?: string;
  divinationTypes: DivinationType[];
  specialties: number[];
  tier: ProviderTier;
  totalOrders: number;
  completedOrders: number;
  averageRating: number;
  totalEarnings: Balance;
  availableBalance: Balance;
  isActive: boolean;
  registeredAt: Timestamp;
}

/**
 * 服务套餐（链上原始数据）
 */
export interface RawServicePackage {
  providerId: string;
  name: string;
  description: string;
  divinationType: number;
  price: string;
  deliveryDays: number;
  maxFollowUps: number;
  isActive: boolean;
  totalOrders?: number;
  createdAt?: number;
}

/**
 * 服务套餐（解析后）
 */
export interface ParsedServicePackage {
  id: number;
  providerId: SubstrateAddress;
  name: string;
  description: string;
  divinationType: DivinationType;
  price: Balance;
  deliveryDays: number;
  maxFollowUps: number;
  isActive: boolean;
  totalOrders: number;
  createdAt: Timestamp;
}

/**
 * 市场订单（链上原始数据）
 */
export interface RawMarketOrder {
  id: number;
  clientId: string;
  providerId: string;
  packageId: number;
  status: MarketOrderStatus | { [key: string]: null };
  questionCid: string;
  resultCid?: string;
  hexagramData?: string;
  price: string;
  createdAt: number;
  acceptedAt?: number;
  completedAt?: number;
  deliveryDeadline?: number;
  followUps?: RawFollowUp[];
}

/**
 * 追问（链上原始数据）
 */
export interface RawFollowUp {
  questionCid: string;
  answerCid?: string;
  createdAt: number;
  answeredAt?: number;
}

/**
 * 市场订单（解析后）
 */
export interface ParsedMarketOrder {
  id: number;
  clientId: SubstrateAddress;
  providerId: SubstrateAddress;
  packageId: number;
  status: MarketOrderStatus;
  questionCid: string;
  resultCid?: string;
  hexagramData?: string;
  price: Balance;
  createdAt: Timestamp;
  acceptedAt?: Timestamp;
  completedAt?: Timestamp;
  deliveryDeadline?: Timestamp;
  followUps: ParsedFollowUp[];
}

/**
 * 追问（解析后）
 */
export interface ParsedFollowUp {
  questionCid: string;
  answerCid?: string;
  createdAt: Timestamp;
  answeredAt?: Timestamp;
}

/**
 * 评价（链上原始数据）
 */
export interface RawReview {
  orderId: number;
  clientId: string;
  providerId: string;
  accuracy: number;
  attitude: number;
  speed: number;
  value: number;
  contentCid?: string;
  replyCid?: string;
  isAnonymous: boolean;
  createdAt: number;
  repliedAt?: number;
}

/**
 * 评价评分
 */
export interface ReviewRatings {
  accuracy: number;
  attitude: number;
  speed: number;
  value: number;
}

/**
 * 评价（解析后）
 */
export interface ParsedReview {
  id: number;
  orderId: number;
  clientId: SubstrateAddress;
  providerId: SubstrateAddress;
  ratings: ReviewRatings;
  contentCid?: string;
  replyCid?: string;
  isAnonymous: boolean;
  createdAt: Timestamp;
  repliedAt?: Timestamp;
}

// ==================== 市场统计 ====================

/**
 * 市场统计（链上原始数据）
 */
export interface RawMarketStats {
  otcPrice: number;
  bridgePrice: number;
  weightedPrice: number;
  simpleAvgPrice: number;
  otcVolume: string;
  bridgeVolume: string;
  totalVolume: string;
}

/**
 * 市场统计（解析后）
 */
export interface ParsedMarketStats {
  otcPrice: number;
  bridgePrice: number;
  weightedPrice: number;
  simpleAvgPrice: number;
  otcVolume: Balance;
  bridgeVolume: Balance;
  totalVolume: Balance;
  priceChange24h?: number;
}

// ==================== 信用系统 ====================

/**
 * 买家信用（链上原始数据）
 */
export interface RawBuyerCredit {
  riskScore: number;
  maxAmount: number;
  concurrentOrders: number;
  maxConcurrentOrders: number;
  completedOrders: number;
}

/**
 * 信用等级
 */
export type CreditLevel = '高信任' | '中信任' | '低信任' | '极低信任' | '新用户';

/**
 * 信用趋势
 */
export type CreditTrend = 'up' | 'down' | 'stable';

/**
 * 买家信用（解析后）
 */
export interface ParsedBuyerCredit {
  riskScore: number;
  level: CreditLevel;
  maxAmount: number;
  concurrentOrders: number;
  maxConcurrentOrders: number;
  completedOrders: number;
  trend: CreditTrend;
}

// ==================== KYC 相关 ====================

/**
 * KYC 状态
 */
export type KycStatus = 'Passed' | 'Failed' | 'Skipped' | 'Exempted';

/**
 * KYC 失败原因
 */
export type KycFailureReason = 
  | 'IdentityNotSet'
  | 'NoValidJudgement'
  | 'InsufficientLevel'
  | 'QualityIssue';

/**
 * KYC 检查结果
 */
export interface KycCheckResult {
  status: KycStatus;
  failureReason: KycFailureReason | null;
}

// ==================== 兑换相关 ====================

/**
 * 兑换状态
 */
export type SwapStatus = 
  | 'Pending'
  | 'Completed'
  | 'Timeout'
  | 'Reported'
  | 'Refunded';

/**
 * 兑换记录（链上原始数据）
 */
export interface RawSwapRecord {
  buyer: string;
  makerId: number;
  dustAmount: string;
  usdtAmount: string;
  buyerTronAddress: string;
  makerTronAddress: string;
  status: SwapStatus | { [key: string]: null };
  timeInfo?: {
    createdAt: number;
    completedAt?: number;
  };
  tronTxHash?: string;
}

/**
 * 兑换记录（解析后）
 */
export interface ParsedSwapRecord {
  id: number;
  buyer: SubstrateAddress;
  makerId: number;
  dustAmount: Balance;
  usdtAmount: Price;
  buyerTronAddress: string;
  makerTronAddress: string;
  status: SwapStatus;
  createdAt: Timestamp;
  completedAt?: Timestamp;
  tronTxHash?: string;
}

// ==================== 聊天相关 ====================

/**
 * 聊天用户 ID
 */
export type ChatUserId = number;

/**
 * 用户状态
 */
export type UserStatus = 'online' | 'offline' | 'busy' | 'away';

/**
 * 消息类型
 */
export type MessageType = 0 | 1 | 2 | 3; // 文本、图片、文件、系统

/**
 * 消息状态
 */
export type MessageStatus = 'sending' | 'sent' | 'failed';

// ==================== 联系人相关 ====================

/**
 * 好友状态
 */
export type FriendStatus = 
  | 'None'
  | 'Pending'
  | 'Accepted'
  | 'Blocked';

/**
 * 联系人信息（链上原始数据）
 */
export interface RawContact {
  alias?: string;
  groups: string[];
  friendStatus: FriendStatus | { [key: string]: null };
  addedAt: number;
}

/**
 * 联系人信息（解析后）
 */
export interface ParsedContact {
  address: SubstrateAddress;
  alias?: string;
  groups: string[];
  friendStatus: FriendStatus;
  addedAt: Timestamp;
}
