//! # Chat Permission Pallet Benchmarking
//!
//! 聊天权限模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_permission_level() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), PermissionLevel::FriendsOnly);
    }

    #[benchmark]
    fn set_rejected_scene_types() {
        let caller: T::AccountId = whitelisted_caller();
        let scene_types: Vec<SceneType> = vec![SceneType::Order];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), scene_types);
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

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target);
    }

    #[benchmark]
    fn add_friend() {
        let caller: T::AccountId = whitelisted_caller();
        let friend: T::AccountId = account("friend", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), friend);
    }

    #[benchmark]
    fn remove_friend() {
        let caller: T::AccountId = whitelisted_caller();
        let friend: T::AccountId = account("friend", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), friend);
    }

    #[benchmark]
    fn add_to_whitelist() {
        let caller: T::AccountId = whitelisted_caller();
        let user: T::AccountId = account("user", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), user);
    }

    #[benchmark]
    fn remove_from_whitelist() {
        let caller: T::AccountId = whitelisted_caller();
        let user: T::AccountId = account("user", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), user);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
