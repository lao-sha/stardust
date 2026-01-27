//! # 用户资料模块测试

use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use pallet_matchmaking_common::{Gender, ProfilePrivacyMode};

#[test]
fn create_profile_works() {
    new_test_ext().execute_with(|| {
        let nickname = b"Alice".to_vec().try_into().unwrap();
        
        assert_ok!(Profile::create_profile(
            RuntimeOrigin::signed(1),
            nickname,
            Gender::Female,
            None,
            None,
            None,
        ));

        // 验证资料已创建
        assert!(Profile::profiles(1).is_some());
        assert_eq!(Profile::profile_count(), 1);
    });
}

#[test]
fn cannot_create_duplicate_profile() {
    new_test_ext().execute_with(|| {
        let nickname = b"Alice".to_vec().try_into().unwrap();
        
        assert_ok!(Profile::create_profile(
            RuntimeOrigin::signed(1),
            nickname.clone(),
            Gender::Female,
            None,
            None,
            None,
        ));

        // 不能重复创建
        assert_noop!(
            Profile::create_profile(
                RuntimeOrigin::signed(1),
                nickname,
                Gender::Female,
                None,
                None,
                None,
            ),
            Error::<Test>::ProfileAlreadyExists
        );
    });
}

#[test]
fn update_profile_works() {
    new_test_ext().execute_with(|| {
        let nickname = b"Alice".to_vec().try_into().unwrap();
        
        assert_ok!(Profile::create_profile(
            RuntimeOrigin::signed(1),
            nickname,
            Gender::Female,
            None,
            None,
            None,
        ));

        let new_nickname = b"Alice2".to_vec().try_into().unwrap();
        assert_ok!(Profile::update_profile(
            RuntimeOrigin::signed(1),
            Some(new_nickname),
            None,
            None,
            None,
        ));

        let profile = Profile::profiles(1).unwrap();
        assert_eq!(profile.nickname.to_vec(), b"Alice2".to_vec());
    });
}

#[test]
fn update_preferences_works() {
    new_test_ext().execute_with(|| {
        let nickname = b"Alice".to_vec().try_into().unwrap();
        
        assert_ok!(Profile::create_profile(
            RuntimeOrigin::signed(1),
            nickname,
            Gender::Female,
            None,
            None,
            None,
        ));

        assert_ok!(Profile::update_preferences(
            RuntimeOrigin::signed(1),
            (25, 35),
            None,
            None,
            None,
        ));

        let profile = Profile::profiles(1).unwrap();
        assert!(profile.partner_preferences.is_some());
        assert_eq!(profile.partner_preferences.unwrap().age_range, (25, 35));
    });
}

#[test]
fn invalid_age_range_fails() {
    new_test_ext().execute_with(|| {
        let nickname = b"Alice".to_vec().try_into().unwrap();
        
        assert_ok!(Profile::create_profile(
            RuntimeOrigin::signed(1),
            nickname,
            Gender::Female,
            None,
            None,
            None,
        ));

        // 最小年龄大于最大年龄
        assert_noop!(
            Profile::update_preferences(
                RuntimeOrigin::signed(1),
                (35, 25),
                None,
                None,
                None,
            ),
            Error::<Test>::InvalidAgeRange
        );

        // 年龄小于18
        assert_noop!(
            Profile::update_preferences(
                RuntimeOrigin::signed(1),
                (16, 25),
                None,
                None,
                None,
            ),
            Error::<Test>::InvalidAgeRange
        );
    });
}

#[test]
fn link_bazi_works() {
    new_test_ext().execute_with(|| {
        let nickname = b"Alice".to_vec().try_into().unwrap();
        
        assert_ok!(Profile::create_profile(
            RuntimeOrigin::signed(1),
            nickname,
            Gender::Female,
            None,
            None,
            None,
        ));

        assert_ok!(Profile::link_bazi(RuntimeOrigin::signed(1), 123));

        let profile = Profile::profiles(1).unwrap();
        assert_eq!(profile.bazi_chart_id, Some(123));
    });
}

#[test]
fn update_privacy_mode_works() {
    new_test_ext().execute_with(|| {
        let nickname = b"Alice".to_vec().try_into().unwrap();
        
        assert_ok!(Profile::create_profile(
            RuntimeOrigin::signed(1),
            nickname,
            Gender::Female,
            None,
            None,
            None,
        ));

        assert_ok!(Profile::update_privacy_mode(
            RuntimeOrigin::signed(1),
            ProfilePrivacyMode::MatchOnly,
        ));

        let profile = Profile::profiles(1).unwrap();
        assert_eq!(profile.privacy_mode, ProfilePrivacyMode::MatchOnly);
    });
}

#[test]
fn delete_profile_works() {
    new_test_ext().execute_with(|| {
        let nickname = b"Alice".to_vec().try_into().unwrap();
        
        assert_ok!(Profile::create_profile(
            RuntimeOrigin::signed(1),
            nickname,
            Gender::Female,
            None,
            None,
            None,
        ));

        assert_eq!(Profile::profile_count(), 1);

        assert_ok!(Profile::delete_profile(RuntimeOrigin::signed(1)));

        assert!(Profile::profiles(1).is_none());
        assert_eq!(Profile::profile_count(), 0);
    });
}
