//! # Divination Meihua Pallet Benchmarking
//!
//! 梅花易数模块基准测试

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
        _(RawOrigin::Signed(caller), 0, 0, 0, true, 0, 0);
    }

    #[benchmark]
    fn divine_by_numbers() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 1, 2, 3, true, 0, 0);
    }

    #[benchmark]
    fn divine_random() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), true, 0, 0);
    }

    #[benchmark]
    fn divine_manual() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 1, 2, 3, true, 0, 0);
    }

    #[benchmark]
    fn divine_by_single_number() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 8, true, 0, 0);
    }

    #[benchmark]
    fn divine_by_gregorian_time() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 2024, 6, 15, 12, true, 0, 0);
    }

    #[benchmark]
    fn divine_by_shake() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), vec![0, 1, 0, 1, 0, 1], true, 0, 0);
    }

    #[benchmark]
    fn set_hexagram_visibility() {
        let caller: T::AccountId = whitelisted_caller();
        let hexagram_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), hexagram_id, true);
    }

    #[benchmark]
    fn delete_hexagram() {
        let caller: T::AccountId = whitelisted_caller();
        let hexagram_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), hexagram_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
