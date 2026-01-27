//! 梅花易数 Pallet 测试模拟环境
//!
//! 提供测试所需的运行时配置和工具函数

use crate as pallet_meihua;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// 构建模拟运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Privacy: pallet_divination_privacy,
        Meihua: pallet_meihua,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u16 = 42;
}

// 系统配置
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
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

// 余额配置
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
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

// 时间戳配置
parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// 隐私授权模块配置
parameter_types! {
    pub const MaxEncryptedDataLen: u32 = 512;
    pub const MaxEncryptedKeyLen: u32 = 128;
    pub const MaxGranteesPerRecord: u32 = 20;
    pub const MaxRecordsPerUser: u32 = 1000;
    pub const MaxProvidersPerType: u32 = 1000;
    pub const MaxGrantsPerProvider: u32 = 500;
    pub const MaxAuthorizationsPerBounty: u32 = 100;
}

impl pallet_divination_privacy::Config for Test {
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

// 梅花易数配置
parameter_types! {
    pub const MaxUserHexagrams: u32 = 1000;
    pub const MaxPublicHexagrams: u32 = 10000;
    pub const DailyFreeDivinations: u32 = 3;
    pub const MaxDailyDivinations: u32 = 50;
    pub const AiInterpretationFee: u64 = 100;
    pub TreasuryAccount: u64 = 999;
}

/// 模拟随机数生成器
pub struct MockRandomness;

impl frame_support::traits::Randomness<H256, u64> for MockRandomness {
    fn random(subject: &[u8]) -> (H256, u64) {
        let mut data = [0u8; 32];
        data[..subject.len().min(32)].copy_from_slice(&subject[..subject.len().min(32)]);
        // 简单的伪随机：使用输入的哈希
        (H256::from_slice(&data), 0)
    }
}

/// 模拟 AI 预言机权限
pub struct MockAiOracleOrigin;

impl<O: Into<Result<frame_system::RawOrigin<u64>, O>> + From<frame_system::RawOrigin<u64>>>
    frame_support::traits::EnsureOrigin<O> for MockAiOracleOrigin
{
    type Success = ();

    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            // 允许 root 或账户 1 作为 AI 预言机
            frame_system::RawOrigin::Root => Ok(()),
            frame_system::RawOrigin::Signed(1) => Ok(()),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(frame_system::RawOrigin::Root))
    }
}

impl pallet_meihua::Config for Test {
    type Currency = Balances;
    type Randomness = MockRandomness;
    type MaxUserHexagrams = MaxUserHexagrams;
    type MaxPublicHexagrams = MaxPublicHexagrams;
    type DailyFreeDivinations = DailyFreeDivinations;
    type MaxDailyDivinations = MaxDailyDivinations;
    type AiInterpretationFee = AiInterpretationFee;
    type TreasuryAccount = TreasuryAccount;
    type AiOracleOrigin = MockAiOracleOrigin;
}

/// 构建测试外部状态
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    // 初始化账户余额
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000),
            (2, 1_000_000),
            (3, 1_000_000),
            (999, 100), // 国库 - 需要大于 existential deposit
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        // 设置一个有效的时间戳：2024-11-29 12:00:00 UTC
        // 这个时间戳对应的农历日期在 1900-2100 范围内
        let _ = Timestamp::set(RuntimeOrigin::none(), 1732881600000);
    });
    ext
}

/// 推进区块
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}
