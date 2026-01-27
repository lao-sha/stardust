//! # Escrow Pallet Benchmarking
//!
//! 托管模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn lock() {
        let caller: T::AccountId = whitelisted_caller();
        let id: u64 = 1;
        let amount: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), id, amount);
    }

    #[benchmark]
    fn release() {
        let caller: T::AccountId = whitelisted_caller();
        let id: u64 = 1;
        let to: T::AccountId = account("receiver", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), id, to);
    }

    #[benchmark]
    fn refund() {
        let caller: T::AccountId = whitelisted_caller();
        let id: u64 = 1;
        let to: T::AccountId = account("refundee", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), id, to);
    }

    #[benchmark]
    fn lock_with_nonce() {
        let caller: T::AccountId = whitelisted_caller();
        let id: u64 = 1;
        let nonce: u32 = 1;
        let amount: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), id, nonce, amount);
    }

    #[benchmark]
    fn release_split() {
        let caller: T::AccountId = whitelisted_caller();
        let id: u64 = 1;
        let splits: Vec<(T::AccountId, BalanceOf<T>)> = vec![];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), id, splits);
    }

    #[benchmark]
    fn dispute() {
        let id: u64 = 1;
        let reason: u16 = 1;

        #[extrinsic_call]
        _(RawOrigin::Root, id, reason);
    }

    #[benchmark]
    fn apply_decision_release_all() {
        let id: u64 = 1;
        let to: T::AccountId = account("receiver", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, id, to);
    }

    #[benchmark]
    fn apply_decision_refund_all() {
        let id: u64 = 1;
        let to: T::AccountId = account("refundee", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, id, to);
    }

    #[benchmark]
    fn apply_decision_partial_bps() {
        let id: u64 = 1;
        let release_to: T::AccountId = account("receiver", 1, 0);
        let refund_to: T::AccountId = account("refundee", 2, 0);
        let release_bps: u16 = 5000;

        #[extrinsic_call]
        _(RawOrigin::Root, id, release_to, refund_to, release_bps);
    }

    #[benchmark]
    fn set_pause() {
        #[extrinsic_call]
        _(RawOrigin::Root, true);
    }

    #[benchmark]
    fn schedule_expiry() {
        let id: u64 = 1;
        let expires_at: BlockNumberFor<T> = 1000u32.into();
        let action: u8 = 0;

        #[extrinsic_call]
        _(RawOrigin::Root, id, expires_at, action);
    }

    #[benchmark]
    fn cancel_expiry() {
        let id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Root, id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
