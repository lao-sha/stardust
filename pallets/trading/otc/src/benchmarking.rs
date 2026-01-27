//! # Trading OTC Pallet Benchmarking
//!
//! OTC 交易模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_order() {
        let caller: T::AccountId = whitelisted_caller();
        let maker: T::AccountId = account("maker", 1, 0);
        let amount: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), maker, amount);
    }

    #[benchmark]
    fn create_first_purchase() {
        let caller: T::AccountId = whitelisted_caller();
        let maker: T::AccountId = account("maker", 1, 0);
        let amount: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), maker, amount);
    }

    #[benchmark]
    fn mark_paid() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id);
    }

    #[benchmark]
    fn release_dust() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id);
    }

    #[benchmark]
    fn cancel_order() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id);
    }

    #[benchmark]
    fn dispute_order() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;
        let reason: Vec<u8> = b"Dispute".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id, reason);
    }

    #[benchmark]
    fn enable_kyc_requirement() {
        let threshold: BalanceOf<T> = 10000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Root, threshold);
    }

    #[benchmark]
    fn disable_kyc_requirement() {
        #[extrinsic_call]
        _(RawOrigin::Root);
    }

    #[benchmark]
    fn update_min_judgment_level() {
        let level: u8 = 2;

        #[extrinsic_call]
        _(RawOrigin::Root, level);
    }

    #[benchmark]
    fn exempt_account_from_kyc() {
        let account: T::AccountId = account("exempt", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, account);
    }

    #[benchmark]
    fn remove_kyc_exemption() {
        let account: T::AccountId = account("exempt", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, account);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
