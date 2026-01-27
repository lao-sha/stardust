//! # Divination Daliuren Pallet Benchmarking
//!
//! 大六壬模块基准测试

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
        _(RawOrigin::Signed(caller), 0, 0, true, None);
    }

    #[benchmark]
    fn divine_by_solar_time() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 2024, 6, 15, 12, None);
    }

    #[benchmark]
    fn divine_random() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, None);
    }

    #[benchmark]
    fn divine_manual() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, 0, 0, true, None);
    }

    #[benchmark]
    fn set_pan_visibility() {
        let caller: T::AccountId = whitelisted_caller();
        let pan_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), pan_id, true);
    }

    #[benchmark]
    fn delete_pan() {
        let caller: T::AccountId = whitelisted_caller();
        let pan_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), pan_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
