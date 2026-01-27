//! # 六爻 Pallet 测试 Mock
//!
//! 本模块提供测试环境的 Mock Runtime 配置

use crate as pallet_liuyao;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128, Hooks},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// 构建 Mock Runtime
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Liuyao: pallet_liuyao,
    }
);

/// 系统配置
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
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

/// 余额配置
impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<50>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

/// 时间戳配置
impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

/// 测试用随机数生成器
pub struct TestRandomness;

impl frame_support::traits::Randomness<H256, u64> for TestRandomness {
    fn random(subject: &[u8]) -> (H256, u64) {
        let mut hash = [0u8; 32];
        for (i, byte) in subject.iter().enumerate() {
            hash[i % 32] ^= byte;
        }
        // 添加更多随机性
        for i in 0..32 {
            hash[i] = hash[i].wrapping_add((i * 7) as u8);
        }
        (H256::from(hash), 0)
    }
}


/// 六爻 Pallet 配置
///
/// 注：RuntimeEvent 关联类型已从 Polkadot SDK 2506 版本开始自动附加
impl pallet_liuyao::Config for Test {
    type Randomness = TestRandomness;
    type MaxUserGuas = ConstU32<100>;
    type MaxPublicGuas = ConstU32<1000>;
    type DailyFreeGuas = ConstU32<3>;
    type MaxDailyGuas = ConstU32<10>;
    type MaxCidLen = ConstU32<64>;
    type MaxEncryptedLen = ConstU32<512>;
    type Currency = Balances;
}

/// 测试账户 ID
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;

/// 构建测试外部性
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 100_000),
            (BOB, 100_000),
            (CHARLIE, 100_000),
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1704067200000); // 2024-01-01
    });
    ext
}

/// 前进到指定区块
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        <System as Hooks<u64>>::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        <System as Hooks<u64>>::on_initialize(System::block_number());
        Timestamp::set_timestamp(Timestamp::get() + 6000);
    }
}

