//! # Divination Privacy Pallet Benchmarking
//!
//! 占卜隐私模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_encryption_key() {
        let caller: T::AccountId = whitelisted_caller();
        let public_key: Vec<u8> = vec![0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), public_key);
    }

    #[benchmark]
    fn update_encryption_key() {
        let caller: T::AccountId = whitelisted_caller();
        let new_public_key: Vec<u8> = vec![1u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), new_public_key);
    }

    #[benchmark]
    fn register_provider() {
        let caller: T::AccountId = whitelisted_caller();
        let public_key: Vec<u8> = vec![0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, public_key);
    }

    #[benchmark]
    fn update_provider_key() {
        let caller: T::AccountId = whitelisted_caller();
        let new_public_key: Vec<u8> = vec![1u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), new_public_key);
    }

    #[benchmark]
    fn set_provider_active() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), true);
    }

    #[benchmark]
    fn unregister_provider() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn create_encrypted_record() {
        let caller: T::AccountId = whitelisted_caller();
        let record_id: [u8; 32] = [0u8; 32];
        let encrypted_data: Vec<u8> = vec![0u8; 64];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), record_id, 0, 0, encrypted_data, None);
    }

    #[benchmark]
    fn update_encrypted_record() {
        let caller: T::AccountId = whitelisted_caller();
        let record_id: [u8; 32] = [0u8; 32];
        let encrypted_data: Vec<u8> = vec![1u8; 64];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), record_id, encrypted_data, None);
    }

    #[benchmark]
    fn change_privacy_mode() {
        let caller: T::AccountId = whitelisted_caller();
        let record_id: [u8; 32] = [0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), record_id, 1);
    }

    #[benchmark]
    fn delete_encrypted_record() {
        let caller: T::AccountId = whitelisted_caller();
        let record_id: [u8; 32] = [0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), record_id);
    }

    #[benchmark]
    fn grant_access() {
        let caller: T::AccountId = whitelisted_caller();
        let grantee: T::AccountId = account("grantee", 1, 0);
        let record_id: [u8; 32] = [0u8; 32];
        let encrypted_key: Vec<u8> = vec![0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), record_id, grantee, encrypted_key, 0, None);
    }

    #[benchmark]
    fn revoke_access() {
        let caller: T::AccountId = whitelisted_caller();
        let grantee: T::AccountId = account("grantee", 1, 0);
        let record_id: [u8; 32] = [0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), record_id, grantee);
    }

    #[benchmark]
    fn revoke_all_access() {
        let caller: T::AccountId = whitelisted_caller();
        let record_id: [u8; 32] = [0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), record_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
