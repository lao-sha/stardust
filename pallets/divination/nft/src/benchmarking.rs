//! # Divination NFT Pallet Benchmarking
//!
//! 占卜 NFT 模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn mint_nft() {
        let caller: T::AccountId = whitelisted_caller();
        let name: Vec<u8> = b"NFT".to_vec();
        let image_cid: Vec<u8> = b"QmImage".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0, 1, name, image_cid, None, None, 500);
    }

    #[benchmark]
    fn transfer_nft() {
        let caller: T::AccountId = whitelisted_caller();
        let to: T::AccountId = account("receiver", 1, 0);
        let nft_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), nft_id, to);
    }

    #[benchmark]
    fn burn_nft() {
        let caller: T::AccountId = whitelisted_caller();
        let nft_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), nft_id);
    }

    #[benchmark]
    fn list_nft() {
        let caller: T::AccountId = whitelisted_caller();
        let nft_id: u64 = 1;
        let price: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), nft_id, price);
    }

    #[benchmark]
    fn cancel_listing() {
        let caller: T::AccountId = whitelisted_caller();
        let nft_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), nft_id);
    }

    #[benchmark]
    fn buy_nft() {
        let caller: T::AccountId = whitelisted_caller();
        let nft_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), nft_id);
    }

    #[benchmark]
    fn make_offer() {
        let caller: T::AccountId = whitelisted_caller();
        let nft_id: u64 = 1;
        let price: BalanceOf<T> = 500u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), nft_id, price);
    }

    #[benchmark]
    fn cancel_offer() {
        let caller: T::AccountId = whitelisted_caller();
        let offer_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), offer_id);
    }

    #[benchmark]
    fn accept_offer() {
        let caller: T::AccountId = whitelisted_caller();
        let offer_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), offer_id);
    }

    #[benchmark]
    fn create_collection() {
        let caller: T::AccountId = whitelisted_caller();
        let name: Vec<u8> = b"Collection".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), name, None, None);
    }

    #[benchmark]
    fn add_to_collection() {
        let caller: T::AccountId = whitelisted_caller();
        let collection_id: u64 = 1;
        let nft_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), collection_id, nft_id);
    }

    #[benchmark]
    fn remove_from_collection() {
        let caller: T::AccountId = whitelisted_caller();
        let collection_id: u64 = 1;
        let nft_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), collection_id, nft_id);
    }

    #[benchmark]
    fn delete_collection() {
        let caller: T::AccountId = whitelisted_caller();
        let collection_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), collection_id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
