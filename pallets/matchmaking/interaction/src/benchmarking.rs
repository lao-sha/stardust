//! # Matchmaking Interaction Pallet Benchmarking
//!
//! 婚恋互动模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn initialize_salt() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn like() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn super_like() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn pass() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn block_user() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn unblock_user() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn verify_interaction() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn mark_super_like_viewed() {
        let caller: T::AccountId = whitelisted_caller();
        let from: T::AccountId = account("from", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), from);
    }

    #[benchmark]
    fn initiate_matchmaking_chat() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn view_profile() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
