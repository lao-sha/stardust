//! # Divination Almanac Pallet Benchmarking
//!
//! 黄历模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_almanac() {
        let year: u16 = 2024;
        let month: u8 = 1;
        let day: u8 = 1;
        let data: Vec<u8> = b"AlmanacData".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, year, month, day, data);
    }

    #[benchmark]
    fn batch_set_almanac(n: Linear<1, 100>) {
        let data: Vec<(u16, u8, u8, Vec<u8>)> = (0..n)
            .map(|i| (2024, 1, (i + 1) as u8, b"Data".to_vec()))
            .collect();

        #[extrinsic_call]
        _(RawOrigin::Root, data);
    }

    #[benchmark]
    fn configure_ocw() {
        let config: Vec<u8> = b"OCWConfig".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, config);
    }

    #[benchmark]
    fn add_authority() {
        let account: T::AccountId = account("authority", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, account);
    }

    #[benchmark]
    fn remove_authority() {
        let account: T::AccountId = account("authority", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, account);
    }

    #[benchmark]
    fn remove_almanac() {
        let year: u16 = 2024;
        let month: u8 = 1;
        let day: u8 = 1;

        #[extrinsic_call]
        _(RawOrigin::Root, year, month, day);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
