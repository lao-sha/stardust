//! # Matchmaking Matching Pallet Benchmarking
//!
//! 合婚匹配模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_request() {
        let caller: T::AccountId = whitelisted_caller();
        let party_b: T::AccountId = account("party_b", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), party_b);
    }

    #[benchmark]
    fn authorize_request() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id);
    }

    #[benchmark]
    fn reject_request() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id);
    }

    #[benchmark]
    fn cancel_request() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id);
    }

    #[benchmark]
    fn generate_report() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
