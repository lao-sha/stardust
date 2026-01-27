//! # Matchmaking Membership Pallet Benchmarking
//!
//! 婚恋会员模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn subscribe() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            SubscriptionDuration::OneMonth,
            false,
            None
        );
    }

    #[benchmark]
    fn renew() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), SubscriptionDuration::OneMonth);
    }

    #[benchmark]
    fn upgrade() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn cancel_auto_renew() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn use_benefit() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), BenefitType::Recommendation);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
