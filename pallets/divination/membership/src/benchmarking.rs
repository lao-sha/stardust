//! # Divination Membership Pallet Benchmarking
//!
//! 占卜会员模块基准测试

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
        _(RawOrigin::Signed(caller), MembershipTier::Basic, SubscriptionDuration::Monthly, false);
    }

    #[benchmark]
    fn upgrade_tier() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), MembershipTier::Premium);
    }

    #[benchmark]
    fn cancel_subscription() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn check_in() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn update_profile() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), None, None, None, None, None, None);
    }

    #[benchmark]
    fn clear_sensitive_data() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn apply_provider() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn verify_provider() {
        let provider: T::AccountId = account("provider", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, provider, true);
    }

    #[benchmark]
    fn use_free_ai() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
