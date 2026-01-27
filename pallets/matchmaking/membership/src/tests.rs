//! # 婚恋会员模块 - 单元测试

use crate::{self as pallet_matchmaking_membership, *};
use frame_support::{
    assert_noop, assert_ok,
    parameter_types,
    traits::{ConstU32, ConstU128},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        MatchmakingMembership: pallet_matchmaking_membership,
    }
);

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
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = ();
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const BlocksPerMonth: u64 = 216000;
    pub const BlocksPerDay: u64 = 7200;
    pub const MonthlyFee: u128 = 10_000_000_000_000; // 10 DUST
    pub const MonthlyFeeUsd: u64 = 10_000_000; // 10 USDT
    pub const LifetimeFee: u128 = 500_000_000_000_000; // 500 DUST
    pub const LifetimeFeeUsd: u64 = 500_000_000; // 500 USDT
    pub TreasuryAccount: u64 = 100;
    pub BurnAccount: u64 = 101;
}

/// Mock UserFundingProvider
pub struct MockUserFundingProvider;

impl pallet_affiliate::UserFundingProvider<u64> for MockUserFundingProvider {
    fn derive_user_funding_account(user: &u64) -> u64 {
        *user + 10000
    }
}

/// Mock Pricing Provider
pub struct MockPricing;

impl pallet_trading_common::PricingProvider<u128> for MockPricing {
    fn get_dust_to_usd_rate() -> Option<u128> {
        // 返回 None 使用兜底值
        None
    }

    fn report_swap_order(_timestamp: u64, _price_usdt: u64, _dust_qty: u128) -> sp_runtime::DispatchResult {
        Ok(())
    }
}

/// Mock Affiliate Distributor
pub struct MockAffiliateDistributor;

impl pallet_affiliate::types::AffiliateDistributor<u64, u128, u64> for MockAffiliateDistributor {
    fn distribute_rewards(
        _buyer: &u64,
        _amount: u128,
        _target: Option<(u8, u64)>,
    ) -> Result<u128, sp_runtime::DispatchError> {
        Ok(0)
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Fungible = Balances;
    type Balance = u128;
    type BlocksPerMonth = BlocksPerMonth;
    type BlocksPerDay = BlocksPerDay;
    type MonthlyFee = MonthlyFee;
    type MonthlyFeeUsd = MonthlyFeeUsd;
    type LifetimeFee = LifetimeFee;
    type LifetimeFeeUsd = LifetimeFeeUsd;
    type Pricing = MockPricing;
    type TreasuryAccount = TreasuryAccount;
    type BurnAccount = BurnAccount;
    type UserFundingProvider = MockUserFundingProvider;
    type AffiliateDistributor = MockAffiliateDistributor;
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000_000_000_000), // 1000 DUST
            (2, 1_000_000_000_000_000),
            (3, 100_000_000_000),       // 0.1 DUST (不足)
            (100, 1), // Treasury (最小存在存款)
            (101, 1), // Burn
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

#[test]
fn subscribe_works() {
    new_test_ext().execute_with(|| {
        // 订阅一个月
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));

        // 检查会员状态
        let membership = Memberships::<Test>::get(1).unwrap();
        assert_eq!(membership.tier, MembershipTier::Annual);
        assert!(!membership.auto_renew);

        // 检查统计
        let stats = GlobalStats::<Test>::get();
        assert_eq!(stats.annual_count, 1);
    });
}

#[test]
fn subscribe_with_discount() {
    new_test_ext().execute_with(|| {
        let initial_balance = Balances::free_balance(1);

        // 订阅一年（20% 折扣）
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneYear,
            true,
            None,
        ));

        // 检查扣费（12个月 * 10 DUST * 80% = 96 DUST）
        let expected_fee = 10_000_000_000_000u128 * 12 * 8000 / 10000;
        let actual_balance = Balances::free_balance(1);
        assert_eq!(initial_balance - actual_balance, expected_fee);
    });
}

#[test]
fn subscribe_insufficient_balance() {
    new_test_ext().execute_with(|| {
        // 账户 3 余额不足
        assert_noop!(
            MatchmakingMembership::subscribe(
                RuntimeOrigin::signed(3),
                SubscriptionDuration::OneMonth,
                false,
                None,
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn subscribe_already_member() {
    new_test_ext().execute_with(|| {
        // 首次订阅
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));

        // 重复订阅应失败
        assert_noop!(
            MatchmakingMembership::subscribe(
                RuntimeOrigin::signed(1),
                SubscriptionDuration::OneMonth,
                false,
                None,
            ),
            Error::<Test>::AlreadyMember
        );
    });
}

#[test]
fn renew_works() {
    new_test_ext().execute_with(|| {
        // 首次订阅
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));

        let membership_before = Memberships::<Test>::get(1).unwrap();

        // 续费
        assert_ok!(MatchmakingMembership::renew(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::ThreeMonths,
        ));

        let membership_after = Memberships::<Test>::get(1).unwrap();
        
        // 到期时间应该延长
        assert!(membership_after.expires_at > membership_before.expires_at);
        // 连续月数应该增加
        assert_eq!(membership_after.consecutive_months, 1 + 3);
    });
}

#[test]
fn upgrade_to_lifetime() {
    new_test_ext().execute_with(|| {
        // 首次订阅年费会员
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneYear,
            false,
            None,
        ));

        // 升级到终身会员
        assert_ok!(MatchmakingMembership::upgrade(RuntimeOrigin::signed(1)));

        let membership = Memberships::<Test>::get(1).unwrap();
        assert_eq!(membership.tier, MembershipTier::Lifetime);
        assert_eq!(membership.expires_at, u64::MAX);
    });
}

#[test]
fn upgrade_already_lifetime() {
    new_test_ext().execute_with(|| {
        // 直接订阅终身会员
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::Lifetime,
            false,
            None,
        ));

        // 再次升级应失败
        assert_noop!(
            MatchmakingMembership::upgrade(RuntimeOrigin::signed(1)),
            Error::<Test>::AlreadyLifetime
        );
    });
}

#[test]
fn cancel_auto_renew_works() {
    new_test_ext().execute_with(|| {
        // 订阅并开启自动续费
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            true,
            None,
        ));

        let membership = Memberships::<Test>::get(1).unwrap();
        assert!(membership.auto_renew);

        // 取消自动续费
        assert_ok!(MatchmakingMembership::cancel_auto_renew(RuntimeOrigin::signed(1)));

        let membership = Memberships::<Test>::get(1).unwrap();
        assert!(!membership.auto_renew);
    });
}

#[test]
fn use_benefit_works() {
    new_test_ext().execute_with(|| {
        // 订阅会员
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));

        // 使用推荐功能
        assert_ok!(MatchmakingMembership::use_benefit(
            RuntimeOrigin::signed(1),
            BenefitType::Recommendation,
        ));

        let usage = DailyUsages::<Test>::get(1);
        assert_eq!(usage.recommendations_used, 1);
    });
}

#[test]
fn use_benefit_daily_limit() {
    new_test_ext().execute_with(|| {
        // 免费用户
        // 免费用户每日推荐限制为 10 次
        for _ in 0..10 {
            assert_ok!(MatchmakingMembership::use_benefit(
                RuntimeOrigin::signed(1),
                BenefitType::Recommendation,
            ));
        }

        // 第 11 次应该失败
        assert_noop!(
            MatchmakingMembership::use_benefit(
                RuntimeOrigin::signed(1),
                BenefitType::Recommendation,
            ),
            Error::<Test>::DailyLimitReached
        );
    });
}

#[test]
fn free_user_no_super_like() {
    new_test_ext().execute_with(|| {
        // 免费用户没有超级喜欢权益
        assert_noop!(
            MatchmakingMembership::use_benefit(
                RuntimeOrigin::signed(1),
                BenefitType::SuperLike,
            ),
            Error::<Test>::BenefitNotAvailable
        );
    });
}

#[test]
fn membership_benefits_correct() {
    // 测试权益配置
    let free = MembershipBenefits::free_tier();
    assert_eq!(free.daily_recommendations, 10);
    assert_eq!(free.daily_super_likes, 0);
    assert!(!free.can_see_who_likes_me);

    let annual = MembershipBenefits::annual_tier();
    assert_eq!(annual.daily_recommendations, 50);
    assert_eq!(annual.daily_super_likes, 5);
    assert!(annual.can_see_who_likes_me);

    let lifetime = MembershipBenefits::lifetime_tier();
    assert_eq!(lifetime.daily_recommendations, 100);
    assert_eq!(lifetime.daily_super_likes, 10);
    assert!(lifetime.dedicated_support);
}

#[test]
fn get_tier_works() {
    new_test_ext().execute_with(|| {
        // 未订阅用户
        assert_eq!(MatchmakingMembership::get_tier(&1), MembershipTier::Free);

        // 订阅后
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));
        assert_eq!(MatchmakingMembership::get_tier(&1), MembershipTier::Annual);
    });
}

// ============================================================================
// 补充测试：Trait 实现
// ============================================================================

#[test]
fn is_active_member_works() {
    new_test_ext().execute_with(|| {
        use crate::traits::MembershipProvider;

        // 未订阅用户不是活跃会员
        assert!(!MatchmakingMembership::is_active_member(&1));

        // 订阅后是活跃会员
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));
        assert!(MatchmakingMembership::is_active_member(&1));
    });
}

#[test]
fn get_expiry_works() {
    new_test_ext().execute_with(|| {
        use crate::traits::MembershipProvider;

        // 未订阅用户无到期时间
        assert_eq!(MatchmakingMembership::get_expiry(&1), None);

        // 订阅后有到期时间
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));
        let expiry = MatchmakingMembership::get_expiry(&1);
        assert!(expiry.is_some());
        assert!(expiry.unwrap() > 0);
    });
}

#[test]
fn get_benefits_works() {
    new_test_ext().execute_with(|| {
        use crate::traits::MembershipProvider;

        // 免费用户权益
        let free_benefits = MatchmakingMembership::get_benefits(&1);
        assert_eq!(free_benefits.daily_recommendations, 10);
        assert_eq!(free_benefits.daily_super_likes, 0);

        // 年费会员权益
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));
        let annual_benefits = MatchmakingMembership::get_benefits(&1);
        assert_eq!(annual_benefits.daily_recommendations, 50);
        assert_eq!(annual_benefits.daily_super_likes, 5);
    });
}

#[test]
fn has_benefit_works() {
    new_test_ext().execute_with(|| {
        use crate::traits::{MembershipProvider, MembershipBenefit};

        // 免费用户没有高级权益
        assert!(!MatchmakingMembership::has_benefit(&1, MembershipBenefit::SeeWhoLikesMe));
        assert!(!MatchmakingMembership::has_benefit(&1, MembershipBenefit::DedicatedSupport));

        // 年费会员有部分高级权益
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));
        assert!(MatchmakingMembership::has_benefit(&1, MembershipBenefit::SeeWhoLikesMe));
        assert!(!MatchmakingMembership::has_benefit(&1, MembershipBenefit::DedicatedSupport));

        // 终身会员有全部权益
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(2),
            SubscriptionDuration::Lifetime,
            false,
            None,
        ));
        assert!(MatchmakingMembership::has_benefit(&2, MembershipBenefit::DedicatedSupport));
    });
}

// ============================================================================
// 补充测试：MembershipUsageTracker Trait
// ============================================================================

#[test]
fn can_use_recommendation_works() {
    new_test_ext().execute_with(|| {
        use crate::traits::MembershipUsageTracker;

        // 初始可以使用
        assert!(MatchmakingMembership::can_use_recommendation(&1));

        // 使用到限额后不能使用
        for _ in 0..10 {
            assert_ok!(MatchmakingMembership::use_benefit(
                RuntimeOrigin::signed(1),
                BenefitType::Recommendation,
            ));
        }
        assert!(!MatchmakingMembership::can_use_recommendation(&1));
    });
}

#[test]
fn can_use_super_like_works() {
    new_test_ext().execute_with(|| {
        use crate::traits::MembershipUsageTracker;

        // 免费用户不能使用超级喜欢
        assert!(!MatchmakingMembership::can_use_super_like(&1));

        // 年费会员可以使用
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));
        assert!(MatchmakingMembership::can_use_super_like(&1));
    });
}

#[test]
fn can_use_compatibility_check_works() {
    new_test_ext().execute_with(|| {
        use crate::traits::MembershipUsageTracker;

        // 免费用户每天1次
        assert!(MatchmakingMembership::can_use_compatibility_check(&1));

        // 使用后不能再用
        assert_ok!(MatchmakingMembership::use_benefit(
            RuntimeOrigin::signed(1),
            BenefitType::CompatibilityCheck,
        ));
        assert!(!MatchmakingMembership::can_use_compatibility_check(&1));
    });
}

#[test]
fn record_usage_via_trait_works() {
    new_test_ext().execute_with(|| {
        use crate::traits::MembershipUsageTracker;

        // 记录推荐使用
        assert!(MatchmakingMembership::record_recommendation_usage(&1).is_ok());
        let usage = DailyUsages::<Test>::get(1);
        assert_eq!(usage.recommendations_used, 1);

        // 年费会员记录超级喜欢
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(2),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));
        assert!(MatchmakingMembership::record_super_like_usage(&2).is_ok());
        let usage2 = DailyUsages::<Test>::get(2);
        assert_eq!(usage2.super_likes_used, 1);

        // 记录合婚分析
        assert!(MatchmakingMembership::record_compatibility_check_usage(&1).is_ok());
        let usage3 = DailyUsages::<Test>::get(1);
        assert_eq!(usage3.compatibility_checks_used, 1);
    });
}

// ============================================================================
// 补充测试：错误场景
// ============================================================================

#[test]
fn renew_not_a_member() {
    new_test_ext().execute_with(|| {
        // 非会员续费应失败
        assert_noop!(
            MatchmakingMembership::renew(
                RuntimeOrigin::signed(1),
                SubscriptionDuration::OneMonth,
            ),
            Error::<Test>::NotAMember
        );
    });
}

#[test]
fn renew_lifetime_fails() {
    new_test_ext().execute_with(|| {
        // 终身会员续费应失败
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::Lifetime,
            false,
            None,
        ));

        assert_noop!(
            MatchmakingMembership::renew(
                RuntimeOrigin::signed(1),
                SubscriptionDuration::OneMonth,
            ),
            Error::<Test>::AlreadyLifetime
        );
    });
}

#[test]
fn upgrade_not_a_member() {
    new_test_ext().execute_with(|| {
        // 非会员升级应失败
        assert_noop!(
            MatchmakingMembership::upgrade(RuntimeOrigin::signed(1)),
            Error::<Test>::NotAMember
        );
    });
}

#[test]
fn cancel_auto_renew_not_a_member() {
    new_test_ext().execute_with(|| {
        // 非会员取消自动续费应失败
        assert_noop!(
            MatchmakingMembership::cancel_auto_renew(RuntimeOrigin::signed(1)),
            Error::<Test>::NotAMember
        );
    });
}

// ============================================================================
// 补充测试：费用分配
// ============================================================================

#[test]
fn fee_distribution_works() {
    new_test_ext().execute_with(|| {
        let initial_balance = Balances::free_balance(1);
        let treasury_initial = Balances::free_balance(100);
        let burn_initial = Balances::free_balance(101);

        // 订阅一个月
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));

        let fee = 10_000_000_000_000u128; // MonthlyFee

        // 验证用户扣费
        assert_eq!(Balances::free_balance(1), initial_balance - fee);

        // 验证销毁账户收到 5%
        let burn_amount = fee * 5 / 100;
        assert_eq!(Balances::free_balance(101), burn_initial + burn_amount);

        // 验证国库收到 2% + 90%（无推荐人时推荐链部分也进国库）
        let treasury_amount = fee * 2 / 100;
        let storage_amount = fee * 3 / 100;
        let distributable = fee - burn_amount - treasury_amount - storage_amount;
        // 无推荐人，distributable 也进国库
        assert_eq!(
            Balances::free_balance(100),
            treasury_initial + treasury_amount + distributable
        );
    });
}

// ============================================================================
// 补充测试：带推荐人订阅
// ============================================================================

#[test]
fn subscribe_with_referrer() {
    new_test_ext().execute_with(|| {
        // 订阅时指定推荐人
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            Some(2),
        ));

        // 检查会员信息中记录了推荐人
        let membership = Memberships::<Test>::get(1).unwrap();
        assert!(membership.referrer.is_some());
    });
}

// ============================================================================
// 补充测试：合婚分析权益
// ============================================================================

#[test]
fn use_compatibility_check_works() {
    new_test_ext().execute_with(|| {
        // 免费用户每天1次合婚分析
        assert_ok!(MatchmakingMembership::use_benefit(
            RuntimeOrigin::signed(1),
            BenefitType::CompatibilityCheck,
        ));

        let usage = DailyUsages::<Test>::get(1);
        assert_eq!(usage.compatibility_checks_used, 1);

        // 第2次应该失败
        assert_noop!(
            MatchmakingMembership::use_benefit(
                RuntimeOrigin::signed(1),
                BenefitType::CompatibilityCheck,
            ),
            Error::<Test>::DailyLimitReached
        );
    });
}

#[test]
fn annual_member_more_compatibility_checks() {
    new_test_ext().execute_with(|| {
        // 年费会员每天10次
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));

        for _ in 0..10 {
            assert_ok!(MatchmakingMembership::use_benefit(
                RuntimeOrigin::signed(1),
                BenefitType::CompatibilityCheck,
            ));
        }

        // 第11次应该失败
        assert_noop!(
            MatchmakingMembership::use_benefit(
                RuntimeOrigin::signed(1),
                BenefitType::CompatibilityCheck,
            ),
            Error::<Test>::DailyLimitReached
        );
    });
}

// ============================================================================
// 补充测试：全局统计
// ============================================================================

#[test]
fn global_stats_updated() {
    new_test_ext().execute_with(|| {
        let stats_before = GlobalStats::<Test>::get();
        assert_eq!(stats_before.annual_count, 0);
        assert_eq!(stats_before.lifetime_count, 0);

        // 订阅年费会员
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(1),
            SubscriptionDuration::OneMonth,
            false,
            None,
        ));

        let stats_after = GlobalStats::<Test>::get();
        assert_eq!(stats_after.annual_count, 1);
        assert!(stats_after.total_revenue > stats_before.total_revenue);

        // 订阅终身会员
        assert_ok!(MatchmakingMembership::subscribe(
            RuntimeOrigin::signed(2),
            SubscriptionDuration::Lifetime,
            false,
            None,
        ));

        let stats_final = GlobalStats::<Test>::get();
        assert_eq!(stats_final.lifetime_count, 1);
    });
}

// ============================================================================
// 补充测试：SubscriptionDuration 方法
// ============================================================================

#[test]
fn subscription_duration_months() {
    assert_eq!(SubscriptionDuration::OneMonth.months(), 1);
    assert_eq!(SubscriptionDuration::ThreeMonths.months(), 3);
    assert_eq!(SubscriptionDuration::SixMonths.months(), 6);
    assert_eq!(SubscriptionDuration::OneYear.months(), 12);
    assert_eq!(SubscriptionDuration::Lifetime.months(), u32::MAX);
}

#[test]
fn subscription_duration_discount_rate() {
    assert_eq!(SubscriptionDuration::OneMonth.discount_rate(), 10000);
    assert_eq!(SubscriptionDuration::ThreeMonths.discount_rate(), 9500);
    assert_eq!(SubscriptionDuration::SixMonths.discount_rate(), 9000);
    assert_eq!(SubscriptionDuration::OneYear.discount_rate(), 8000);
}
