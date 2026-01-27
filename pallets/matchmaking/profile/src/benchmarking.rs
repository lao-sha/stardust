//! # Matchmaking Profile Pallet Benchmarking
//!
//! 婚恋档案模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_profile() {
        let caller: T::AccountId = whitelisted_caller();
        let nickname: Vec<u8> = b"TestUser".to_vec();
        let bio: Vec<u8> = b"Hello world".to_vec();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            nickname,
            Gender::Male,
            1990,
            1,
            1,
            bio,
            None,
            None
        );
    }

    #[benchmark]
    fn update_profile() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            Some(b"NewNick".to_vec()),
            Some(b"New bio".to_vec()),
            None,
            None
        );
    }

    #[benchmark]
    fn update_preferences() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            None,
            None,
            None,
            None,
            None,
            None,
            None
        );
    }

    #[benchmark]
    fn link_bazi() {
        let caller: T::AccountId = whitelisted_caller();
        let bazi_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), bazi_id);
    }

    #[benchmark]
    fn update_privacy_mode() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), PrivacyMode::Public);
    }

    #[benchmark]
    fn delete_profile() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn pay_monthly_fee() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), None);
    }

    #[benchmark]
    fn update_user_personality() {
        let caller: T::AccountId = whitelisted_caller();
        let traits: Vec<PersonalityTrait> = vec![];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), traits);
    }

    #[benchmark]
    fn sync_bazi_personality() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn upload_photo() {
        let caller: T::AccountId = whitelisted_caller();
        let cid: Vec<u8> = b"QmPhoto".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), cid, false);
    }

    #[benchmark]
    fn upload_photos_batch() {
        let caller: T::AccountId = whitelisted_caller();
        let cids: Vec<Vec<u8>> = vec![b"QmPhoto1".to_vec()];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), cids);
    }

    #[benchmark]
    fn handle_violation() {
        let user: T::AccountId = account("user", 1, 0);
        let violation_type: u8 = 1;
        let reason: Vec<u8> = b"Violation".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, user, violation_type, reason);
    }

    #[benchmark]
    fn top_up_deposit() {
        let caller: T::AccountId = whitelisted_caller();
        let amount: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), amount);
    }

    #[benchmark]
    fn lift_suspension() {
        let user: T::AccountId = account("user", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, user);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
