//! # Divination Liuyao Pallet Benchmarking
//!
//! 六爻模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn divine_by_coins() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, 0, 0, vec![0, 0, 0, 0, 0, 0]);
    }

    #[benchmark]
    fn divine_by_numbers() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 1, 2, 3);
    }

    #[benchmark]
    fn divine_random() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn divine_manual() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), vec![0, 1, 0, 1, 0, 1]);
    }

    #[benchmark]
    fn divine_by_time() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, 0, 0);
    }

    #[benchmark]
    fn divine_by_solar_time() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 2024, 6, 15, 12);
    }

    #[benchmark]
    fn set_gua_visibility() {
        let caller: T::AccountId = whitelisted_caller();
        let gua_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), gua_id, true);
    }

    #[benchmark]
    fn delete_gua() {
        let caller: T::AccountId = whitelisted_caller();
        let gua_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), gua_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
