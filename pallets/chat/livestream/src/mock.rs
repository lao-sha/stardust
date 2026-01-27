//! 直播间模块测试 Mock

use crate as pallet_livestream;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU128, ConstU32, ConstU64, ConstU8},
    PalletId,
};
use frame_system::EnsureRoot;
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

// 配置测试运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Livestream: pallet_livestream,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Balance>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
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

parameter_types! {
    pub const LivestreamPalletId: PalletId = PalletId(*b"py/lives");
}

impl pallet_livestream::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxTitleLen = ConstU32<100>;
    type MaxDescriptionLen = ConstU32<500>;
    type MaxCidLen = ConstU32<64>;
    type MaxGiftNameLen = ConstU32<32>;
    type MaxCoHostsPerRoom = ConstU32<4>;
    type PlatformFeePercent = ConstU8<20>;
    type MinWithdrawAmount = ConstU128<1_000_000_000_000>; // 1 DUST
    type RoomBond = ConstU128<10_000_000_000_000>; // 10 DUST 兜底
    type RoomBondUsd = ConstU64<5_000_000>; // 5 USDT
    type DepositCalculator = (); // 使用空实现，返回兜底值
    type PalletId = LivestreamPalletId;
    type GovernanceOrigin = EnsureRoot<u64>;
    type WeightInfo = ();
}

/// 构建测试外部环境
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000_000_000_000), // 1000 DUST
            (2, 1_000_000_000_000_000),
            (3, 1_000_000_000_000_000),
            (4, 100_000_000_000_000),   // 100 DUST
            (5, 100_000_000_000_000),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // 初始化直播间模块 (空礼物列表，测试中单独创建)
    pallet_livestream::GenesisConfig::<Test> {
        gifts: vec![],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

/// 测试账户
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;
