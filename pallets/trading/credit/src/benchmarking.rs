//! # Trading Credit Pallet Benchmarking
//!
//! 交易信用模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn endorse_user() {
        let caller: T::AccountId = whitelisted_caller();
        let new_user: T::AccountId = account("new_user", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), new_user);
    }

    #[benchmark]
    fn set_referrer() {
        let caller: T::AccountId = whitelisted_caller();
        let referrer: T::AccountId = account("referrer", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), referrer);
    }

    #[benchmark]
    fn rate_maker() {
        let caller: T::AccountId = whitelisted_caller();
        let maker: T::AccountId = account("maker", 1, 0);
        let order_id: u64 = 1;
        let rating: u8 = 5;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), maker, order_id, rating);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
