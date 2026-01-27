//! 群聊模块测试 Mock

use crate as pallet_chat_group;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128, Randomness as RandomnessTrait},
    PalletId,
};
use frame_system::EnsureRoot;
use sp_runtime::{BuildStorage, traits::Hash};
use sp_core::H256;

type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;

// 简单的测试用随机数生成器
pub struct TestRandomness;
impl RandomnessTrait<H256, u64> for TestRandomness {
    fn random(subject: &[u8]) -> (H256, u64) {
        let block_number = frame_system::Pallet::<Test>::block_number();
        let hash = sp_io::hashing::blake2_256(subject);
        (H256::from(hash), block_number)
    }
}

// 配置测试运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        ChatGroup: pallet_chat_group,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<u128>;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<1>;
    type WeightInfo = ();
}

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
    type MaxFreezes = ConstU32<0>;
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
}

parameter_types! {
    pub const GroupPalletId: PalletId = PalletId(*b"py/group");
    pub const TreasuryAccountId: u64 = 999;
}

impl pallet_chat_group::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Randomness = TestRandomness;
    type TimeProvider = Timestamp;
    type MaxGroupNameLen = ConstU32<64>;
    type MaxGroupDescriptionLen = ConstU32<512>;
    type MaxGroupMembers = ConstU32<500>;
    type MaxGroupsPerUser = ConstU32<100>;
    type MaxMessageLen = ConstU32<2048>;
    type MaxGroupMessageHistory = ConstU32<1000>;
    type MaxCidLen = ConstU32<128>;
    type MaxKeyLen = ConstU32<256>;
    type PalletId = GroupPalletId;
    type MessageRateLimit = ConstU32<60>;
    type GroupCreationCooldown = ConstU64<10>;
    type GroupDeposit = ConstU128<50_000_000_000_000_000_000>; // 50 DUST 兜底
    type GroupDepositUsd = ConstU64<5_000_000>; // 5 USDT
    type DepositCalculator = (); // 使用空实现，返回兜底值
    type TreasuryAccount = TreasuryAccountId;
    type GovernanceOrigin = EnsureRoot<u64>;
    type WeightInfo = ();
}

/// 构建测试外部环境
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        // Timestamp 会自动设置，不需要手动设置
    });
    ext
}

/// 测试账户
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;
