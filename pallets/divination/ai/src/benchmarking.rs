//! # Divination AI Pallet Benchmarking
//!
//! AI 占卜解读模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn request_interpretation() {
        let caller: T::AccountId = whitelisted_caller();
        let divination_type: u8 = 1;
        let result_id: u64 = 1;
        let interpretation_type: u8 = 0;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), divination_type, result_id, interpretation_type, None);
    }

    #[benchmark]
    fn accept_request() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id);
    }

    #[benchmark]
    fn submit_result() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;
        let result_cid: Vec<u8> = b"QmResult".to_vec();
        let confidence: u8 = 80;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id, result_cid, confidence);
    }

    #[benchmark]
    fn report_failure() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;
        let reason: Vec<u8> = b"Failed".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id, reason);
    }

    #[benchmark]
    fn rate_result() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;
        let rating: u8 = 5;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id, rating);
    }

    #[benchmark]
    fn register_oracle() {
        let caller: T::AccountId = whitelisted_caller();
        let name: Vec<u8> = b"Oracle1".to_vec();
        let endpoint: Vec<u8> = b"https://oracle.example.com".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), name, endpoint, vec![]);
    }

    #[benchmark]
    fn unregister_oracle() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn pause_oracle() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn resume_oracle() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn create_dispute() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;
        let reason: Vec<u8> = b"Dispute reason".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id, reason);
    }

    #[benchmark]
    fn resolve_dispute() {
        let request_id: u64 = 1;
        let in_favor_of_user: bool = true;
        let reason: Vec<u8> = b"Resolution".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, request_id, in_favor_of_user, reason);
    }

    #[benchmark]
    fn update_fee_distribution() {
        let burn_percent: u8 = 5;
        let treasury_percent: u8 = 10;

        #[extrinsic_call]
        _(RawOrigin::Root, burn_percent, treasury_percent);
    }

    #[benchmark]
    fn cancel_request() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
