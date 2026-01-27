//! # Divination Bazi Pallet Benchmarking
//!
//! 八字命盘模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_bazi_chart() {
        let caller: T::AccountId = whitelisted_caller();
        let year: i32 = 1990;
        let month: u8 = 6;
        let day: u8 = 15;
        let hour: u8 = 12;
        let minute: u8 = 0;
        let gender: bool = true;
        let longitude: i32 = 1160000;

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            year,
            month,
            day,
            hour,
            minute,
            gender,
            longitude,
            false,
            false,
            false
        );
    }

    #[benchmark]
    fn delete_bazi_chart() {
        let caller: T::AccountId = whitelisted_caller();
        let chart_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), chart_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
