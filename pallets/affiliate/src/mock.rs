//! # Mock Runtime for Affiliate Pallet Testing
//!
//! 函数级详细中文注释：提供 Affiliate Pallet 的测试运行时环境

use crate as pallet_affiliate;
use frame_support::{
    parameter_types,
    traits::ConstU32,
    PalletId,
};
use sp_runtime::{
    BuildStorage,
    traits::{BlakeTwo256, IdentityLookup},
};

type Block = frame_system::mocking::MockBlock<Test>;

// 函数级中文注释：构建测试运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Referral: pallet_affiliate_referral,
        Affiliate: pallet_affiliate,
    }
);

// ========================================
// System 配置
// ========================================

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = ();
}

// ========================================
// Timestamp 配置
// ========================================

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// ========================================
// Balances 配置
// ========================================

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

// ========================================
// Affiliate 配置参数
// ========================================

parameter_types! {
    /// 函数级中文注释：联盟托管 Pallet ID
    pub const AffiliatePalletId: PalletId = PalletId(*b"py/affil");
    
    /// 函数级中文注释：推荐码最大长度
    pub const MaxCodeLen: u32 = 32;
    
    /// 函数级中文注释：推荐链最大搜索深度
    pub const MaxSearchHops: u32 = 20;
    
    /// 函数级中文注释：国库账户（测试账户：999）
    pub const TreasuryAccount: u64 = 999;
    
    /// 函数级中文注释：销毁账户（测试账户：998）
    pub const BurnAccount: u64 = 998;
    
    /// 函数级中文注释：存储费用账户（测试账户：997）
    pub const StorageAccount: u64 = 997;
    
    /// 最大活跃提案数
    pub const MaxActiveProposals: u32 = 10;
    
    /// 最大待执行提案数
    pub const MaxReadyProposals: u32 = 5;
    
    /// 历史记录保留周数
    pub const HistoryRetentionWeeks: u32 = 12;
    
    /// 提案过期区块数（7天）
    pub const ProposalExpiry: u64 = 100800;
}

// ========================================
// Referral Pallet 配置
// ========================================

impl pallet_affiliate_referral::Config for Test {
    type MembershipProvider = MockMembershipProvider;
    type MaxCodeLen = MaxCodeLen;
    type MaxSearchHops = MaxSearchHops;
    type WeightInfo = ();
}

// ========================================
// Mock MembershipProvider
// ========================================

/// 函数级中文注释：模拟会员信息提供者
///
/// 测试环境简化规则：
/// - 账户 ID > 0 且 <= 900 的为有效会员
/// - 账户 ID > 900 的为无效会员（用于测试失败场景）
pub struct MockMembershipProvider;

impl pallet_affiliate::MembershipProvider<u64> for MockMembershipProvider {
    /// 函数级中文注释：检查账户是否为有效会员
    fn is_valid_member(who: &u64) -> bool {
        *who > 0 && *who <= 900
    }
}

// ========================================
// Affiliate Pallet 配置
// ========================================

/// Mock UserFundingProvider 实现
pub struct MockUserFundingProvider;

impl pallet_affiliate::UserFundingProvider<u64> for MockUserFundingProvider {
    fn derive_user_funding_account(user: &u64) -> u64 {
        *user + 1000 // 简单的派生逻辑
    }
}

impl pallet_affiliate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EscrowPalletId = AffiliatePalletId;
    type WithdrawOrigin = frame_system::EnsureRoot<u64>;
    type AdminOrigin = frame_system::EnsureRoot<u64>;
    type BurnAccount = BurnAccount;
    type TreasuryAccount = TreasuryAccount;
    type UserFundingProvider = MockUserFundingProvider;
    type MaxActiveProposals = MaxActiveProposals;
    type MaxReadyProposals = MaxReadyProposals;
    type HistoryRetentionWeeks = HistoryRetentionWeeks;
    type ProposalExpiry = ProposalExpiry;
    type ProposalDeposit = frame_support::traits::ConstU128<50_000_000_000_000_000_000>; // 50 DUST
    type ProposalDepositUsd = frame_support::traits::ConstU64<50_000_000>; // 50 USDT
    type DepositCalculator = (); // 使用空实现，返回兜底值
    type WeightInfo = ();
}

// ========================================
// 测试辅助函数
// ========================================

/// 函数级中文注释：创建测试环境
///
/// 初始化测试环境，并为测试账户分配初始余额。
///
/// **测试账户**：
/// - Alice (1): 10,000 DUST
/// - Bob (2): 10,000 DUST
/// - Charlie (3): 10,000 DUST
/// - Dave (4): 10,000 DUST
/// - Eve (5): 10,000 DUST
/// - Frank (6): 10,000 DUST
/// - Grace (7): 10,000 DUST
/// - Heidi (8): 10,000 DUST
/// - Ivan (9): 10,000 DUST
/// - Judy (10): 10,000 DUST
/// - Treasury (999): 1,000 DUST
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10_000_000_000_000_000),   // Alice: 10,000 DUST
            (2, 10_000_000_000_000_000),   // Bob: 10,000 DUST
            (3, 10_000_000_000_000_000),   // Charlie: 10,000 DUST
            (4, 10_000_000_000_000_000),   // Dave: 10,000 DUST
            (5, 10_000_000_000_000_000),   // Eve: 10,000 DUST
            (6, 10_000_000_000_000_000),   // Frank: 10,000 DUST
            (7, 10_000_000_000_000_000),   // Grace: 10,000 DUST
            (8, 10_000_000_000_000_000),   // Heidi: 10,000 DUST
            (9, 10_000_000_000_000_000),   // Ivan: 10,000 DUST
            (10, 10_000_000_000_000_000),  // Judy: 10,000 DUST
            (999, 1_000_000_000_000_000),  // Treasury: 1,000 DUST
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1000);
    });
    ext
}

/// 函数级中文注释：前进到指定区块
///
/// **参数**：
/// - `n`: 目标区块号
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        Timestamp::set_timestamp((System::block_number() * 6000) as u64);
    }
}

/// 函数级中文注释：获取账户余额
pub fn balance_of(who: u64) -> u128 {
    Balances::free_balance(who)
}

/// 函数级中文注释：获取托管账户余额
#[allow(dead_code)]
pub fn escrow_balance() -> u128 {
    Affiliate::escrow_balance()
}
