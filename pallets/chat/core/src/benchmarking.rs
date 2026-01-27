//! # Chat Core Pallet Benchmarking
//!
//! 聊天核心模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::BoundedVec;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn send_message() {
        let caller: T::AccountId = whitelisted_caller();
        let receiver: T::AccountId = account("receiver", 1, 0);
        let content_cid: BoundedVec<u8, T::MaxCidLen> = 
            b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec().try_into().unwrap();

        #[extrinsic_call]
        send_message(
            RawOrigin::Signed(caller),
            receiver,
            content_cid,
            MessageType::Text,
            None
        );
    }

    #[benchmark]
    fn mark_as_read() {
        let caller: T::AccountId = whitelisted_caller();
        // 假设消息ID 1 存在
        let message_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), message_id);
    }

    #[benchmark]
    fn delete_message() {
        let caller: T::AccountId = whitelisted_caller();
        let message_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), message_id);
    }

    #[benchmark]
    fn mark_batch_as_read(n: Linear<1, 100>) {
        let caller: T::AccountId = whitelisted_caller();
        let message_ids: Vec<u64> = (1..=n as u64).collect();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), message_ids);
    }

    #[benchmark]
    fn mark_session_as_read(n: Linear<1, 100>) {
        let caller: T::AccountId = whitelisted_caller();
        let session_id = T::Hash::default();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), session_id);
    }

    #[benchmark]
    fn archive_session() {
        let caller: T::AccountId = whitelisted_caller();
        let session_id = T::Hash::default();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), session_id);
    }

    #[benchmark]
    fn block_user() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn unblock_user() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 1, 0);
        
        // 先拉黑
        Blacklist::<T>::insert(&caller, &target, ());

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn cleanup_old_messages(n: Linear<1, 1000>) {
        #[extrinsic_call]
        _(RawOrigin::Root, n);
    }

    #[benchmark]
    fn register_chat_user() {
        let caller: T::AccountId = whitelisted_caller();
        let nickname: Option<Vec<u8>> = Some(b"TestUser".to_vec());

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), nickname);
    }

    #[benchmark]
    fn update_chat_profile() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            Some(b"NewNick".to_vec()),
            None,
            None
        );
    }

    #[benchmark]
    fn set_user_status() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), UserStatus::Online);
    }

    #[benchmark]
    fn update_privacy_settings() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            Some(true),
            Some(true),
            Some(true)
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
