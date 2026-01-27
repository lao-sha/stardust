//! # Affiliate Pallet Tests
//!
//! 函数级详细中文注释：Affiliate Pallet 完整测试套件

use crate::{mock::*, Error, pallet::*};
use frame_support::{assert_noop, assert_ok, BoundedVec};

// ========================================
// 基础功能测试
// ========================================

#[test]
fn test_new_test_ext_setup() {
    new_test_ext().execute_with(|| {
        assert_eq!(System::block_number(), 1);
        assert_eq!(balance_of(1), 10_000_000_000_000_000);
        assert_eq!(balance_of(999), 1_000_000_000_000_000);
    });
}

#[test]
fn test_run_to_block() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(System::block_number(), 10);
    });
}

#[test]
fn test_membership_provider() {
    new_test_ext().execute_with(|| {
        use crate::MembershipProvider;
        assert!(MockMembershipProvider::is_valid_member(&1));
        assert!(MockMembershipProvider::is_valid_member(&900));
        assert!(!MockMembershipProvider::is_valid_member(&901));
    });
}

// ========================================
// 推荐关系测试（委托到 referral pallet）
// ========================================

#[test]
fn test_claim_code_success() {
    new_test_ext().execute_with(|| {
        // Alice 认领推荐码
        assert_ok!(Affiliate::claim_code(
            RuntimeOrigin::signed(1),
            b"ALICE001".to_vec()
        ));

        // 验证推荐码已关联
        let code: BoundedVec<u8, MaxCodeLen> = b"ALICE001".to_vec().try_into().unwrap();
        assert_eq!(Referral::account_by_code(&code), Some(1));
    });
}

#[test]
fn test_claim_code_not_member() {
    new_test_ext().execute_with(|| {
        // 非会员无法认领
        assert_noop!(
            Affiliate::claim_code(RuntimeOrigin::signed(901), b"CODE0901".to_vec()),
            pallet_affiliate_referral::Error::<Test>::NotMember
        );
    });
}

#[test]
fn test_bind_sponsor_success() {
    new_test_ext().execute_with(|| {
        // Bob 先认领推荐码
        assert_ok!(Affiliate::claim_code(
            RuntimeOrigin::signed(2),
            b"BOBCODE1".to_vec()
        ));

        // Alice 绑定 Bob 为推荐人
        assert_ok!(Affiliate::bind_sponsor(
            RuntimeOrigin::signed(1),
            b"BOBCODE1".to_vec()
        ));

        // 验证推荐关系
        assert_eq!(Referral::sponsor_of(1), Some(2));
    });
}

#[test]
fn test_bind_sponsor_code_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Affiliate::bind_sponsor(RuntimeOrigin::signed(1), b"NOTEXIST".to_vec()),
            pallet_affiliate_referral::Error::<Test>::CodeNotFound
        );
    });
}

#[test]
fn test_bind_sponsor_already_bound() {
    new_test_ext().execute_with(|| {
        // 设置推荐码
        assert_ok!(Affiliate::claim_code(RuntimeOrigin::signed(2), b"BOBCODE1".to_vec()));
        assert_ok!(Affiliate::claim_code(RuntimeOrigin::signed(3), b"CHARLIE1".to_vec()));

        // Alice 绑定 Bob
        assert_ok!(Affiliate::bind_sponsor(RuntimeOrigin::signed(1), b"BOBCODE1".to_vec()));

        // Alice 尝试再次绑定
        assert_noop!(
            Affiliate::bind_sponsor(RuntimeOrigin::signed(1), b"CHARLIE1".to_vec()),
            pallet_affiliate_referral::Error::<Test>::AlreadyBound
        );
    });
}

// ========================================
// 配置管理测试
// ========================================

#[test]
fn test_set_settlement_mode_weekly() {
    new_test_ext().execute_with(|| {
        // Root 设置为周结算模式
        assert_ok!(Affiliate::set_settlement_mode(
            RuntimeOrigin::root(),
            0, // Weekly
            0,
            0
        ));

        assert_eq!(
            Affiliate::settlement_mode(),
            crate::types::SettlementMode::Weekly
        );
    });
}

#[test]
fn test_set_settlement_mode_instant() {
    new_test_ext().execute_with(|| {
        assert_ok!(Affiliate::set_settlement_mode(
            RuntimeOrigin::root(),
            1, // Instant
            0,
            0
        ));

        assert_eq!(
            Affiliate::settlement_mode(),
            crate::types::SettlementMode::Instant
        );
    });
}

#[test]
fn test_set_settlement_mode_hybrid() {
    new_test_ext().execute_with(|| {
        assert_ok!(Affiliate::set_settlement_mode(
            RuntimeOrigin::root(),
            2, // Hybrid
            5, // instant_levels
            10 // weekly_levels
        ));

        assert_eq!(
            Affiliate::settlement_mode(),
            crate::types::SettlementMode::Hybrid {
                instant_levels: 5,
                weekly_levels: 10,
            }
        );
    });
}

#[test]
fn test_set_settlement_mode_hybrid_too_many_levels() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Affiliate::set_settlement_mode(
                RuntimeOrigin::root(),
                2, // Hybrid
                10,
                10 // 总共 20 层超限
            ),
            Error::<Test>::HybridLevelsTooMany
        );
    });
}

#[test]
fn test_set_settlement_mode_invalid() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Affiliate::set_settlement_mode(RuntimeOrigin::root(), 99, 0, 0),
            Error::<Test>::InvalidMode
        );
    });
}

#[test]
fn test_set_weekly_percents_success() {
    new_test_ext().execute_with(|| {
        let new_percents = vec![25, 15, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5];
        assert_ok!(Affiliate::set_weekly_percents(
            RuntimeOrigin::root(),
            new_percents.clone()
        ));

        let stored: Vec<u8> = Affiliate::weekly_percents().into_inner();
        assert_eq!(stored, new_percents);
    });
}

#[test]
fn test_set_weekly_percents_invalid_length() {
    new_test_ext().execute_with(|| {
        let invalid_percents = vec![20, 10, 5]; // 只有 3 个
        assert_noop!(
            Affiliate::set_weekly_percents(RuntimeOrigin::root(), invalid_percents),
            Error::<Test>::InvalidPercents
        );
    });
}

#[test]
fn test_set_blocks_per_week() {
    new_test_ext().execute_with(|| {
        assert_ok!(Affiliate::set_blocks_per_week(RuntimeOrigin::root(), 50400));
        assert_eq!(Affiliate::blocks_per_week(), 50400);
    });
}

// ========================================
// 托管测试
// ========================================

#[test]
fn test_escrow_account_exists() {
    new_test_ext().execute_with(|| {
        let escrow = Affiliate::escrow_account();
        // 托管账户应该存在
        assert!(escrow != 0u64);
    });
}

#[test]
fn test_escrow_balance_initial() {
    new_test_ext().execute_with(|| {
        // 初始托管余额为 0
        assert_eq!(Affiliate::escrow_balance(), 0);
    });
}

// ========================================
// 推荐链测试
// ========================================

#[test]
fn test_get_referral_chain_empty() {
    new_test_ext().execute_with(|| {
        let chain = Affiliate::get_referral_chain(&1);
        assert!(chain.is_empty());
    });
}

#[test]
fn test_get_referral_chain_single_level() {
    new_test_ext().execute_with(|| {
        // 设置 Alice → Bob
        assert_ok!(Affiliate::claim_code(RuntimeOrigin::signed(2), b"BOBCODE1".to_vec()));
        assert_ok!(Affiliate::bind_sponsor(RuntimeOrigin::signed(1), b"BOBCODE1".to_vec()));

        let chain = Affiliate::get_referral_chain(&1);
        assert_eq!(chain, vec![2]);
    });
}

#[test]
fn test_get_referral_chain_multi_level() {
    new_test_ext().execute_with(|| {
        // 设置推荐链：1 → 2 → 3 → 4
        assert_ok!(Affiliate::claim_code(RuntimeOrigin::signed(2), b"BOBCODE1".to_vec()));
        assert_ok!(Affiliate::claim_code(RuntimeOrigin::signed(3), b"CHARLIE1".to_vec()));
        assert_ok!(Affiliate::claim_code(RuntimeOrigin::signed(4), b"DAVEXXXX".to_vec()));

        assert_ok!(Affiliate::bind_sponsor(RuntimeOrigin::signed(1), b"BOBCODE1".to_vec()));
        assert_ok!(Affiliate::bind_sponsor(RuntimeOrigin::signed(2), b"CHARLIE1".to_vec()));
        assert_ok!(Affiliate::bind_sponsor(RuntimeOrigin::signed(3), b"DAVEXXXX".to_vec()));

        let chain = Affiliate::get_referral_chain(&1);
        assert_eq!(chain, vec![2, 3, 4]);
    });
}

// ========================================
// 治理测试
// ========================================

#[test]
fn test_governance_paused_initial() {
    new_test_ext().execute_with(|| {
        assert!(!GovernancePaused::<Test>::get());
    });
}

#[test]
fn test_emergency_pause_governance() {
    new_test_ext().execute_with(|| {
        let reason: BoundedVec<u8, frame_support::traits::ConstU32<64>> = 
            b"Security issue".to_vec().try_into().unwrap();
        
        assert_ok!(Affiliate::emergency_pause_governance(
            RuntimeOrigin::root(),
            reason
        ));

        assert!(GovernancePaused::<Test>::get());
    });
}

#[test]
fn test_resume_governance() {
    new_test_ext().execute_with(|| {
        // 先暂停
        let reason: BoundedVec<u8, frame_support::traits::ConstU32<64>> = 
            b"Test".to_vec().try_into().unwrap();
        assert_ok!(Affiliate::emergency_pause_governance(RuntimeOrigin::root(), reason));
        
        // 恢复
        assert_ok!(Affiliate::resume_governance(RuntimeOrigin::root()));
        assert!(!GovernancePaused::<Test>::get());
    });
}

// ========================================
// 默认值测试
// ========================================

#[test]
fn test_default_instant_percents() {
    new_test_ext().execute_with(|| {
        let percents: Vec<u8> = Affiliate::instant_percents().into_inner();
        // 默认即时分成比例：30, 25, 15, 10, 7, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1
        assert_eq!(percents.len(), 15);
        assert_eq!(percents[0], 30); // L1
        assert_eq!(percents[1], 25); // L2
        assert_eq!(percents[2], 15); // L3
    });
}

#[test]
fn test_default_weekly_percents() {
    new_test_ext().execute_with(|| {
        let percents: Vec<u8> = Affiliate::weekly_percents().into_inner();
        // 默认周结算比例：20, 10, 4, 4, 4, ...
        assert_eq!(percents.len(), 15);
        assert_eq!(percents[0], 20); // L1
        assert_eq!(percents[1], 10); // L2
        assert_eq!(percents[2], 4);  // L3
    });
}

#[test]
fn test_default_blocks_per_week() {
    new_test_ext().execute_with(|| {
        // 默认每周区块数：100800（6秒出块，1周）
        assert_eq!(Affiliate::blocks_per_week(), 100800);
    });
}
