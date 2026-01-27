//! 塔罗牌 Pallet 测试 Mock 运行时
//!
//! 本模块提供用于单元测试的模拟运行时环境

use crate as pallet_tarot;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::ConstU32,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

// 配置 Mock 运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
        Tarot: pallet_tarot,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<0>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Test {}

parameter_types! {
    /// 每次占卜最大牌数（对应最复杂的牌阵：年度运势12张）
    pub const MaxCardsPerReading: u32 = 12;
    /// 每个用户最多存储100条占卜记录
    pub const MaxUserReadings: u32 = 100;
    /// 公开占卜列表最多1000条
    pub const MaxPublicReadings: u32 = 1000;
    /// 每日免费占卜3次
    pub const DailyFreeDivinations: u32 = 3;
    /// 每日最大占卜10次
    pub const MaxDailyDivinations: u32 = 10;
    /// AI 解读费用：10 DUST
    pub const AiInterpretationFee: Balance = 10_000_000_000_000; // 10 DUST (12 decimals)
    /// 国库账户
    pub const TreasuryAccountId: u64 = 999;
}

/// AI 预言机权限来源（测试用：允许 root）
pub struct MockAiOracleOrigin;
impl frame_support::traits::EnsureOrigin<RuntimeOrigin> for MockAiOracleOrigin {
    type Success = ();
    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        match o.clone().into() {
            Ok(frame_system::RawOrigin::Root) => Ok(()),
            _ => Err(o),
        }
    }
    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::root())
    }
}

/// 获取国库账户 ID
pub struct TreasuryAccount;
impl frame_support::traits::Get<u64> for TreasuryAccount {
    fn get() -> u64 {
        TreasuryAccountId::get()
    }
}

impl pallet_tarot::Config for Test {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type MaxCardsPerReading = MaxCardsPerReading;
    type MaxUserReadings = MaxUserReadings;
    type MaxPublicReadings = MaxPublicReadings;
    type DailyFreeDivinations = DailyFreeDivinations;
    type MaxDailyDivinations = MaxDailyDivinations;
    type AiInterpretationFee = AiInterpretationFee;
    type TreasuryAccount = TreasuryAccount;
    type AiOracleOrigin = MockAiOracleOrigin;
}

/// 测试账户定义
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const TREASURY: u64 = 999;

/// 初始余额
pub const INITIAL_BALANCE: Balance = 100_000_000_000_000_000; // 100,000 DUST

/// 构建测试环境
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, INITIAL_BALANCE),
            (BOB, INITIAL_BALANCE),
            (CHARLIE, INITIAL_BALANCE),
            (TREASURY, INITIAL_BALANCE),
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        // 设置初始时间戳（2024年1月1日 00:00:00 UTC）
        Timestamp::set_timestamp(1704067200000);
    });
    ext
}

/// 推进区块
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        // 每个区块时间增加6秒（6000毫秒）
        let current_time = Timestamp::get();
        Timestamp::set_timestamp(current_time + 6000);
    }
}
