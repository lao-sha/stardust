//! # Divination Market Pallet Benchmarking
//!
//! 占卜市场模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_provider() {
        let caller: T::AccountId = whitelisted_caller();
        let name: Vec<u8> = b"Provider".to_vec();
        let bio: Vec<u8> = b"Bio".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), name, bio, 0, 0);
    }

    #[benchmark]
    fn update_provider() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), None, None, None, None);
    }

    #[benchmark]
    fn pause_provider() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn resume_provider() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn top_up_deposit() {
        let caller: T::AccountId = whitelisted_caller();
        let amount: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), amount);
    }

    #[benchmark]
    fn deactivate_provider() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn create_package() {
        let caller: T::AccountId = whitelisted_caller();
        let name: Vec<u8> = b"Package".to_vec();
        let description: Vec<u8> = b"Desc".to_vec();
        let price: BalanceOf<T> = 100u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), name, description, price, 0, 3, 1);
    }

    #[benchmark]
    fn update_package() {
        let caller: T::AccountId = whitelisted_caller();
        let package_id: u32 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), package_id, None, None, None, None);
    }

    #[benchmark]
    fn remove_package() {
        let caller: T::AccountId = whitelisted_caller();
        let package_id: u32 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), package_id);
    }

    #[benchmark]
    fn create_order() {
        let caller: T::AccountId = whitelisted_caller();
        let provider: T::AccountId = account("provider", 1, 0);
        let package_id: u32 = 1;
        let question: Vec<u8> = b"Question".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), provider, package_id, question, None, None);
    }

    #[benchmark]
    fn accept_order() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id);
    }

    #[benchmark]
    fn reject_order() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id);
    }

    #[benchmark]
    fn submit_interpretation() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;
        let content: Vec<u8> = b"Interpretation".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id, content, vec![]);
    }

    #[benchmark]
    fn submit_follow_up() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;
        let question: Vec<u8> = b"Follow up".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id, question);
    }

    #[benchmark]
    fn reply_follow_up() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;
        let follow_up_index: u32 = 0;
        let reply: Vec<u8> = b"Reply".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id, follow_up_index, reply);
    }

    #[benchmark]
    fn submit_review() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;
        let rating: u8 = 5;
        let comment: Vec<u8> = b"Great".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id, rating, comment);
    }

    #[benchmark]
    fn reply_review() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;
        let reply: Vec<u8> = b"Thanks".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id, reply);
    }

    #[benchmark]
    fn request_withdrawal() {
        let caller: T::AccountId = whitelisted_caller();
        let amount: BalanceOf<T> = 100u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), amount);
    }

    #[benchmark]
    fn cancel_order() {
        let caller: T::AccountId = whitelisted_caller();
        let order_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id);
    }

    #[benchmark]
    fn create_bounty() {
        let caller: T::AccountId = whitelisted_caller();
        let title: Vec<u8> = b"Bounty".to_vec();
        let description: Vec<u8> = b"Desc".to_vec();
        let reward: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), title, description, reward, 0, None, false, true);
    }

    #[benchmark]
    fn submit_bounty_answer() {
        let caller: T::AccountId = whitelisted_caller();
        let bounty_id: u64 = 1;
        let answer: Vec<u8> = b"Answer".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), bounty_id, answer);
    }

    #[benchmark]
    fn close_bounty() {
        let caller: T::AccountId = whitelisted_caller();
        let bounty_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), bounty_id);
    }

    #[benchmark]
    fn vote_bounty_answer() {
        let caller: T::AccountId = whitelisted_caller();
        let bounty_id: u64 = 1;
        let answer_index: u32 = 0;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), bounty_id, answer_index);
    }

    #[benchmark]
    fn adopt_bounty_answers() {
        let caller: T::AccountId = whitelisted_caller();
        let bounty_id: u64 = 1;
        let winners: Vec<u32> = vec![0];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), bounty_id, winners);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
