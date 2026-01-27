//! # Divination OCW-TEE Pallet Benchmarking
//!
//! OCW-TEE 模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_public_request() {
        let caller: T::AccountId = whitelisted_caller();
        let divination_type: u8 = 0;
        let input_data: Vec<u8> = vec![0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), divination_type, input_data);
    }

    #[benchmark]
    fn create_encrypted_request() {
        let caller: T::AccountId = whitelisted_caller();
        let divination_type: u8 = 0;
        let encrypted_input: Vec<u8> = vec![0u8; 64];
        let user_pubkey: Vec<u8> = vec![0u8; 32];
        let privacy_mode: u8 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), divination_type, encrypted_input, user_pubkey, privacy_mode);
    }

    #[benchmark]
    fn submit_result() {
        let request_id: u64 = 1;
        let manifest_cid: Vec<u8> = b"QmManifest".to_vec();
        let manifest_hash: [u8; 32] = [0u8; 32];
        let type_index: Vec<u8> = vec![0u8; 8];
        let proof: Vec<u8> = vec![];

        #[extrinsic_call]
        _(RawOrigin::Root, request_id, manifest_cid, manifest_hash, type_index, proof);
    }

    #[benchmark]
    fn update_request_status() {
        let request_id: u64 = 1;
        let status: u8 = 1;

        #[extrinsic_call]
        _(RawOrigin::Root, request_id, status);
    }

    #[benchmark]
    fn mark_request_failed() {
        let request_id: u64 = 1;
        let reason: Vec<u8> = b"Failed".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, request_id, reason);
    }

    #[benchmark]
    fn cancel_request() {
        let caller: T::AccountId = whitelisted_caller();
        let request_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), request_id);
    }

    #[benchmark]
    fn update_divination() {
        let caller: T::AccountId = whitelisted_caller();
        let original_id: u64 = 1;
        let new_input: Vec<u8> = vec![1u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), original_id, new_input);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
