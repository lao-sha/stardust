//! # Trading Pricing Pallet Benchmarking
//!
//! 定价模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_cold_start_params() {
        let target_rate: u128 = 7_000_000;
        let min_rate: u128 = 5_000_000;
        let max_rate: u128 = 10_000_000;

        #[extrinsic_call]
        _(RawOrigin::Root, target_rate, min_rate, max_rate);
    }

    #[benchmark]
    fn reset_cold_start() {
        let reason: Vec<u8> = b"Reset".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, reason);
    }

    #[benchmark]
    fn ocw_submit_exchange_rate() {
        let rate: u128 = 7_000_000;

        #[extrinsic_call]
        _(RawOrigin::Root, rate);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
