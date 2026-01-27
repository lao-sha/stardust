/**
 * 星尘玄鉴 - 服务层导出
 */

// 基础服务
export { secureStorage, SecureStorageService } from './secure-storage.service';
export { keyringService, KeyringService } from './keyring.service';
export { biometricService, BiometricService, BiometricType } from './biometric.service';
export { sessionService, SessionService } from './session.service';

// 聊天服务
export { ChatService, getChatService, initChatService } from './chat.service';

// 联系人服务
export { ContactsService, getContactsService, initContactsService } from './contacts.service';

// Bridge 服务
export { bridgeService, BridgeService } from './bridge.service';
export { SwapType, SwapStatus, BridgeType } from './bridge.service';
export type { SwapRecord, StatusCallback as BridgeStatusCallback } from './bridge.service';

// 做市商服务
export { makerService, MakerService } from './maker.service';
export {
  ApplicationStatus,
  Direction,
  WithdrawalStatus,
} from './maker.service';
export type {
  MakerApplication,
  WithdrawalRequest,
  PenaltyType,
  PenaltyRecord,
  MakerInfoInput,
} from './maker.service';

// Trading 服务
export { tradingService, TradingService } from './trading.service';

// Matchmaking 服务
export { matchmakingService, MatchmakingService } from './matchmaking.service';
export type {
  UserProfile,
  MatchPreferences,
  MatchRequest,
  StatusCallback as MatchmakingStatusCallback,
} from './matchmaking.service';
export {
  Gender,
  MatchmakingPrivacyMode,
  InteractionType,
  MatchRequestStatus,
} from './matchmaking.service';

// IPFS 存储服务
export { ipfsStorageService, IpfsStorageService } from './ipfs-storage.service';
export type { PinnedContent, SubjectAccount } from './ipfs-storage.service';
export { PinTier, PinStatus } from './ipfs-storage.service';

// 仲裁服务
export { arbitrationService, ArbitrationService } from './arbitration.service';
export type { Dispute, Evidence as ArbitrationEvidence, ArbitrationResult } from './arbitration.service';
export { DisputeStatus, DisputeType } from './arbitration.service';

// 证据存证服务
export { evidenceService, EvidenceService } from './evidence.service';
export type { Evidence } from './evidence.service';
export { EvidenceStatus, EvidenceType } from './evidence.service';

// 错误上报服务
export {
  initErrorReporting,
  captureException,
  captureMessage,
  addBreadcrumb,
  setUser,
  setTag,
  setContext,
  startTransaction,
  measureAsync,
  createRemoteLogger,
  createErrorReporter,
  isReady as isErrorReportingReady,
  isSentryAvailable,
} from './error-reporting.service';
export type { ErrorReportingConfig, UserContext, ErrorTags, ErrorExtra } from './error-reporting.service';
