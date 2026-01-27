//! Mock runtime for testing the membership pallet.

use crate as pallet_divination_membership;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128},
    PalletId,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Membership: pallet_divination_membership,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

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
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
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

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

parameter_types! {
    pub const MembershipPalletId: PalletId = PalletId(*b"py/membr");
    pub const TreasuryAccountId: u64 = 1000;
    pub const BurnAccountId: u64 = 9999;
    pub const RewardPoolAllocation: u32 = 1000; // 10%
    pub const NewAccountCooldown: u64 = 100; // 100 blocks for testing (vs 50400 in prod)
    pub const MinBalanceForRewards: u128 = 1_000_000_000_000; // 1 DUST
    pub const BlocksPerDay: u64 = 10; // 10 blocks for testing (vs 7200 in prod)
    pub const BlocksPerMonth: u64 = 300; // 300 blocks for testing (vs 216000 in prod)
    pub const MaxDisplayNameLength: u32 = 32;
    pub const MaxEncryptedDataLength: u32 = 256;
    pub const MaxRewardHistorySize: u32 = 100;
}

/// Mock UserFundingProvider 实现
pub struct MockUserFundingProvider;

impl pallet_affiliate::UserFundingProvider<u64> for MockUserFundingProvider {
    fn derive_user_funding_account(user: &u64) -> u64 {
        *user + 10000
    }
}

/// Mock AffiliateDistributor 实现
pub struct MockAffiliateDistributor;

impl pallet_affiliate::types::AffiliateDistributor<u64, u128, u64> for MockAffiliateDistributor {
    fn distribute_rewards(
        _buyer: &u64,
        _amount: u128,
        _target: Option<(u8, u64)>,
    ) -> Result<u128, sp_runtime::DispatchError> {
        Ok(0) // 不分配任何奖励
    }
}

/// Mock PricingProvider 实现
pub struct MockPricingProvider;

impl pallet_trading_common::PricingProvider<u128> for MockPricingProvider {
    fn get_dust_to_usd_rate() -> Option<u128> {
        Some(1_000_000) // 1 DUST = 1 USD
    }
    
    fn report_swap_order(_timestamp: u64, _price_usdt: u64, _dust_qty: u128) -> sp_runtime::DispatchResult {
        Ok(())
    }
}

impl pallet_divination_membership::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = ();
    type PalletId = MembershipPalletId;
    type TreasuryAccount = TreasuryAccountId;
    type BurnAccount = BurnAccountId;
    type UserFundingProvider = MockUserFundingProvider;
    type AffiliateDistributor = MockAffiliateDistributor;
    type RewardPoolAllocation = RewardPoolAllocation;
    type NewAccountCooldown = NewAccountCooldown;
    type MinBalanceForRewards = MinBalanceForRewards;
    type BlocksPerDay = BlocksPerDay;
    type BlocksPerMonth = BlocksPerMonth;
    type MaxDisplayNameLength = MaxDisplayNameLength;
    type MaxEncryptedDataLength = MaxEncryptedDataLength;
    type MaxRewardHistorySize = MaxRewardHistorySize;
    type Pricing = MockPricingProvider;
}

/// Build genesis storage for testing.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1000 * DUST),  // User 1: 1000 DUST
            (2, 500 * DUST),   // User 2: 500 DUST
            (3, 10 * DUST),    // User 3: 10 DUST
            // User 4 is omitted (zero balance)
            (TreasuryAccountId::get(), 1), // Treasury (needs existential deposit)
            (Membership::reward_pool_account(), 100_000 * DUST), // Reward pool
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// DUST unit (10^12).
pub const DUST: u128 = 1_000_000_000_000;

/// Advance to specified block number.
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}

/// Advance by specified number of blocks.
pub fn advance_blocks(n: u64) {
    let target = System::block_number() + n;
    run_to_block(target);
}

/// Get account balance.
pub fn balance(who: u64) -> u128 {
    Balances::free_balance(who)
}

/// Get reward pool balance.
pub fn reward_pool_balance() -> u128 {
    Balances::free_balance(Membership::reward_pool_account())
}

/// Get treasury balance.
pub fn treasury_balance() -> u128 {
    Balances::free_balance(TreasuryAccountId::get())
}
