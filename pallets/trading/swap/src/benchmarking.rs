//! # Trading Swap Pallet Benchmarking
//!
//! 兑换模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn maker_swap() {
        let caller: T::AccountId = whitelisted_caller();
        let buyer: T::AccountId = account("buyer", 1, 0);
        let amount: BalanceOf<T> = 1000u32.into();
        let price: u64 = 7_000_000;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), buyer, amount, price);
    }

    #[benchmark]
    fn mark_swap_complete() {
        let caller: T::AccountId = whitelisted_caller();
        let swap_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), swap_id);
    }

    #[benchmark]
    fn report_swap() {
        let caller: T::AccountId = whitelisted_caller();
        let swap_id: u64 = 1;
        let price: u64 = 7_000_000;
        let dust_qty: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), swap_id, price, dust_qty);
    }

    #[benchmark]
    fn confirm_verification() {
        let swap_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Root, swap_id);
    }

    #[benchmark]
    fn handle_verification_timeout() {
        let swap_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Root, swap_id);
    }

    #[benchmark]
    fn ocw_submit_verification() {
        let swap_id: u64 = 1;
        let verified: bool = true;
        let reason: Option<Vec<u8>> = None;

        #[extrinsic_call]
        _(RawOrigin::Root, swap_id, verified, reason);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
