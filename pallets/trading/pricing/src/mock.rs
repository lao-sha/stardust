// 函数级中文注释：pallet-pricing的Mock Runtime，用于单元测试
// Phase 3 Week 2 Day 2

use crate as pallet_pricing;
use frame_support::{parameter_types, traits::ConstU16};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
    testing::TestXt,
};

type Block = frame_system::mocking::MockBlock<Test>;

/// Mock extrinsic type for testing
pub type Extrinsic = TestXt<RuntimeCall, ()>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Pricing: pallet_pricing,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaxPriceDeviation: u16 = 2000; // 20% (2000 bps)
    pub const ExchangeRateUpdateInterval: u32 = 10; // 测试用较短间隔
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Block = Block;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type RuntimeTask = ();
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

impl pallet_pricing::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxPriceDeviation = MaxPriceDeviation;
    type ExchangeRateUpdateInterval = ExchangeRateUpdateInterval;
}

/// 函数级中文注释：创建测试环境
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}

