// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

// Substrate and Polkadot dependencies
use sp_runtime::traits::AccountIdConversion;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstBool, ConstU128, ConstU16, ConstU32, ConstU64, ConstU8, VariantCountOf, EitherOfDiverse},
	weights::{
		constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		IdentityFee, Weight,
	},
};
use frame_system::{limits::{BlockLength, BlockWeights}, EnsureRoot};
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::{traits::One, Perbill};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
	AccountId, Aura, Balance, Balances, Block, BlockNumber, Hash, Nonce, PalletInfo, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask,
	System, EXISTENTIAL_DEPOSIT, SLOT_DURATION, VERSION, UNIT, MINUTES, HOURS, DAYS,
	TechnicalCommittee, ArbitrationCommittee, TreasuryCouncil, ContentCommittee,
};

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;

	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
		Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
		NORMAL_DISPATCH_RATIO,
	);
	pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@frame_system::config_preludes::SolochainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
	/// The block type for the runtime.
	type Block = Block;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<32>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type DoneSlashHandler = ();
}

parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
	type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

// ============================================================================
// Divination Pallets Configuration
// ============================================================================

// -------------------- Almanac (黄历) --------------------

parameter_types! {
	pub const MaxBatchSize: u32 = 31;
	pub const MaxHistoryYears: u32 = 3;
}

impl pallet_almanac::Config for Runtime {
	type WeightInfo = ();
	type MaxBatchSize = MaxBatchSize;
	type MaxHistoryYears = MaxHistoryYears;
}

// -------------------- Privacy (隐私授权) --------------------

parameter_types! {
	pub const MaxEncryptedDataLen: u32 = 4096;
	pub const MaxEncryptedKeyLen: u32 = 256;
	pub const MaxGranteesPerRecord: u32 = 100;
	pub const MaxRecordsPerUser: u32 = 10000;
	pub const MaxProvidersPerType: u32 = 10000;
	pub const MaxGrantsPerProvider: u32 = 1000;
	pub const MaxAuthorizationsPerBounty: u32 = 100;
}

impl pallet_divination_privacy::Config for Runtime {
	type MaxEncryptedDataLen = MaxEncryptedDataLen;
	type MaxEncryptedKeyLen = MaxEncryptedKeyLen;
	type MaxGranteesPerRecord = MaxGranteesPerRecord;
	type MaxRecordsPerUser = MaxRecordsPerUser;
	type MaxProvidersPerType = MaxProvidersPerType;
	type MaxGrantsPerProvider = MaxGrantsPerProvider;
	type MaxAuthorizationsPerBounty = MaxAuthorizationsPerBounty;
	type EventHandler = ();
	type WeightInfo = ();
}

// -------------------- TEE Privacy (TEE 隐私计算) --------------------

parameter_types! {
	/// TEE 节点认证有效期 (约 24 小时)
	pub const AttestationValidity: u32 = 14400;
	/// 计算请求超时区块数 (约 10 分钟)
	pub const TeeRequestTimeout: u32 = 100;
	/// TEE 节点惩罚比例 (10%)
	pub const TeeSlashRatio: u32 = 100;
}

impl pallet_tee_privacy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MaxNodes = ConstU32<100>;
	type MaxPendingRequests = ConstU32<10000>;
	type AttestationValidity = AttestationValidity;
	type MaxAllowedMrEnclaves = ConstU32<50>;
	type RequestTimeout = TeeRequestTimeout;
	type MinimumStake = ConstU128<{ 100 * UNIT }>;
	type BaseComputeFee = ConstU128<{ UNIT / 10 }>;
	type SlashRatio = TeeSlashRatio;
	type MaxBatchSize = ConstU32<100>;
	type WeightInfo = pallet_tee_privacy::weights::SubstrateWeight<Runtime>;
}

// -------------------- 全局系统账户（简化方案：4 个核心账户）--------------------

parameter_types! {
	// 1. 国库账户 - 核心账户，含平台收入、存储补贴
	pub const TreasuryPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/trsry");
	pub TreasuryAccountId: AccountId = TreasuryPalletId::get().into_account_truncating();
	
	// 2. 销毁账户 - 专用于代币销毁，必须独立
	pub const BurnPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/burn!");
	pub BurnAccountId: AccountId = BurnPalletId::get().into_account_truncating();
}


// -------------------- AI 解读模块 --------------------

impl pallet_divination_ai::Config for Runtime {
	type AiCurrency = Balances;
	type DivinationProvider = pallet_divination_common::NullDivinationProvider;
	type ContentRegistry = pallet_storage_service::Pallet<Runtime>;
	type BaseInterpretationFee = ConstU128<{ 1 * UNIT }>;
	type MinOracleStake = ConstU128<{ 10 * UNIT }>;
	type DisputeDeposit = ConstU128<{ UNIT / 2 }>;
	type DisputeDepositUsd = ConstU64<1_000_000>; // 1 USDT
	type DepositCalculator = pallet_trading_common::DepositCalculatorImpl<TradingPricingProvider, Balance>;
	type RequestTimeout = ConstU32<{ 10 * MINUTES }>;
	type ProcessingTimeout = ConstU32<{ 5 * MINUTES }>;
	type DisputePeriod = ConstU32<{ 1 * HOURS }>;
	type MaxCidLength = ConstU32<128>;
	type MaxOracles = ConstU32<100>;
	type TreasuryAccount = TreasuryAccountId;
	type ArbitratorOrigin = frame_system::EnsureRoot<AccountId>;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
}

// -------------------- Market (服务市场) --------------------

impl pallet_divination_market::Config for Runtime {
	type Currency = Balances;
	type DivinationProvider = pallet_divination_common::NullDivinationProvider;
	type ContentRegistry = pallet_storage_service::Pallet<Runtime>;
	type MinDeposit = ConstU128<{ 10 * UNIT }>;  // 最低保证金 10 DUST（兜底值）
	type MinDepositUsd = ConstU64<100_000_000>;  // 最低保证金 100 USDT（精度10^6，使用pricing换算）
	type Pricing = TradingPricingProvider;  // 定价接口
	type MinServicePrice = ConstU128<{ UNIT / 10 }>;
	type MaxServicePrice = ConstU128<{ 100_000_000 * UNIT }>;  // 修复 H-13: 最大服务价格 1亿 DUST
	type OrderTimeout = ConstU32<{ 24 * HOURS }>;
	type AcceptTimeout = ConstU32<{ 1 * HOURS }>;
	type ReviewPeriod = ConstU32<{ 7 * DAYS }>;
	type WithdrawalCooldown = ConstU32<{ 1 * HOURS }>;
	type MaxNameLength = ConstU32<64>;
	type MaxBioLength = ConstU32<256>;
	type MaxDescriptionLength = ConstU32<512>;
	type MaxCidLength = ConstU32<64>;
	type MaxPackagesPerProvider = ConstU32<10>;
	type MaxFollowUpsPerOrder = ConstU32<5>;
	type PlatformAccount = TreasuryAccountId;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
	type TreasuryAccount = TreasuryAccountId;
	// 🆕 联盟计酬集成 - Using stub until pallet_affiliate is integrated
	type AffiliateDistributor = StubAffiliateDistributor;
	// 🆕 解读修改窗口（2天 ≈ 28800 blocks，按6秒/块）
	type InterpretationEditWindow = ConstU32<28800>;
	// 🆕 聊天权限集成（订单创建时自动授权双方聊天）
	type ChatPermission = pallet_chat_permission::Pallet<Runtime>;
	// 🆕 订单聊天授权有效期（30天 ≈ 432000 blocks，按6秒/块）
	type OrderChatDuration = ConstU32<{ 30 * DAYS }>;
}

// Stub implementation for AffiliateDistributor until pallet_affiliate is integrated
pub struct StubAffiliateDistributor;

impl pallet_affiliate::types::AffiliateDistributor<AccountId, u128, BlockNumber> for StubAffiliateDistributor {
	fn distribute_rewards(
		_buyer: &AccountId,
		_amount: u128,
		_target: Option<(u8, u64)>,
	) -> Result<u128, sp_runtime::DispatchError> {
		Ok(0)
	}
}

// UserFundingProvider 实现 - 使用存储服务模块的派生账户
pub struct StorageUserFundingProvider;

impl pallet_affiliate::UserFundingProvider<AccountId> for StorageUserFundingProvider {
	fn derive_user_funding_account(user: &AccountId) -> AccountId {
		pallet_storage_service::Pallet::<Runtime>::derive_user_funding_account(user)
	}
}

// -------------------- NFT 模块 --------------------

impl pallet_divination_nft::Config for Runtime {
	type NftCurrency = Balances;
	type DivinationProvider = pallet_divination_common::NullDivinationProvider;
	type ContentRegistry = pallet_storage_service::Pallet<Runtime>;
	type MaxNameLength = ConstU32<64>;
	type MaxCidLength = ConstU32<128>;
	type MaxCollectionsPerUser = ConstU32<50>;
	type MaxNftsPerCollection = ConstU32<1000>;
	type MaxOffersPerNft = ConstU32<100>;
	type BaseMintFee = ConstU128<UNIT>;
	type PlatformFeeRate = ConstU16<250>; // 2.5%
	type MaxRoyaltyRate = ConstU16<2500>; // 25%
	type OfferValidityPeriod = ConstU32<{ 7 * DAYS }>;
	type PlatformAccount = TreasuryAccountId;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
}

// -------------------- Meihua (梅花易数) --------------------

/// 安全随机数生成器 - 基于 Collective Coin Flipping 机制
/// 
/// 原理：
/// - 结合多个历史区块哈希（81个区块，对应九宫格 9x9）
/// - 混合当前区块信息和用户提供的 subject
/// - 使用 blake2_256 进行哈希混合
/// 
/// 安全性：
/// - 单个验证者无法预测或操控结果
/// - 需要控制连续 81 个区块才能完全操控（在 Aura 共识下极难实现）
/// - 适用于占卜、抽签等非金融高价值场景
/// 
/// 注意：对于需要更高安全性的场景（如大额抽奖），建议使用 VRF 或 Commit-Reveal 机制
pub struct CollectiveFlipRandomness;

impl frame_support::traits::Randomness<Hash, BlockNumber> for CollectiveFlipRandomness {
	fn random(subject: &[u8]) -> (Hash, BlockNumber) {
		let block_number = System::block_number();
		
		// 收集最近 81 个区块的哈希（九宫格 9x9，占卜意义）
		// 如果区块数不足，则使用可用的区块
		let mut combined_entropy = alloc::vec::Vec::with_capacity(81 * 32 + subject.len() + 8);
		
		// 添加 subject 作为熵源
		combined_entropy.extend_from_slice(subject);
		
		// 添加当前区块号
		combined_entropy.extend_from_slice(&block_number.to_le_bytes());
		
		// 收集历史区块哈希
		let blocks_to_collect = core::cmp::min(block_number.saturating_sub(1), 81);
		for i in 1..=blocks_to_collect {
			let hash = System::block_hash(block_number.saturating_sub(i as u32));
			combined_entropy.extend_from_slice(hash.as_ref());
		}
		
		// 添加父区块哈希作为额外熵源
		let parent_hash = System::parent_hash();
		combined_entropy.extend_from_slice(parent_hash.as_ref());
		
		// 使用 blake2_256 生成最终随机值
		let final_hash = sp_core::hashing::blake2_256(&combined_entropy);
		
		(Hash::from_slice(&final_hash), block_number)
	}
}

impl pallet_meihua::Config for Runtime {
	type Currency = Balances;
	type Randomness = CollectiveFlipRandomness;
	type MaxUserHexagrams = ConstU32<1000>;
	type MaxPublicHexagrams = ConstU32<10000>;
	type DailyFreeDivinations = ConstU32<3>;
	type MaxDailyDivinations = ConstU32<100>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
}

// -------------------- Bazi (八字) --------------------

// Temporary placeholder for PrivacyProvider until full integration is complete
pub struct BaziPrivacyProvider;

impl pallet_divination_privacy::traits::EncryptedRecordManager<AccountId, BlockNumber> for BaziPrivacyProvider {
	fn create_record(
		_owner: &AccountId,
		_divination_type: pallet_divination_common::DivinationType,
		_result_id: u64,
		_privacy_mode: pallet_divination_privacy::types::PrivacyMode,
		_encrypted_data: alloc::vec::Vec<u8>,
		_nonce: [u8; 24],
		_auth_tag: [u8; 16],
		_data_hash: [u8; 32],
		_owner_encrypted_key: alloc::vec::Vec<u8>,
	) -> frame_support::dispatch::DispatchResult {
		Ok(())
	}

	fn delete_record(
		_owner: &AccountId,
		_divination_type: pallet_divination_common::DivinationType,
		_result_id: u64,
	) -> frame_support::dispatch::DispatchResult {
		Ok(())
	}

	fn grant_access(
		_grantor: &AccountId,
		_divination_type: pallet_divination_common::DivinationType,
		_result_id: u64,
		_grantee: &AccountId,
		_encrypted_key: alloc::vec::Vec<u8>,
		_role: pallet_divination_privacy::types::AccessRole,
		_scope: pallet_divination_privacy::types::AccessScope,
		_expires_at: BlockNumber,
		_bounty_id: Option<u64>,
	) -> frame_support::dispatch::DispatchResult {
		Ok(())
	}

	fn revoke_access(
		_grantor: &AccountId,
		_divination_type: pallet_divination_common::DivinationType,
		_result_id: u64,
		_grantee: &AccountId,
	) -> frame_support::dispatch::DispatchResult {
		Ok(())
	}
}

impl pallet_bazi_chart::Config for Runtime {
	type WeightInfo = ();
	type MaxChartsPerAccount = ConstU32<100>;
	type MaxDaYunSteps = ConstU32<12>;
	type MaxCangGan = ConstU32<3>;
	type PrivacyProvider = BaziPrivacyProvider;
}

// -------------------- Liuyao (六爻) --------------------

impl pallet_liuyao::Config for Runtime {
	type Currency = Balances;
	type Randomness = CollectiveFlipRandomness;
	type MaxUserGuas = ConstU32<1000>;
	type MaxPublicGuas = ConstU32<10000>;
	type DailyFreeGuas = ConstU32<3>;
	type MaxDailyGuas = ConstU32<100>;
	type MaxCidLen = ConstU32<64>;
	type MaxEncryptedLen = ConstU32<512>;
}

// -------------------- Qimen (奇门遁甲) --------------------

impl pallet_qimen::Config for Runtime {
	type Currency = Balances;
	type Randomness = CollectiveFlipRandomness;
	type MaxUserCharts = ConstU32<1000>;
	type MaxPublicCharts = ConstU32<10000>;
	type DailyFreeCharts = ConstU32<3>;
	type MaxDailyCharts = ConstU32<100>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxCidLen = ConstU32<64>;
	type MaxEncryptedLen = ConstU32<512>;
}

// -------------------- Ziwei (紫微斗数) --------------------

impl pallet_ziwei::Config for Runtime {
	type Currency = Balances;
	type Randomness = CollectiveFlipRandomness;
	type MaxUserCharts = ConstU32<1000>;
	type MaxPublicCharts = ConstU32<10000>;
	type DailyFreeCharts = ConstU32<3>;
	type MaxDailyCharts = ConstU32<100>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxCidLen = ConstU32<64>;
	type MaxEncryptedLen = ConstU32<512>;
}

// -------------------- Xiaoliuren (小六壬) --------------------

impl pallet_xiaoliuren::Config for Runtime {
	type Currency = Balances;
	type Randomness = CollectiveFlipRandomness;
	type MaxUserPans = ConstU32<1000>;
	type MaxPublicPans = ConstU32<10000>;
	type MaxCidLen = ConstU32<64>;
	type DailyFreeDivinations = ConstU32<10>;
	type MaxDailyDivinations = ConstU32<100>;
	type MaxEncryptedLen = ConstU32<512>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
}

// -------------------- Daliuren (大六壬) --------------------

impl pallet_daliuren::Config for Runtime {
	type Currency = Balances;
	type Randomness = CollectiveFlipRandomness;
	type MaxCidLen = ConstU32<64>;
	type MaxDailyDivinations = ConstU32<50>;
	type MaxEncryptedLen = ConstU32<512>;
	type DivinationFee = ConstU128<UNIT>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type AiSubmitter = frame_system::EnsureSigned<AccountId>;
	type WeightInfo = ();
}

// -------------------- Tarot (塔罗牌) --------------------

impl pallet_tarot::Config for Runtime {
	type Currency = Balances;
	type Randomness = CollectiveFlipRandomness;
	type MaxCardsPerReading = ConstU32<12>;
	type MaxUserReadings = ConstU32<1000>;
	type MaxPublicReadings = ConstU32<10000>;
	type DailyFreeDivinations = ConstU32<3>;
	type MaxDailyDivinations = ConstU32<100>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
}

// ============================================================================
// Chat Pallets Configuration
// ============================================================================

// -------------------- Chat Permission (聊天权限) --------------------

impl pallet_chat_permission::Config for Runtime {
	type MaxBlockListSize = ConstU32<1000>;
	type MaxWhitelistSize = ConstU32<1000>;
	type MaxScenesPerPair = ConstU32<50>;
}

// -------------------- Chat Core (私聊核心) --------------------

/// 时间戳提供器 - 使用 pallet_timestamp
pub struct TimestampProvider;

impl frame_support::traits::UnixTime for TimestampProvider {
	fn now() -> core::time::Duration {
		let millis = pallet_timestamp::Pallet::<Runtime>::get();
		core::time::Duration::from_millis(millis)
	}
}

impl pallet_chat_core::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_chat_core::SubstrateWeight<Runtime>;
	type MaxCidLen = ConstU32<128>;
	type MaxSessionsPerUser = ConstU32<1000>;
	type MaxMessagesPerSession = ConstU32<10000>;
	type RateLimitWindow = ConstU32<100>;
	type MaxMessagesPerWindow = ConstU32<50>;
	type MessageExpirationTime = ConstU32<{ 180 * DAYS }>;
	type Randomness = CollectiveFlipRandomness;
	type UnixTime = TimestampProvider;
	type MaxNicknameLength = ConstU32<64>;
	type MaxSignatureLength = ConstU32<256>;
}

// -------------------- Chat Group (群聊) --------------------

parameter_types! {
	pub const ChatGroupPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/chatg");
}

parameter_types! {
	pub const GroupDeposit: Balance = 50 * UNIT; // 创建群组保证金兜底值 50 DUST
	pub const GroupDepositUsd: u64 = 5_000_000; // 创建群组保证金 5 USDT（精度10^6）
}

impl pallet_chat_group::Config for Runtime {
	type Randomness = CollectiveFlipRandomness;
	type TimeProvider = TimestampProvider;
	type Currency = Balances;
	type MaxGroupNameLen = ConstU32<64>;
	type MaxGroupDescriptionLen = ConstU32<256>;
	type MaxGroupMembers = ConstU32<1000>;
	type MaxGroupsPerUser = ConstU32<100>;
	type MaxMessageLen = ConstU32<4096>;
	type MaxGroupMessageHistory = ConstU32<10000>;
	type MaxCidLen = ConstU32<128>;
	type MaxKeyLen = ConstU32<256>;
	type PalletId = ChatGroupPalletId;
	type MessageRateLimit = ConstU32<60>; // 每分钟最多60条消息
	type GroupCreationCooldown = ConstU32<{ 10 * MINUTES }>; // 创建群组冷却时间
	type GroupDeposit = GroupDeposit;
	type GroupDepositUsd = GroupDepositUsd;
	type DepositCalculator = pallet_trading_common::DepositCalculatorImpl<TradingPricingProvider, Balance>;
	type TreasuryAccount = TreasuryAccountId;
	type GovernanceOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

// -------------------- Livestream (直播间) --------------------

parameter_types! {
	pub const LivestreamPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/lives");
}

impl pallet_livestream::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MaxTitleLen = ConstU32<100>;
	type MaxDescriptionLen = ConstU32<500>;
	type MaxCidLen = ConstU32<64>;
	type MaxGiftNameLen = ConstU32<32>;
	type MaxCoHostsPerRoom = ConstU32<4>;
	type PlatformFeePercent = ConstU8<20>; // 20% 平台抽成
	type MinWithdrawAmount = ConstU128<{ 1 * UNIT }>; // 最小提现 1 DUST
	type RoomBond = ConstU128<{ UNIT / 20 }>; // 创建直播间保证金兜底值 0.05 DUST
	type RoomBondUsd = ConstU64<5_000_000>; // 创建直播间保证金 5 USDT
	type DepositCalculator = pallet_trading_common::DepositCalculatorImpl<TradingPricingProvider, Balance>;
	type PalletId = LivestreamPalletId;
	// 🆕 封禁权限：内容委员会 1/2 多数
	type GovernanceOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, ContentCollectiveInstance, 1, 2>;
	type WeightInfo = ();
}

// ============================================================================
// Trading Pallets Configuration
// ============================================================================

// -------------------- Pricing (价格预言机) --------------------

impl pallet_trading_pricing::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxPriceDeviation = ConstU16<2000>; // 20% 最大价格偏离
	type ExchangeRateUpdateInterval = ConstU32<{ 24 * HOURS }>; // 24小时更新汇率
}

// -------------------- Credit (信用风控) --------------------

impl pallet_trading_credit::Config for Runtime {
	type Currency = Balances;
	// 买家信用配置
	type InitialBuyerCreditScore = ConstU16<500>;
	type OrderCompletedBonus = ConstU16<10>;
	type OrderDefaultPenalty = ConstU16<50>;
	type BlocksPerDay = ConstU32<{ DAYS }>;
	type MinimumBalance = ConstU128<{ 100 * UNIT }>;
	// 做市商信用配置
	type InitialMakerCreditScore = ConstU16<820>;
	type MakerOrderCompletedBonus = ConstU16<2>;
	type MakerOrderTimeoutPenalty = ConstU16<10>;
	type MakerDisputeLossPenalty = ConstU16<20>;
	type MakerSuspensionThreshold = ConstU16<750>;
	type MakerWarningThreshold = ConstU16<800>;
	type CreditWeightInfo = ();
}

// -------------------- Maker (做市商管理) --------------------

/// Pricing Provider 实现 - 统一实现 pallet_trading_common::PricingProvider
pub struct TradingPricingProvider;

impl pallet_trading_common::PricingProvider<Balance> for TradingPricingProvider {
	fn get_dust_to_usd_rate() -> Option<Balance> {
		let price = pallet_trading_pricing::Pallet::<Runtime>::get_dust_market_price_weighted();
		if price > 0 {
			Some(price as Balance)
		} else {
			None
		}
	}
	
	fn report_swap_order(timestamp: u64, price_usdt: u64, dust_qty: u128) -> sp_runtime::DispatchResult {
		pallet_trading_pricing::Pallet::<Runtime>::add_swap_order(timestamp, price_usdt, dust_qty)
	}
}

impl pallet_trading_maker::Config for Runtime {
	type Currency = Balances;
	type MakerCredit = pallet_trading_credit::Pallet<Runtime>;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
	type Timestamp = TimestampProvider;
	type MakerDepositAmount = ConstU128<{ 1000 * UNIT }>;
	type TargetDepositUsd = ConstU64<1000_000_000>; // 1000 USD
	type DepositReplenishThreshold = ConstU64<950_000_000>; // 950 USD
	type DepositReplenishTarget = ConstU64<1050_000_000>; // 1050 USD
	type PriceCheckInterval = ConstU32<{ HOURS }>; // 每小时检查
	type AppealDeadline = ConstU32<{ 7 * DAYS }>; // 7天申诉期
	type Pricing = TradingPricingProvider;
	type MakerApplicationTimeout = ConstU32<{ 7 * DAYS }>;
	type WithdrawalCooldown = ConstU32<{ 7 * DAYS }>;
	type ContentRegistry = pallet_storage_service::Pallet<Runtime>;
	type WeightInfo = ();
	type TreasuryAccount = TreasuryAccountId; // 国库账户
}

// -------------------- Bridge (桥接服务) --------------------

/// Bridge Maker 接口适配器
pub struct BridgeMakerAdapter;

impl pallet_trading_common::MakerInterface<AccountId, Balance> for BridgeMakerAdapter {
	fn get_maker_application(maker_id: u64) -> Option<pallet_trading_common::MakerApplicationInfo<AccountId, Balance>> {
		pallet_trading_maker::Pallet::<Runtime>::maker_applications(maker_id).map(|app| {
			pallet_trading_common::MakerApplicationInfo {
				account: app.owner,
				tron_address: app.tron_address,
				is_active: app.status == pallet_trading_maker::pallet::ApplicationStatus::Active,
				_phantom: core::marker::PhantomData,
			}
		})
	}

	fn is_maker_active(maker_id: u64) -> bool {
		pallet_trading_maker::Pallet::<Runtime>::is_maker_active(maker_id)
	}

	fn get_maker_id(who: &AccountId) -> Option<u64> {
		pallet_trading_maker::Pallet::<Runtime>::get_maker_id(who)
	}

	fn get_deposit_usd_value(maker_id: u64) -> Result<u64, sp_runtime::DispatchError> {
		pallet_trading_maker::Pallet::<Runtime>::get_deposit_usd_value(maker_id)
	}
}

/// Bridge Credit 接口适配器
pub struct BridgeCreditAdapter;

impl pallet_trading_common::MakerCreditInterface for BridgeCreditAdapter {
	fn record_maker_order_completed(maker_id: u64, order_id: u64, response_time_seconds: u32) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_order_completed(maker_id, order_id, response_time_seconds)
	}

	fn record_maker_order_timeout(maker_id: u64, order_id: u64) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_order_timeout(maker_id, order_id)
	}

	fn record_maker_dispute_result(maker_id: u64, order_id: u64, maker_win: bool) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_dispute_result(maker_id, order_id, maker_win)
	}
}

impl pallet_trading_swap::Config for Runtime {
	type Currency = Balances;
	type Escrow = pallet_escrow::Pallet<Runtime>;
	type Pricing = TradingPricingProvider;
	type MakerPallet = BridgeMakerAdapter;
	type Credit = BridgeCreditAdapter;
	type OcwSwapTimeoutBlocks = ConstU32<{ 1 * HOURS }>; // OCW 1小时超时
	// 🆕 2026-01-20: TRC20 验证超时时间（2小时）
	type VerificationTimeoutBlocks = ConstU32<{ 2 * HOURS }>;
	// 🆕 2026-01-20: 验证权限（理事会 2/3 多数或 Root）
	type VerificationOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, pallet_collective::Instance1, 2, 3>,
	>;
	type MinSwapAmount = ConstU128<{ 10 * UNIT }>; // 最小兑换10 DUST
	// 🆕 存储膨胀防护：TRON 交易哈希 TTL（30天 = 432000 区块 @6秒/块）
	type TxHashTtlBlocks = ConstU32<{ 30 * DAYS }>;
	type WeightInfo = ();
	// 🆕 P3: 仲裁证据 CID 锁定管理器（预留，待 submit_evidence 函数实现后启用）
	type CidLockManager = pallet_storage_service::Pallet<Runtime>;
}

// -------------------- OTC (场外交易) --------------------

/// OTC Maker 接口适配器
pub struct OtcMakerAdapter;

impl pallet_trading_common::MakerInterface<AccountId, Balance> for OtcMakerAdapter {
	fn get_maker_application(maker_id: u64) -> Option<pallet_trading_common::MakerApplicationInfo<AccountId, Balance>> {
		pallet_trading_maker::Pallet::<Runtime>::maker_applications(maker_id).map(|app| {
			pallet_trading_common::MakerApplicationInfo {
				account: app.owner,
				tron_address: app.tron_address,
				is_active: app.status == pallet_trading_maker::pallet::ApplicationStatus::Active,
				_phantom: core::marker::PhantomData,
			}
		})
	}

	fn is_maker_active(maker_id: u64) -> bool {
		pallet_trading_maker::Pallet::<Runtime>::is_maker_active(maker_id)
	}

	fn get_maker_id(who: &AccountId) -> Option<u64> {
		pallet_trading_maker::Pallet::<Runtime>::get_maker_id(who)
	}

	fn get_deposit_usd_value(maker_id: u64) -> Result<u64, sp_runtime::DispatchError> {
		pallet_trading_maker::Pallet::<Runtime>::get_deposit_usd_value(maker_id)
	}
}

/// OTC Maker Credit 接口适配器
pub struct OtcMakerCreditAdapter;

impl pallet_trading_common::MakerCreditInterface for OtcMakerCreditAdapter {
	fn record_maker_order_completed(maker_id: u64, order_id: u64, response_time_seconds: u32) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_order_completed(maker_id, order_id, response_time_seconds)
	}

	fn record_maker_order_timeout(maker_id: u64, order_id: u64) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_order_timeout(maker_id, order_id)
	}

	fn record_maker_dispute_result(maker_id: u64, order_id: u64, maker_win: bool) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_dispute_result(maker_id, order_id, maker_win)
	}
}

/// OTC Identity Provider - 暂时跳过 KYC 验证
pub struct NullIdentityProvider;

impl pallet_trading_otc::pallet::IdentityVerificationProvider<AccountId> for NullIdentityProvider {
	fn get_highest_judgement_priority(_who: &AccountId) -> Option<u8> {
		// 暂时返回 KnownGood 等级，跳过 KYC 验证
		Some(3)
	}

	fn has_problematic_judgement(_who: &AccountId) -> bool {
		false
	}
}

impl pallet_trading_otc::Config for Runtime {
	type Currency = Balances;
	type Timestamp = TimestampProvider;
	type Escrow = pallet_escrow::Pallet<Runtime>;
	type Credit = pallet_trading_credit::Pallet<Runtime>;
	type MakerCredit = OtcMakerCreditAdapter;
	type Pricing = TradingPricingProvider;
	type MakerPallet = OtcMakerAdapter;
	type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
	type IdentityProvider = NullIdentityProvider;
	type ChatPermission = pallet_chat_permission::Pallet<Runtime>;
	type OrderTimeout = ConstU64<3600000>; // 1小时（毫秒）
	type EvidenceWindow = ConstU64<86400000>; // 24小时（毫秒）
	type FirstPurchaseUsdValue = ConstU128<10_000_000>; // 10 USD (精度 10^6)
	type MinFirstPurchaseDustAmount = ConstU128<{ 1 * UNIT }>; // 最小1 DUST (防止汇率过高)
	type MaxFirstPurchaseDustAmount = ConstU128<{ 100_000_000 * UNIT }>; // 最大1亿DUST (防止汇率异常低)
	type MaxOrderUsdAmount = ConstU64<200_000_000>; // 200 USD
	type MinOrderUsdAmount = ConstU64<20_000_000>; // 20 USD
	type FirstPurchaseUsdAmount = ConstU64<10_000_000>; // 10 USD
	type AmountValidationTolerance = ConstU16<100>; // 1% 容差
	type MaxFirstPurchaseOrdersPerMaker = ConstU32<5>;
	// 🆕 2026-01-18: 买家押金机制配置
	type MinDeposit = ConstU128<{ UNIT / 10 }>; // 最小押金 0.1 DUST
	type DepositRateLow = ConstU16<300>; // 3% (信用分 50-69)
	type DepositRateMedium = ConstU16<500>; // 5% (信用分 30-49)
	type DepositRateHigh = ConstU16<1000>; // 10% (信用分 < 30)
	type CreditScoreExempt = ConstU16<70>; // 信用分 >= 70 免押金
	type MinOrdersForExempt = ConstU32<5>; // 至少完成 5 单才能免押金
	type CancelPenaltyRate = ConstU16<3000>; // 取消订单扣除 30% 押金
	type MinMakerDepositUsd = ConstU64<500_000_000>; // 做市商最低押金 500 USDT（精度10^6）
	type DisputeResponseTimeout = ConstU64<86400>; // 24小时（秒）
	type DisputeArbitrationTimeout = ConstU64<172800>; // 48小时（秒）
	type ArbitratorOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
	// 🆕 P3: 争议证据 CID 锁定管理器
	type CidLockManager = pallet_storage_service::Pallet<Runtime>;
}

// ============================================================================
// Escrow, Referral, IPFS Pallets Configuration
// ============================================================================

// -------------------- Escrow (托管) --------------------

parameter_types! {
	pub const EscrowPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/escro");
}

/// 托管过期策略实现
pub struct DefaultExpiryPolicy;

impl pallet_escrow::ExpiryPolicy<AccountId, BlockNumber> for DefaultExpiryPolicy {
	fn on_expire(_id: u64) -> Result<pallet_escrow::ExpiryAction<AccountId>, sp_runtime::DispatchError> {
		// 默认策略：过期后不执行任何操作
		Ok(pallet_escrow::ExpiryAction::Noop)
	}

	fn now() -> BlockNumber {
		System::block_number()
	}
}

impl pallet_escrow::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EscrowPalletId = EscrowPalletId;
	type AuthorizedOrigin = frame_system::EnsureSigned<AccountId>;
	type AdminOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxExpiringPerBlock = ConstU32<100>;
	type ExpiryPolicy = DefaultExpiryPolicy;
}

// -------------------- Referral (推荐关系) --------------------

parameter_types! {
	/// 联盟分成最低 USDT 要求（精度 10^6，30_000_000 = 30 USDT）
	pub const AffiliateMinUsdt: u64 = 30_000_000;
}

/// 基于余额的会员验证 - 账户余额 >= 30 USDT 等值 DUST 才有资格获得联盟分成
/// 使用 pricing 模块的实时 DUST/USDT 价格进行换算
pub struct BalanceBasedMembership;

impl pallet_referral::MembershipProvider<AccountId> for BalanceBasedMembership {
	fn is_valid_member(who: &AccountId) -> bool {
		// 获取账户可用余额
		let balance = pallet_balances::Pallet::<Runtime>::free_balance(who);

		// 获取 DUST/USDT 价格（精度 10^6）
		let price_usdt = pallet_trading_pricing::Pallet::<Runtime>::get_dust_market_price_weighted();

		// 价格为 0 时使用保底逻辑（要求最低 ED）
		if price_usdt == 0 {
			return balance >= EXISTENTIAL_DEPOSIT;
		}

		// 计算 30 USDT 等值的 DUST 数量
		// min_dust = 30_USDT * 10^12 / price_usdt
		// 其中 30_USDT = 30_000_000（精度 10^6）
		let min_usdt = AffiliateMinUsdt::get() as u128;
		let min_dust = min_usdt
			.saturating_mul(1_000_000_000_000u128)  // 10^12 DUST 精度
			.checked_div(price_usdt as u128)
			.unwrap_or(0);

		balance >= min_dust
	}
}

impl pallet_referral::Config for Runtime {
	type MembershipProvider = BalanceBasedMembership;
	type MaxCodeLen = ConstU32<32>;
	type MaxSearchHops = ConstU32<20>;
	type WeightInfo = pallet_referral::weights::SubstrateWeight<Runtime>;
}

// -------------------- Storage Service (存储服务) --------------------

parameter_types! {
	// 3. 存储服务主账户 - 核心账户，含费用收集
	pub const StorageServicePalletId: frame_support::PalletId = frame_support::PalletId(*b"py/storg");
	pub StoragePoolAccountId: AccountId = StorageServicePalletId::get().into_account_truncating();
	
	// 4. 运营商托管账户 - 必须独立
	pub OperatorEscrowAccountId: AccountId = StorageServicePalletId::get().into_sub_account_truncating(b"escrow");
}

impl pallet_storage_service::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Balance = Balance;
	type FeeCollector = StoragePoolAccountId;
	// 内容委员会 1/2 多数通过（P0 治理集成）
	type GovernanceOrigin = pallet_collective::EnsureProportionAtLeast<
		AccountId,
		ContentCollectiveInstance,
		1, 2  // 1/2 多数通过
	>;
	type MaxCidHashLen = ConstU32<64>;
	type MaxPeerIdLen = ConstU32<128>;
	type MinOperatorBond = ConstU128<{ 100 * UNIT }>;
	type MinOperatorBondUsd = ConstU64<100_000_000>; // 100 USDT
	type DepositCalculator = pallet_trading_common::DepositCalculatorImpl<TradingPricingProvider, Balance>;
	type MinCapacityGiB = ConstU32<10>;
	type WeightInfo = ();
	type SubjectPalletId = StorageServicePalletId;
	type IpfsPoolAccount = StoragePoolAccountId;
	type OperatorEscrowAccount = OperatorEscrowAccountId;
	type MonthlyPublicFeeQuota = ConstU128<{ 10 * UNIT }>;
	type QuotaResetPeriod = ConstU32<{ 30 * DAYS }>;
	type DefaultBillingPeriod = ConstU32<{ 30 * DAYS }>;
}

// -------------------- Evidence (证据存证) --------------------

parameter_types! {
	pub const EvidenceNsBytes: [u8; 8] = *b"evidence";
}

/// 证据授权适配器 - 暂时允许所有签名用户
pub struct AlwaysAuthorizedEvidence;

impl pallet_evidence::pallet::EvidenceAuthorizer<AccountId> for AlwaysAuthorizedEvidence {
	fn is_authorized(_ns: [u8; 8], _who: &AccountId) -> bool {
		// 暂时允许所有签名用户提交证据
		// 后续可以对接更细粒度的权限系统
		true
	}
}

impl pallet_evidence::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// Phase 1.5 新参数
	type MaxContentCidLen = ConstU32<64>;
	type MaxSchemeLen = ConstU32<32>;
	// 旧版参数（向后兼容）
	type MaxCidLen = ConstU32<64>;
	type MaxImg = ConstU32<20>;
	type MaxVid = ConstU32<10>;
	type MaxDoc = ConstU32<20>;
	type MaxMemoLen = ConstU32<512>;
	type MaxAuthorizedUsers = ConstU32<50>;
	type MaxKeyLen = ConstU32<512>;
	type EvidenceNsBytes = EvidenceNsBytes;
	type Authorizer = AlwaysAuthorizedEvidence;
	type MaxPerSubjectTarget = ConstU32<1000>;
	type MaxPerSubjectNs = ConstU32<1000>;
	type WindowBlocks = ConstU32<{ 10 * MINUTES }>;
	type MaxPerWindow = ConstU32<100>;
	type EnableGlobalCidDedup = ConstBool<true>;
	type MaxListLen = ConstU32<100>;
	type WeightInfo = pallet_evidence::weights::SubstrateWeight<Runtime>;
	// IPFS 相关
	type IpfsPinner = pallet_storage_service::Pallet<Runtime>;
	type Balance = Balance;
	type DefaultStoragePrice = ConstU128<{ UNIT / 10 }>;
	// 🆕 证据修改窗口（2天 ≈ 28800 blocks，按6秒/块）
	type EvidenceEditWindow = ConstU32<28800>;
}

// -------------------- Arbitration (仲裁) --------------------

/// 统一仲裁域路由器
/// 
/// 将仲裁决议路由到各业务模块执行，支持12个业务域
pub struct UnifiedArbitrationRouter;

impl pallet_arbitration::pallet::ArbitrationRouter<AccountId, Balance> for UnifiedArbitrationRouter {
	/// 校验是否允许发起争议
	fn can_dispute(domain: [u8; 8], who: &AccountId, id: u64) -> bool {
		use pallet_arbitration::pallet::domains;
		
		match domain {
			// 需要验证参与方身份的域
			d if d == domains::OTC_ORDER => {
				pallet_trading_otc::Orders::<Runtime>::get(id)
					.map(|order| order.taker == *who || order.maker == *who)
					.unwrap_or(false)
			},
			d if d == domains::DIVINATION => {
				pallet_divination_market::Orders::<Runtime>::get(id)
					.map(|order| order.customer == *who)
					.unwrap_or(false)
			},
			// 需要验证对象存在的域
			d if d == domains::LIVESTREAM => pallet_livestream::LiveRooms::<Runtime>::get(id).is_some(),
			d if d == domains::MAKER => pallet_trading_maker::MakerApplications::<Runtime>::get(id).is_some(),
			d if d == domains::NFT_TRADE => pallet_divination_nft::Nfts::<Runtime>::get(id).is_some(),
			d if d == domains::SWAP => pallet_trading_swap::MakerSwaps::<Runtime>::get(id).is_some(),
			// 其他域：任何人可以投诉
			_ => true,
		}
	}

	/// 应用裁决（放款/退款/部分放款）
	fn apply_decision(domain: [u8; 8], id: u64, decision: pallet_arbitration::pallet::Decision) -> sp_runtime::DispatchResult {
		use pallet_arbitration::pallet::{Decision, domains};
		
		match domain {
			d if d == domains::OTC_ORDER => {
				// OTC 裁决执行：正确路由到支持 Partial 的函数
				pallet_trading_otc::Pallet::<Runtime>::apply_arbitration_decision(id, decision)
			},
			d if d == domains::DIVINATION => {
				// 占卜服务裁决执行：扣除提供者保证金并退款
				match decision {
					Decision::Refund => {
						// 客户胜诉：扣除提供者30%保证金赔付客户，全额退款
						let _ = pallet_divination_market::Pallet::<Runtime>::slash_provider_deposit(id, 3000, true);
						pallet_divination_market::Pallet::<Runtime>::refund_customer_on_complaint(id, 10000)
					},
					Decision::Release => {
						// 提供者胜诉：不扣除保证金，不退款
						Ok(())
					},
					Decision::Partial(bps) => {
						// 部分胜诉：按比例扣除保证金和退款
						let slash_bps = (bps / 2) as u16; // 扣除比例减半
						let _ = pallet_divination_market::Pallet::<Runtime>::slash_provider_deposit(id, slash_bps, true);
						pallet_divination_market::Pallet::<Runtime>::refund_customer_on_complaint(id, bps)
					},
				}
			},
			d if d == domains::CHAT_GROUP => {
				// 群组投诉裁决执行
				// TODO: 群组保证金扣除功能待实现 (slash_group_bond)
				// 当前直接返回 Ok，仲裁模块已处理押金分配
				match decision {
					Decision::Refund => Ok(()), // 投诉方胜诉
					Decision::Release => Ok(()), // 群主胜诉
					Decision::Partial(_) => Ok(()), // 部分胜诉
				}
			},
			d if d == domains::LIVESTREAM => {
				// 直播间投诉裁决执行：扣除主播保证金
				match decision {
					Decision::Refund => {
						// 投诉方胜诉：扣除主播30%保证金赔付投诉方
						let _ = pallet_livestream::Pallet::<Runtime>::slash_room_bond(id, 3000, None);
						Ok(())
					},
					Decision::Release => Ok(()), // 主播胜诉：不扣除
					Decision::Partial(bps) => {
						let slash_bps = (bps / 2) as u16;
						let _ = pallet_livestream::Pallet::<Runtime>::slash_room_bond(id, slash_bps, None);
						Ok(())
					},
				}
			},
			// 其他域暂时无需额外操作，仲裁模块已处理押金分配
			_ => Ok(())
		}
	}

	/// 获取纠纷对方账户
	fn get_counterparty(domain: [u8; 8], initiator: &AccountId, id: u64) -> Result<AccountId, sp_runtime::DispatchError> {
		use pallet_arbitration::pallet::domains;
		use sp_runtime::DispatchError;
		
		match domain {
			d if d == domains::OTC_ORDER => {
				let order = pallet_trading_otc::Orders::<Runtime>::get(id)
					.ok_or(DispatchError::Other("OrderNotFound"))?;
				if order.taker == *initiator {
					Ok(order.maker)
				} else {
					Ok(order.taker)
				}
			},
			d if d == domains::LIVESTREAM => {
				let room = pallet_livestream::LiveRooms::<Runtime>::get(id)
					.ok_or(DispatchError::Other("RoomNotFound"))?;
				Ok(room.host)
			},
			d if d == domains::DIVINATION => {
				let order = pallet_divination_market::Orders::<Runtime>::get(id)
					.ok_or(DispatchError::Other("OrderNotFound"))?;
				if order.customer == *initiator {
					Ok(order.provider)
				} else {
					Ok(order.customer)
				}
			},
			d if d == domains::CHAT_GROUP => {
				let group = pallet_chat_group::Groups::<Runtime>::get(id)
					.ok_or(DispatchError::Other("GroupNotFound"))?;
				Ok(group.owner)
			},
			d if d == domains::MAKER => {
				let maker_app = pallet_trading_maker::MakerApplications::<Runtime>::get(id)
					.ok_or(DispatchError::Other("MakerNotFound"))?;
				Ok(maker_app.owner)
			},
			_ => {
				// 对于其他域，返回平台账户（PalletId 派生）
				Ok(TreasuryAccountId::get())
			}
		}
	}

	/// 获取订单/交易金额（用于计算押金）
	fn get_order_amount(domain: [u8; 8], id: u64) -> Result<Balance, sp_runtime::DispatchError> {
		use pallet_arbitration::pallet::domains;
		use sp_runtime::DispatchError;
		
		match domain {
			d if d == domains::OTC_ORDER => {
				let order = pallet_trading_otc::Orders::<Runtime>::get(id)
					.ok_or(DispatchError::Other("OrderNotFound"))?;
				Ok(order.amount)
			},
			d if d == domains::DIVINATION => {
				let order = pallet_divination_market::Orders::<Runtime>::get(id)
					.ok_or(DispatchError::Other("OrderNotFound"))?;
				Ok(order.amount)
			},
			d if d == domains::LIVESTREAM => {
				// 直播间投诉：使用固定金额 10 UNIT
				Ok(10 * UNIT)
			},
			d if d == domains::CHAT_GROUP => {
				// 群组投诉：使用固定金额 5 UNIT
				Ok(5 * UNIT)
			},
			_ => {
				// 默认固定金额 10 UNIT
				Ok(10 * UNIT)
			}
		}
	}

	/// 获取做市商ID（用于信用分更新）
	fn get_maker_id(domain: [u8; 8], id: u64) -> Option<u64> {
		use pallet_arbitration::pallet::domains;
		
		match domain {
			d if d == domains::OTC_ORDER => {
				// OTC 订单：从订单获取 maker_id
				pallet_trading_otc::Orders::<Runtime>::get(id)
					.map(|order| order.maker_id)
			},
			d if d == domains::MAKER => {
				// 做市商域：id 本身就是 maker_id
				Some(id)
			},
			_ => None,
		}
	}
}

/// 信用分更新器实现
pub struct TradingCreditUpdater;

impl pallet_arbitration::pallet::CreditUpdater for TradingCreditUpdater {
	fn record_maker_dispute_result(maker_id: u64, order_id: u64, maker_win: bool) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_dispute_result(maker_id, order_id, maker_win)
	}
}

impl pallet_arbitration::pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxEvidence = ConstU32<20>;
	type MaxCidLen = ConstU32<64>;
	type Escrow = pallet_escrow::Pallet<Runtime>;
	type WeightInfo = pallet_arbitration::weights::SubstrateWeight<Runtime>;
	type Router = UnifiedArbitrationRouter;
	type DecisionOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, ArbitrationCollectiveInstance, 2, 3>;
	type Fungible = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type DepositRatioBps = ConstU16<1500>; // 15% 押金比例
	type ResponseDeadline = ConstU32<{ 7 * DAYS }>; // 7天应诉期限
	type RejectedSlashBps = ConstU16<3000>; // 驳回时罚没30%
	type PartialSlashBps = ConstU16<5000>; // 部分胜诉罚没50%
	type ComplaintDeposit = ConstU128<{ UNIT / 10 }>; // 投诉押金兜底值 0.1 DUST
	type ComplaintDepositUsd = ConstU64<1_000_000>; // 投诉押金 1 USDT（精度10^6，使用pricing换算）
	type Pricing = TradingPricingProvider; // 定价接口
	type ComplaintSlashBps = ConstU16<5000>; // 投诉败诉罚没50%
	type TreasuryAccount = TreasuryAccountId;
	// 🆕 P2: CID 锁定管理器
	type CidLockManager = pallet_storage_service::Pallet<Runtime>;
	// 🆕 信用分更新器
	type CreditUpdater = TradingCreditUpdater;
}

// ============================================================================
// Governance: Collective (Committees) Configuration
// ============================================================================

// -------------------- 1. 技术委员会 (Technical Committee) --------------------
// 职责：紧急升级、runtime 参数调整、技术提案审核

pub type TechnicalCollectiveInstance = pallet_collective::Instance1;

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 7 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 11;
	pub MaxTechnicalProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<TechnicalCollectiveInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxTechnicalProposalWeight;
	type DisapproveOrigin = frame_system::EnsureRoot<AccountId>;
	type KillOrigin = frame_system::EnsureRoot<AccountId>;
	type Consideration = ();
}

// -------------------- 2. 仲裁委员会 (Arbitration Committee) --------------------
// 职责：处理 OTC/Bridge/供奉订单的争议裁决

pub type ArbitrationCollectiveInstance = pallet_collective::Instance2;

parameter_types! {
	pub const ArbitrationMotionDuration: BlockNumber = 3 * DAYS;
	pub const ArbitrationMaxProposals: u32 = 200;
	pub const ArbitrationMaxMembers: u32 = 15;
	pub MaxArbitrationProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<ArbitrationCollectiveInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = ArbitrationMotionDuration;
	type MaxProposals = ArbitrationMaxProposals;
	type MaxMembers = ArbitrationMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxArbitrationProposalWeight;
	type DisapproveOrigin = frame_system::EnsureRoot<AccountId>;
	type KillOrigin = frame_system::EnsureRoot<AccountId>;
	type Consideration = ();
}

// -------------------- 3. 财务委员会 (Treasury Council) --------------------
// 职责：审批国库支出、资金分配、生态激励

pub type TreasuryCollectiveInstance = pallet_collective::Instance3;

parameter_types! {
	pub const TreasuryMotionDuration: BlockNumber = 5 * DAYS;
	pub const TreasuryMaxProposals: u32 = 50;
	pub const TreasuryMaxMembers: u32 = 9;
	pub MaxTreasuryProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<TreasuryCollectiveInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TreasuryMotionDuration;
	type MaxProposals = TreasuryMaxProposals;
	type MaxMembers = TreasuryMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxTreasuryProposalWeight;
	type DisapproveOrigin = frame_system::EnsureRoot<AccountId>;
	type KillOrigin = frame_system::EnsureRoot<AccountId>;
	type Consideration = ();
}

// -------------------- 4. 内容委员会 (Content Committee) --------------------
// 职责：审核占卜师资质、直播内容合规、证据真实性

pub type ContentCollectiveInstance = pallet_collective::Instance4;

parameter_types! {
	pub const ContentMotionDuration: BlockNumber = 2 * DAYS;
	pub const ContentMaxProposals: u32 = 100;
	pub const ContentMaxMembers: u32 = 7;
	pub MaxContentProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<ContentCollectiveInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = ContentMotionDuration;
	type MaxProposals = ContentMaxProposals;
	type MaxMembers = ContentMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxContentProposalWeight;
	type DisapproveOrigin = frame_system::EnsureRoot<AccountId>;
	type KillOrigin = frame_system::EnsureRoot<AccountId>;
	type Consideration = ();
}

// -------------------- Membership Pallets for Committees --------------------

// 技术委员会成员管理
pub type TechnicalMembershipInstance = pallet_collective_membership::Instance1;

impl pallet_collective_membership::Config<TechnicalMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = frame_system::EnsureRoot<AccountId>;
	type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
	type SwapOrigin = frame_system::EnsureRoot<AccountId>;
	type ResetOrigin = frame_system::EnsureRoot<AccountId>;
	type PrimeOrigin = frame_system::EnsureRoot<AccountId>;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = pallet_collective_membership::weights::SubstrateWeight<Runtime>;
}

// 仲裁委员会成员管理
pub type ArbitrationMembershipInstance = pallet_collective_membership::Instance2;

impl pallet_collective_membership::Config<ArbitrationMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = frame_system::EnsureRoot<AccountId>;
	type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
	type SwapOrigin = frame_system::EnsureRoot<AccountId>;
	type ResetOrigin = frame_system::EnsureRoot<AccountId>;
	type PrimeOrigin = frame_system::EnsureRoot<AccountId>;
	type MembershipInitialized = ArbitrationCommittee;
	type MembershipChanged = ArbitrationCommittee;
	type MaxMembers = ArbitrationMaxMembers;
	type WeightInfo = pallet_collective_membership::weights::SubstrateWeight<Runtime>;
}

// 财务委员会成员管理
pub type TreasuryMembershipInstance = pallet_collective_membership::Instance3;

impl pallet_collective_membership::Config<TreasuryMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = frame_system::EnsureRoot<AccountId>;
	type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
	type SwapOrigin = frame_system::EnsureRoot<AccountId>;
	type ResetOrigin = frame_system::EnsureRoot<AccountId>;
	type PrimeOrigin = frame_system::EnsureRoot<AccountId>;
	type MembershipInitialized = TreasuryCouncil;
	type MembershipChanged = TreasuryCouncil;
	type MaxMembers = TreasuryMaxMembers;
	type WeightInfo = pallet_collective_membership::weights::SubstrateWeight<Runtime>;
}

// 内容委员会成员管理
pub type ContentMembershipInstance = pallet_collective_membership::Instance4;

impl pallet_collective_membership::Config<ContentMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = frame_system::EnsureRoot<AccountId>;
	type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
	type SwapOrigin = frame_system::EnsureRoot<AccountId>;
	type ResetOrigin = frame_system::EnsureRoot<AccountId>;
	type PrimeOrigin = frame_system::EnsureRoot<AccountId>;
	type MembershipInitialized = ContentCommittee;
	type MembershipChanged = ContentCommittee;
	type MaxMembers = ContentMaxMembers;
	type WeightInfo = pallet_collective_membership::weights::SubstrateWeight<Runtime>;
}

// ============================================================================
// Divination Membership Pallet Configuration
// ============================================================================

parameter_types! {
	pub const DivinationMembershipPalletId: frame_support::PalletId = frame_support::PalletId(*b"div/memb");
	pub const RewardPoolAllocation: u32 = 1000; // 10% 分配到奖励池
	pub const NewAccountCooldown: BlockNumber = 7 * DAYS; // 7天冷却期
	pub const MinBalanceForRewards: Balance = UNIT; // 最低 1 DUST
	pub const BlocksPerDay: BlockNumber = DAYS; // 每天区块数
	pub const BlocksPerMonth: BlockNumber = 30 * DAYS; // 每月区块数
	pub const MaxDisplayNameLength: u32 = 64;
	pub const MaxEncryptedDataLength: u32 = 1024;
	pub const MaxRewardHistorySize: u32 = 50;
}

impl pallet_divination_membership::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
	type PalletId = DivinationMembershipPalletId;
	type TreasuryAccount = TreasuryAccountId;
	type BurnAccount = BurnAccountId;
	type UserFundingProvider = StorageUserFundingProvider;
	type AffiliateDistributor = StubAffiliateDistributor;
	type RewardPoolAllocation = RewardPoolAllocation;
	type NewAccountCooldown = NewAccountCooldown;
	type MinBalanceForRewards = MinBalanceForRewards;
	type BlocksPerDay = BlocksPerDay;
	type BlocksPerMonth = BlocksPerMonth;
	type MaxDisplayNameLength = MaxDisplayNameLength;
	type MaxEncryptedDataLength = MaxEncryptedDataLength;
	type MaxRewardHistorySize = MaxRewardHistorySize;
	type Pricing = TradingPricingProvider;
}

// ============================================================================
// Matchmaking Membership Pallet Configuration
// ============================================================================

parameter_types! {
	pub const MatchmakingBlocksPerMonth: BlockNumber = 30 * DAYS;
	pub const MatchmakingBlocksPerDay: BlockNumber = DAYS;
	pub const MatchmakingMonthlyFee: Balance = 10 * UNIT; // 兜底值 10 DUST
	pub const MatchmakingMonthlyFeeUsd: u64 = 10_000_000; // 10 USDT
	pub const MatchmakingLifetimeFee: Balance = 500 * UNIT; // 兜底值 500 DUST
	pub const MatchmakingLifetimeFeeUsd: u64 = 500_000_000; // 500 USDT
	// Profile 保证金配置
	pub const ProfileDeposit: Balance = 500 * UNIT; // 兜底值 500 DUST
	pub const ProfileDepositUsd: u64 = 50_000_000; // 50 USDT
	pub const ProfileMonthlyFee: Balance = 20 * UNIT; // 兜底值 20 DUST
	pub const ProfileMonthlyFeeUsd: u64 = 2_000_000; // 2 USDT
}

impl pallet_matchmaking_membership::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Fungible = Balances;
	type Balance = Balance;
	type BlocksPerMonth = MatchmakingBlocksPerMonth;
	type BlocksPerDay = MatchmakingBlocksPerDay;
	type MonthlyFee = MatchmakingMonthlyFee;
	type MonthlyFeeUsd = MatchmakingMonthlyFeeUsd;
	type LifetimeFee = MatchmakingLifetimeFee;
	type LifetimeFeeUsd = MatchmakingLifetimeFeeUsd;
	type Pricing = TradingPricingProvider;
	type TreasuryAccount = TreasuryAccountId;
	type BurnAccount = BurnAccountId;
	type UserFundingProvider = StorageUserFundingProvider;
	type AffiliateDistributor = StubAffiliateDistributor;
}

// ============================================================================
// Matchmaking Profile Pallet Configuration
// ============================================================================

impl pallet_matchmaking_profile::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxNicknameLen = ConstU32<64>;
	type MaxLocationLen = ConstU32<128>;
	type MaxCidLen = ConstU32<64>;
	type MaxBioLen = ConstU32<512>;
	type MaxDescLen = ConstU32<256>;
	type MaxOccupationLen = ConstU32<64>;
	type MaxTraits = ConstU32<10>;
	type MaxHobbies = ConstU32<20>;
	type MaxHobbyLen = ConstU32<32>;
	type WeightInfo = ();
	type Fungible = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type ProfileDeposit = ProfileDeposit;
	type ProfileDepositUsd = ProfileDepositUsd;
	type MonthlyFee = ProfileMonthlyFee;
	type MonthlyFeeUsd = ProfileMonthlyFeeUsd;
	type Pricing = TradingPricingProvider;
	type TreasuryAccount = TreasuryAccountId;
	type BurnAccount = BurnAccountId;
	type StorageAccount = StoragePoolAccountId;
	type AffiliateDistributor = StubAffiliateDistributor;
	type IpfsPinner = pallet_storage_service::Pallet<Runtime>;
	type GovernanceOrigin = EnsureRoot<AccountId>;
	type BlocksPerDay = MatchmakingBlocksPerDay;
	type Balance = Balance;
}

// ============================================================================
// Storage Lifecycle Pallet Configuration
// ============================================================================

impl pallet_storage_lifecycle::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type L1ArchiveDelay = ConstU32<{ 30 * DAYS }>;  // 30天后归档到L1
	type L2ArchiveDelay = ConstU32<{ 90 * DAYS }>;  // L1后90天归档到L2
	type PurgeDelay = ConstU32<{ 180 * DAYS }>;     // L2后180天可清除
	type EnablePurge = ConstBool<false>;             // 默认不启用清除
	type MaxBatchSize = ConstU32<100>;               // 每次最多处理100条
}
