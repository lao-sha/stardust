//! 直播间模块单元测试

use crate::{mock::*, Error, LiveRoomStatus, LiveRoomType};
use frame_support::{assert_noop, assert_ok};

// ============ 直播间创建测试 ============

#[test]
fn create_room_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            Some(b"Description".to_vec()),
            LiveRoomType::Normal,
            None,
            None,
        ));

        // 检查直播间创建
        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.host, ALICE);
        assert_eq!(room.status, LiveRoomStatus::Preparing);
        assert_eq!(room.room_type, LiveRoomType::Normal);

        // 检查主播映射
        assert_eq!(Livestream::host_room(ALICE), Some(0));
    });
}

#[test]
fn create_room_fails_if_already_has_room() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Room 1".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        assert_noop!(
            Livestream::create_room(
                RuntimeOrigin::signed(ALICE),
                b"Room 2".to_vec(),
                None,
                LiveRoomType::Normal,
                None,
                None,
            ),
            Error::<Test>::HostAlreadyHasRoom
        );
    });
}

#[test]
fn start_live_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // 创建直播间
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        // 开始直播
        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Live);
    });
}

#[test]
fn end_live_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // 创建并开始直播
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));
        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        // 结束直播
        assert_ok!(Livestream::end_live(RuntimeOrigin::signed(ALICE), 0));

        // 检查直播间状态变为 Ended
        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Ended);
    });
}

#[test]
fn calculate_room_bond_returns_fallback() {
    new_test_ext().execute_with(|| {
        // 使用空实现的 DepositCalculator，应返回兜底值
        let bond = Livestream::calculate_room_bond();
        assert_eq!(bond, 10_000_000_000_000u128); // 10 DUST
    });
}

// ============ 礼物系统测试 ============

#[test]
fn create_gift_works() {
    new_test_ext().execute_with(|| {
        // 管理员创建礼物
        assert_ok!(Livestream::create_gift(
            RuntimeOrigin::root(),
            b"Rose".to_vec(),
            1_000_000_000_000, // 1 DUST
            b"QmTestIconCid".to_vec(),
        ));

        let gift = Livestream::gifts(0).unwrap();
        assert_eq!(gift.price, 1_000_000_000_000);
        assert!(gift.enabled);
    });
}
