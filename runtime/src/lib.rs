#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod configs;

extern crate alloc;
use alloc::vec::Vec;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiAddress, MultiSignature,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub mod genesis_config_presets;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
		pub grandpa: Grandpa,
	}
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: alloc::borrow::Cow::Borrowed("stardust"),
	impl_name: alloc::borrow::Cow::Borrowed("stardust"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 100,
	impl_version: 1,
	apis: apis::RUNTIME_API_VERSIONS,
	transaction_version: 1,
	system_version: 1,
};

mod block_times {
	/// This determines the average expected block time that we are targeting. Blocks will be
	/// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
	/// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
	/// slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLI_SECS_PER_BLOCK: u64 = 6000;

	// NOTE: Currently it is not possible to change the slot duration after the chain has started.
	// Attempting to do so will brick block production.
	pub const SLOT_DURATION: u64 = MILLI_SECS_PER_BLOCK;
}
pub use block_times::*;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLI_SECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const BLOCK_HASH_COUNT: BlockNumber = 2400;

// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLI_UNIT: Balance = 1_000_000_000;
pub const MICRO_UNIT: Balance = 1_000_000;

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLI_UNIT;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The `TransactionExtension` to the basic transaction logic.
pub type TxExtension = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
	frame_system::WeightReclaim<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, TxExtension>;

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
pub type Migrations = ();

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

// Create the runtime by composing the FRAME pallets that were previously configured.
#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Runtime;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;

	#[runtime::pallet_index(1)]
	pub type Timestamp = pallet_timestamp;

	#[runtime::pallet_index(2)]
	pub type Aura = pallet_aura;

	#[runtime::pallet_index(3)]
	pub type Grandpa = pallet_grandpa;

	#[runtime::pallet_index(4)]
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(5)]
	pub type TransactionPayment = pallet_transaction_payment;

	#[runtime::pallet_index(6)]
	pub type Sudo = pallet_sudo;

	// ============================================================================
	// Governance: Committees (Collective + Membership)
	// ============================================================================

	// 1. 技术委员会 (Technical Committee)
	#[runtime::pallet_index(70)]
	pub type TechnicalCommittee = pallet_collective<Instance1>;

	#[runtime::pallet_index(71)]
	pub type TechnicalMembership = pallet_collective_membership<Instance1>;

	// 2. 仲裁委员会 (Arbitration Committee)
	#[runtime::pallet_index(72)]
	pub type ArbitrationCommittee = pallet_collective<Instance2>;

	#[runtime::pallet_index(73)]
	pub type ArbitrationMembership = pallet_collective_membership<Instance2>;

	// 3. 财务委员会 (Treasury Council)
	#[runtime::pallet_index(74)]
	pub type TreasuryCouncil = pallet_collective<Instance3>;

	#[runtime::pallet_index(75)]
	pub type TreasuryMembership = pallet_collective_membership<Instance3>;

	// 4. 内容委员会 (Content Committee)
	#[runtime::pallet_index(76)]
	pub type ContentCommittee = pallet_collective<Instance4>;

	#[runtime::pallet_index(77)]
	pub type ContentMembership = pallet_collective_membership<Instance4>;

	// ============================================================================
	// Divination Pallets
	// ============================================================================

	// 基础模块
	#[runtime::pallet_index(10)]
	pub type Almanac = pallet_almanac;

	#[runtime::pallet_index(11)]
	pub type Privacy = pallet_divination_privacy;

	#[runtime::pallet_index(15)]
	pub type TeePrivacy = pallet_tee_privacy;

	// 服务模块
	#[runtime::pallet_index(12)]
	pub type DivinationAi = pallet_divination_ai;

	#[runtime::pallet_index(13)]
	pub type DivinationMarket = pallet_divination_market;

	#[runtime::pallet_index(14)]
	pub type DivinationNft = pallet_divination_nft;

	// 占卜模块 - 中华术数
	#[runtime::pallet_index(20)]
	pub type Meihua = pallet_meihua;

	#[runtime::pallet_index(21)]
	pub type Bazi = pallet_bazi_chart;

	#[runtime::pallet_index(22)]
	pub type Liuyao = pallet_liuyao;

	#[runtime::pallet_index(23)]
	pub type Qimen = pallet_qimen;

	#[runtime::pallet_index(24)]
	pub type Ziwei = pallet_ziwei;

	#[runtime::pallet_index(25)]
	pub type Xiaoliuren = pallet_xiaoliuren;

	#[runtime::pallet_index(26)]
	pub type Daliuren = pallet_daliuren;

	// 占卜模块 - 西方占卜
	#[runtime::pallet_index(30)]
	pub type Tarot = pallet_tarot;

	// 占卜会员模块
	#[runtime::pallet_index(31)]
	pub type DivinationMembership = pallet_divination_membership;

	// ============================================================================
	// Chat Pallets
	// ============================================================================

	#[runtime::pallet_index(40)]
	pub type ChatPermission = pallet_chat_permission;

	#[runtime::pallet_index(41)]
	pub type ChatCore = pallet_chat_core;

	#[runtime::pallet_index(42)]
	pub type ChatGroup = pallet_chat_group;

	#[runtime::pallet_index(43)]
	pub type Livestream = pallet_livestream;

	// ============================================================================
	// Trading Pallets
	// ============================================================================

	#[runtime::pallet_index(50)]
	pub type TradingPricing = pallet_trading_pricing;

	#[runtime::pallet_index(51)]
	pub type TradingCredit = pallet_trading_credit;

	#[runtime::pallet_index(52)]
	pub type TradingMaker = pallet_trading_maker;

	#[runtime::pallet_index(53)]
	pub type TradingSwap = pallet_trading_swap;

	#[runtime::pallet_index(54)]
	pub type TradingOtc = pallet_trading_otc;

	// ============================================================================
	// Escrow, Referral, IPFS Pallets
	// ============================================================================

	#[runtime::pallet_index(60)]
	pub type Escrow = pallet_escrow;

	#[runtime::pallet_index(61)]
	pub type AffiliateReferral = pallet_referral;

	#[runtime::pallet_index(62)]
	pub type StorageService = pallet_storage_service;

	#[runtime::pallet_index(63)]
	pub type Evidence = pallet_evidence;

	#[runtime::pallet_index(64)]
	pub type Arbitration = pallet_arbitration;

	#[runtime::pallet_index(65)]
	pub type StorageLifecycle = pallet_storage_lifecycle;

	// ============================================================================
	// Matchmaking Pallets
	// ============================================================================

	#[runtime::pallet_index(80)]
	pub type MatchmakingMembership = pallet_matchmaking_membership;

	#[runtime::pallet_index(81)]
	pub type MatchmakingProfile = pallet_matchmaking_profile;
}
