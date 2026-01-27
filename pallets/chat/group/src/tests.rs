//! 群聊模块单元测试

use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

// 类型别名
type ChatGroup = crate::Pallet<Test>;
type RuntimeOrigin = <Test as frame_system::Config>::RuntimeOrigin;

// ============ 基础群组功能测试 ============

#[test]
fn create_group_works() {
    new_test_ext().execute_with(|| {
        // 给 ALICE 一些余额用于保证金
        let _ = Balances::make_free_balance_be(&ALICE, 100_000_000_000_000_000_000);
        
        // 创建群组
        assert_ok!(ChatGroup::create_group(
            RuntimeOrigin::signed(ALICE),
            b"Test Group".to_vec(),
            Some(b"Test Description".to_vec()),
            1, // Business encryption
            true,
        ));

        // 验证事件
        let events = System::events();
        assert!(!events.is_empty());
    });
}

#[test]
fn create_group_fails_without_balance() {
    new_test_ext().execute_with(|| {
        // ALICE 没有余额
        assert_noop!(
            ChatGroup::create_group(
                RuntimeOrigin::signed(ALICE),
                b"Test Group".to_vec(),
                None,
                0,
                true,
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn create_group_fails_with_long_name() {
    new_test_ext().execute_with(|| {
        let _ = Balances::make_free_balance_be(&ALICE, 100_000_000_000_000_000_000);
        
        // 名称超过 64 字节
        let long_name = vec![b'A'; 65];
        
        assert_noop!(
            ChatGroup::create_group(
                RuntimeOrigin::signed(ALICE),
                long_name,
                None,
                0,
                true,
            ),
            Error::<Test>::GroupNameTooLong
        );
    });
}

#[test]
fn calculate_deposit_returns_fallback() {
    new_test_ext().execute_with(|| {
        // 使用空实现的 DepositCalculator，应返回兜底值
        let deposit = ChatGroup::calculate_deposit_amount();
        assert_eq!(deposit, 50_000_000_000_000_000_000u128); // 50 DUST
    });
}
