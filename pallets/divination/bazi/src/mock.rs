//! # 测试模拟环境
//!
//! 为单元测试提供模拟的运行时环境

use crate as pallet_bazi_chart;
use frame_support::{
	derive_impl,
	dispatch::DispatchResult,
	parameter_types,
	traits::ConstU32,
};
use sp_runtime::BuildStorage;
use pallet_divination_privacy::types::PrivacyMode;
use pallet_divination_common::DivinationType;

type Block = frame_system::mocking::MockBlock<Test>;

/// Mock PrivacyProvider 实现（用于测试）
pub struct MockPrivacyProvider;

impl pallet_divination_privacy::traits::EncryptedRecordManager<u64, u64> for MockPrivacyProvider {
	fn create_record(
		_owner: &u64,
		_divination_type: DivinationType,
		_result_id: u64,
		_privacy_mode: PrivacyMode,
		_encrypted_data: sp_std::vec::Vec<u8>,
		_nonce: [u8; 24],
		_auth_tag: [u8; 16],
		_data_hash: [u8; 32],
		_owner_encrypted_key: sp_std::vec::Vec<u8>,
	) -> DispatchResult {
		Ok(())
	}

	fn delete_record(
		_owner: &u64,
		_divination_type: DivinationType,
		_result_id: u64,
	) -> DispatchResult {
		Ok(())
	}

	fn grant_access(
		_grantor: &u64,
		_divination_type: DivinationType,
		_result_id: u64,
		_grantee: &u64,
		_encrypted_key: sp_std::vec::Vec<u8>,
		_role: pallet_divination_privacy::types::AccessRole,
		_scope: pallet_divination_privacy::types::AccessScope,
		_expires_at: u64,
		_bounty_id: Option<u64>,
	) -> DispatchResult {
		Ok(())
	}

	fn revoke_access(
		_grantor: &u64,
		_divination_type: DivinationType,
		_result_id: u64,
		_grantee: &u64,
	) -> DispatchResult {
		Ok(())
	}
}

// 配置测试运行时
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		BaziChart: pallet_bazi_chart,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountData = pallet_balances::AccountData<u128>;
}

// 配置 Balances pallet
parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = u128;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
}

impl pallet_bazi_chart::Config for Test {
	type WeightInfo = ();
	type MaxChartsPerAccount = ConstU32<10>;
	type MaxDaYunSteps = ConstU32<12>;
	type MaxCangGan = ConstU32<3>;
	type PrivacyProvider = MockPrivacyProvider;
	type Currency = Balances;
}

// 构建测试用的存储
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	// 为测试账户配置初始余额
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 1_000_000_000),  // 账户 1 有 10 亿余额
			(2, 1_000_000_000),  // 账户 2 有 10 亿余额
		],
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
