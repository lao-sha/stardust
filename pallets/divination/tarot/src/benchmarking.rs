//! # Divination Tarot Pallet Benchmarking
//!
//! 塔罗牌模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn divine_random() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, None, 0);
    }

    #[benchmark]
    fn divine_by_time() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, None, 0);
    }

    #[benchmark]
    fn divine_by_numbers() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), vec![1, 2, 3], 0, None, 0);
    }

    #[benchmark]
    fn divine_manual() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), vec![(0, false)], 0, None, 0);
    }

    #[benchmark]
    fn divine_random_with_cut() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, Some(10), None, 0);
    }

    #[benchmark]
    fn set_reading_privacy_mode() {
        let caller: T::AccountId = whitelisted_caller();
        let reading_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), reading_id, 0);
    }

    #[benchmark]
    fn delete_reading() {
        let caller: T::AccountId = whitelisted_caller();
        let reading_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), reading_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
