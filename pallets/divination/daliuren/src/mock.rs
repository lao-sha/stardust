//! # 测试环境配置
//!
//! 本模块提供大六壬排盘 pallet 的测试环境。

use crate as pallet_daliuren;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::ConstU64,
};
use frame_system::EnsureSigned;
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

// 构建测试运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        DaLiuRen: pallet_daliuren,
    }
);

/// 系统配置
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<u64>;
}

/// 余额配置
#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
}

/// 时间戳配置
impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<3000>;
    type WeightInfo = ();
}

parameter_types! {
    /// CID 最大长度
    pub const MaxCidLen: u32 = 64;
    /// 每日最大起课次数
    pub const MaxDailyDivinations: u32 = 100;
    /// 加密数据最大长度
    pub const MaxEncryptedLen: u32 = 512;
    /// 起课费用
    pub const DivinationFee: u64 = 1_000_000_000; // 1 DUST (12 decimals scaled down)
    /// AI 解读费用
    pub const AiInterpretationFee: u64 = 5_000_000_000; // 5 DUST
}

/// 测试随机数生成器
pub struct TestRandomness;

impl frame_support::traits::Randomness<sp_core::H256, u64> for TestRandomness {
    fn random(subject: &[u8]) -> (sp_core::H256, u64) {
        use sp_io::hashing::blake2_256;
        let block_number = System::block_number();
        let mut data = subject.to_vec();
        data.extend_from_slice(&block_number.to_le_bytes());
        (sp_core::H256::from(blake2_256(&data)), block_number)
    }
}

/// 大六壬配置
impl pallet_daliuren::Config for Test {
    type Currency = Balances;
    type Randomness = TestRandomness;
    type MaxCidLen = MaxCidLen;
    type MaxDailyDivinations = MaxDailyDivinations;
    type MaxEncryptedLen = MaxEncryptedLen;
    type DivinationFee = DivinationFee;
    type AiInterpretationFee = AiInterpretationFee;
    type AiSubmitter = EnsureSigned<u64>;
    type WeightInfo = ();
}

/// 测试账户
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const AI_SERVICE: u64 = 100;

/// 初始余额
pub const INITIAL_BALANCE: u64 = 1_000_000_000_000_000; // 1000 DUST

/// 构建测试环境
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, INITIAL_BALANCE),
            (BOB, INITIAL_BALANCE),
            (AI_SERVICE, INITIAL_BALANCE),
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// 推进区块
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}
