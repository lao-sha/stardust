//! # Divination Ziwei Pallet Benchmarking
//!
//! 紫微斗数模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn divine_by_time() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 1990, 6, 15, 0, true, false);
    }

    #[benchmark]
    fn divine_manual() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, 0, 0, 0, true);
    }

    #[benchmark]
    fn divine_random() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn divine_by_solar_time() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 1990, 6, 15, 0, true);
    }

    #[benchmark]
    fn set_chart_visibility() {
        let caller: T::AccountId = whitelisted_caller();
        let chart_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), chart_id, true);
    }

    #[benchmark]
    fn delete_chart() {
        let caller: T::AccountId = whitelisted_caller();
        let chart_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), chart_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
