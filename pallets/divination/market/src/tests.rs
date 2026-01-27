//! 通用占卜服务市场 Pallet 单元测试

use crate::{mock::*, types::*, Error};
use frame_support::{assert_noop, assert_ok};
use pallet_divination_common::{DivinationType, RarityInput};

// ==================== 提供者测试 ====================

/// 测试提供者注册
#[test]
fn register_provider_works() {
    new_test_ext().execute_with(|| {
        let provider = 10u64;
        let name = b"Master Wang".to_vec();
        let bio = b"Expert in Meihua divination".to_vec();
        let specialties = 0b00001111u16; // 前4种领域
        let supported_types = 0b00000011u8; // Meihua + Bazi

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(provider),
            name,
            bio,
            specialties,
            supported_types
        ));

        // 验证提供者已注册
        let p = DivinationMarket::providers(provider).expect("Provider should exist");
        assert_eq!(p.account, provider);
        assert_eq!(p.tier, ProviderTier::Novice);
        assert_eq!(p.status, ProviderStatus::Active);
        assert_eq!(p.deposit, 10000); // MinDeposit
        assert_eq!(p.specialties, specialties);
        assert_eq!(p.supported_divination_types, supported_types);

        // 验证统计更新
        let stats = DivinationMarket::market_stats();
        assert_eq!(stats.active_providers, 1);
    });
}

/// 测试重复注册失败
#[test]
fn register_provider_already_exists_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider1".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        assert_noop!(
            DivinationMarket::register_provider(
                RuntimeOrigin::signed(10),
                b"Provider2".to_vec(),
                b"Bio2".to_vec(),
                0b00000010,
                0b00000010
            ),
            Error::<Test>::ProviderAlreadyExists
        );
    });
}

/// 测试更新提供者信息
#[test]
fn update_provider_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"OldName".to_vec(),
            b"OldBio".to_vec(),
            0b00000001,
            0b00000001
        ));

        assert_ok!(DivinationMarket::update_provider(
            RuntimeOrigin::signed(10),
            Some(b"NewName".to_vec()),
            Some(b"NewBio".to_vec()),
            None,
            Some(0b00001111),
            Some(0b00000011), // 支持更多占卜类型
            Some(true)
        ));

        let p = DivinationMarket::providers(10).unwrap();
        assert_eq!(p.name.to_vec(), b"NewName".to_vec());
        assert_eq!(p.specialties, 0b00001111);
        assert_eq!(p.supported_divination_types, 0b00000011);
        assert!(p.accepts_urgent);
    });
}

/// 测试暂停和恢复提供者
#[test]
fn pause_resume_provider_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        // 暂停
        assert_ok!(DivinationMarket::pause_provider(RuntimeOrigin::signed(10)));
        let p = DivinationMarket::providers(10).unwrap();
        assert_eq!(p.status, ProviderStatus::Paused);
        assert_eq!(DivinationMarket::market_stats().active_providers, 0);

        // 恢复
        assert_ok!(DivinationMarket::resume_provider(RuntimeOrigin::signed(10)));
        let p = DivinationMarket::providers(10).unwrap();
        assert_eq!(p.status, ProviderStatus::Active);
        assert_eq!(DivinationMarket::market_stats().active_providers, 1);
    });
}

/// 测试注销提供者
#[test]
fn deactivate_provider_works() {
    new_test_ext().execute_with(|| {
        let initial_balance = Balances::free_balance(10);

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        // 保证金被锁定
        assert_eq!(Balances::reserved_balance(10), 10000);

        assert_ok!(DivinationMarket::deactivate_provider(RuntimeOrigin::signed(
            10
        )));

        // 提供者已删除
        assert!(DivinationMarket::providers(10).is_none());
        // 保证金已退还
        assert_eq!(Balances::free_balance(10), initial_balance);
        assert_eq!(DivinationMarket::market_stats().active_providers, 0);
    });
}

// ==================== 套餐测试 ====================

/// 测试创建服务套餐
#[test]
fn create_package_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001 // 支持 Meihua
        ));

        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Basic Reading".to_vec(),
            b"Simple text-based reading".to_vec(),
            1000, // price
            0,    // duration
            2,    // follow_up_count
            true, // urgent_available
            2000  // urgent_surcharge (20%)
        ));

        let package = DivinationMarket::packages(10, 0).expect("Package should exist");
        assert_eq!(package.price, 1000);
        assert_eq!(package.follow_up_count, 2);
        assert_eq!(package.divination_type, DivinationType::Meihua);
        assert!(package.is_active);
    });
}

/// 测试价格低于最低限制
#[test]
fn create_package_price_too_low_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        assert_noop!(
            DivinationMarket::create_package(
                RuntimeOrigin::signed(10),
                DivinationType::Meihua,
                ServiceType::TextReading,
                b"Cheap".to_vec(),
                b"Too cheap".to_vec(),
                50, // price below MinServicePrice (100)
                0,
                0,
                false,
                0
            ),
            Error::<Test>::PriceTooLow
        );
    });
}

/// 测试不支持的占卜类型
#[test]
fn create_package_unsupported_type_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001 // 只支持 Meihua
        ));

        assert_noop!(
            DivinationMarket::create_package(
                RuntimeOrigin::signed(10),
                DivinationType::Bazi, // 不支持 Bazi
                ServiceType::TextReading,
                b"Bazi Package".to_vec(),
                b"Desc".to_vec(),
                1000,
                0,
                0,
                false,
                0
            ),
            Error::<Test>::DivinationTypeNotSupported
        );
    });
}

/// 测试更新套餐
#[test]
fn update_package_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            2,
            false,
            0
        ));

        assert_ok!(DivinationMarket::update_package(
            RuntimeOrigin::signed(10),
            0,
            Some(2000),
            Some(b"New description".to_vec()),
            None
        ));

        let package = DivinationMarket::packages(10, 0).unwrap();
        assert_eq!(package.price, 2000);
    });
}

// ==================== 订单测试 ====================

/// 测试创建订单
#[test]
fn create_order_works() {
    new_test_ext().execute_with(|| {
        // 添加模拟占卜结果
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        // 注册提供者和创建套餐
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            2,
            false,
            0
        ));

        let customer_initial = Balances::free_balance(1);
        let platform_initial = Balances::free_balance(999);

        // 客户下单
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10, // provider
            DivinationType::Meihua,
            1,  // result_id
            0,  // package_id
            b"QmQuestionCid".to_vec(),
            false // not urgent
        ));

        let order = DivinationMarket::orders(0).expect("Order should exist");
        assert_eq!(order.customer, 1);
        assert_eq!(order.provider, 10);
        assert_eq!(order.divination_type, DivinationType::Meihua);
        assert_eq!(order.result_id, 1);
        assert_eq!(order.amount, 1000);
        assert_eq!(order.status, OrderStatus::Paid);
        assert_eq!(order.follow_ups_remaining, 2);

        // 验证资金转移
        assert_eq!(Balances::free_balance(1), customer_initial - 1000);
        assert_eq!(Balances::free_balance(999), platform_initial + 1000);

        // 验证统计
        let stats = DivinationMarket::market_stats();
        assert_eq!(stats.total_orders, 1);
        assert_eq!(stats.total_volume, 1000);

        // 验证类型统计
        let type_stats = DivinationMarket::type_stats(DivinationType::Meihua);
        assert_eq!(type_stats.order_count, 1);
    });
}

/// 测试不能给自己下单
#[test]
fn create_order_self_fails() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 10, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));

        assert_noop!(
            DivinationMarket::create_order(
                RuntimeOrigin::signed(10), // provider ordering for self
                10,
                DivinationType::Meihua,
                1,
                0,
                b"Cid".to_vec(),
                false
            ),
            Error::<Test>::CannotOrderSelf
        );
    });
}

/// 测试占卜结果不存在
#[test]
fn create_order_result_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));

        // 没有添加模拟数据
        assert_noop!(
            DivinationMarket::create_order(
                RuntimeOrigin::signed(1),
                10,
                DivinationType::Meihua,
                999, // 不存在的结果
                0,
                b"Cid".to_vec(),
                false
            ),
            Error::<Test>::DivinationResultNotFound
        );
    });
}

/// 测试接受订单
#[test]
fn accept_order_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));

        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));

        // 接受订单
        assert_ok!(DivinationMarket::accept_order(RuntimeOrigin::signed(10), 0));

        let order = DivinationMarket::orders(0).unwrap();
        assert_eq!(order.status, OrderStatus::Accepted);
        assert!(order.accepted_at.is_some());
    });
}

/// 测试拒绝订单
#[test]
fn reject_order_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));

        let customer_initial = Balances::free_balance(1);

        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));

        // 拒绝订单
        assert_ok!(DivinationMarket::reject_order(RuntimeOrigin::signed(10), 0));

        let order = DivinationMarket::orders(0).unwrap();
        assert_eq!(order.status, OrderStatus::Cancelled);

        // 验证退款
        assert_eq!(Balances::free_balance(1), customer_initial);
    });
}

/// 测试提交解读
#[test]
fn submit_interpretation_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));
        assert_ok!(DivinationMarket::accept_order(RuntimeOrigin::signed(10), 0));

        // 提交解读
        assert_ok!(DivinationMarket::submit_interpretation(
            RuntimeOrigin::signed(10),
            0,
            b"QmAnswerCid".to_vec(),
            vec![],  // imgs
            vec![],  // vids
            vec![]   // docs
        ));

        let order = DivinationMarket::orders(0).unwrap();
        assert_eq!(order.status, OrderStatus::Completed);
        assert!(order.interpretation_cid.is_some());

        // 验证提供者收益
        let provider = DivinationMarket::providers(10).unwrap();
        assert_eq!(provider.completed_orders, 1);

        // 20% 平台费，800 给提供者
        let provider_balance = DivinationMarket::provider_balances(10);
        assert_eq!(provider_balance, 800);

        // 验证类型统计
        let type_stats = DivinationMarket::type_stats(DivinationType::Meihua);
        assert_eq!(type_stats.completed_count, 1);
    });
}

/// 测试客户取消订单
#[test]
fn cancel_order_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));

        let customer_initial = Balances::free_balance(1);

        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));

        // 取消订单（接单前）
        assert_ok!(DivinationMarket::cancel_order(RuntimeOrigin::signed(1), 0));

        let order = DivinationMarket::orders(0).unwrap();
        assert_eq!(order.status, OrderStatus::Cancelled);
        assert_eq!(Balances::free_balance(1), customer_initial);
    });
}

// ==================== 追问测试 ====================

/// 测试追问功能
#[test]
fn follow_up_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            2, // 2 follow-ups
            false,
            0
        ));
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));
        assert_ok!(DivinationMarket::accept_order(RuntimeOrigin::signed(10), 0));
        assert_ok!(DivinationMarket::submit_interpretation(
            RuntimeOrigin::signed(10),
            0,
            b"Answer".to_vec(),
            vec![],
            vec![],
            vec![]
        ));

        // 提交追问
        assert_ok!(DivinationMarket::submit_follow_up(
            RuntimeOrigin::signed(1),
            0,
            b"FollowUpQuestion".to_vec()
        ));

        let order = DivinationMarket::orders(0).unwrap();
        assert_eq!(order.follow_ups_remaining, 1);

        let follow_ups = DivinationMarket::follow_ups(0);
        assert_eq!(follow_ups.len(), 1);
        assert!(follow_ups[0].reply_cid.is_none());

        // 回复追问
        assert_ok!(DivinationMarket::reply_follow_up(
            RuntimeOrigin::signed(10),
            0,
            0,
            b"FollowUpAnswer".to_vec()
        ));

        let follow_ups = DivinationMarket::follow_ups(0);
        assert!(follow_ups[0].reply_cid.is_some());
    });
}

/// 测试追问次数用完
#[test]
fn follow_up_exhausted_fails() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0, // 0 follow-ups
            false,
            0
        ));
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));
        assert_ok!(DivinationMarket::accept_order(RuntimeOrigin::signed(10), 0));
        assert_ok!(DivinationMarket::submit_interpretation(
            RuntimeOrigin::signed(10),
            0,
            b"Answer".to_vec(),
            vec![],
            vec![],
            vec![]
        ));

        assert_noop!(
            DivinationMarket::submit_follow_up(RuntimeOrigin::signed(1), 0, b"Question".to_vec()),
            Error::<Test>::NoFollowUpsRemaining
        );
    });
}

// ==================== 评价测试 ====================

/// 测试评价功能
#[test]
fn submit_review_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));
        assert_ok!(DivinationMarket::accept_order(RuntimeOrigin::signed(10), 0));
        assert_ok!(DivinationMarket::submit_interpretation(
            RuntimeOrigin::signed(10),
            0,
            b"Answer".to_vec(),
            vec![],
            vec![],
            vec![]
        ));

        // 提交评价
        assert_ok!(DivinationMarket::submit_review(
            RuntimeOrigin::signed(1),
            0,
            5, // overall
            4, // accuracy
            5, // attitude
            4, // response
            Some(b"Great service!".to_vec()),
            false // not anonymous
        ));

        let order = DivinationMarket::orders(0).unwrap();
        assert_eq!(order.status, OrderStatus::Reviewed);
        assert_eq!(order.rating, Some(5));

        let review = DivinationMarket::reviews(0).expect("Review should exist");
        assert_eq!(review.overall_rating, 5);
        assert_eq!(review.divination_type, DivinationType::Meihua);
        assert!(!review.is_anonymous);

        // 验证提供者评分更新
        let provider = DivinationMarket::providers(10).unwrap();
        assert_eq!(provider.total_ratings, 1);
        assert_eq!(provider.rating_sum, 5);
    });
}

/// 测试无效评分
#[test]
fn invalid_rating_fails() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));
        assert_ok!(DivinationMarket::accept_order(RuntimeOrigin::signed(10), 0));
        assert_ok!(DivinationMarket::submit_interpretation(
            RuntimeOrigin::signed(10),
            0,
            b"Answer".to_vec(),
            vec![],
            vec![],
            vec![]
        ));

        // 评分 0 无效
        assert_noop!(
            DivinationMarket::submit_review(RuntimeOrigin::signed(1), 0, 0, 1, 1, 1, None, false),
            Error::<Test>::InvalidRating
        );

        // 评分 6 无效
        assert_noop!(
            DivinationMarket::submit_review(RuntimeOrigin::signed(1), 0, 6, 1, 1, 1, None, false),
            Error::<Test>::InvalidRating
        );
    });
}

/// 测试提供者回复评价
#[test]
fn reply_review_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));
        assert_ok!(DivinationMarket::accept_order(RuntimeOrigin::signed(10), 0));
        assert_ok!(DivinationMarket::submit_interpretation(
            RuntimeOrigin::signed(10),
            0,
            b"Answer".to_vec(),
            vec![],
            vec![],
            vec![]
        ));
        assert_ok!(DivinationMarket::submit_review(
            RuntimeOrigin::signed(1),
            0,
            5,
            5,
            5,
            5,
            None,
            false
        ));

        // 提供者回复
        assert_ok!(DivinationMarket::reply_review(
            RuntimeOrigin::signed(10),
            0,
            b"Thank you!".to_vec()
        ));

        let review = DivinationMarket::reviews(0).unwrap();
        assert!(review.provider_reply_cid.is_some());
    });
}

// ==================== 提现测试 ====================

/// 测试提现功能
#[test]
fn request_withdrawal_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));
        assert_ok!(DivinationMarket::accept_order(RuntimeOrigin::signed(10), 0));
        assert_ok!(DivinationMarket::submit_interpretation(
            RuntimeOrigin::signed(10),
            0,
            b"Answer".to_vec(),
            vec![],
            vec![],
            vec![]
        ));

        let provider_initial = Balances::free_balance(10);
        let balance = DivinationMarket::provider_balances(10);
        assert_eq!(balance, 800); // 1000 - 20% fee

        // 提现
        assert_ok!(DivinationMarket::request_withdrawal(
            RuntimeOrigin::signed(10),
            500
        ));

        assert_eq!(DivinationMarket::provider_balances(10), 300);
        assert_eq!(Balances::free_balance(10), provider_initial + 500);
    });
}

/// 测试余额不足提现失败
#[test]
fn withdrawal_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        assert_noop!(
            DivinationMarket::request_withdrawal(RuntimeOrigin::signed(10), 1000),
            Error::<Test>::InsufficientBalance
        );
    });
}

// ==================== 加急订单测试 ====================

/// 测试加急订单
#[test]
fn urgent_order_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        // 设置接受加急
        assert_ok!(DivinationMarket::update_provider(
            RuntimeOrigin::signed(10),
            None,
            None,
            None,
            None,
            None,
            Some(true)
        ));

        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            true, // urgent available
            2000  // 20% surcharge
        ));

        // 创建加急订单
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            true // urgent
        ));

        let order = DivinationMarket::orders(0).unwrap();
        assert!(order.is_urgent);
        assert_eq!(order.amount, 1200); // 1000 + 20%
    });
}

// ==================== 多占卜类型测试 ====================

/// 测试多种占卜类型
#[test]
fn multiple_divination_types_work() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());
        MockDivinationProvider::add_result(DivinationType::Bazi, 1, 2, RarityInput::common());

        // 注册支持多种类型的提供者
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000011 // 支持 Meihua 和 Bazi
        ));

        // 创建梅花套餐
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Meihua,
            ServiceType::TextReading,
            b"Meihua Package".to_vec(),
            b"Desc".to_vec(),
            1000,
            0,
            0,
            false,
            0
        ));

        // 创建八字套餐
        assert_ok!(DivinationMarket::create_package(
            RuntimeOrigin::signed(10),
            DivinationType::Bazi,
            ServiceType::TextReading,
            b"Bazi Package".to_vec(),
            b"Desc".to_vec(),
            2000,
            0,
            0,
            false,
            0
        ));

        // 梅花订单
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(1),
            10,
            DivinationType::Meihua,
            1,
            0,
            b"Cid".to_vec(),
            false
        ));

        // 八字订单
        assert_ok!(DivinationMarket::create_order(
            RuntimeOrigin::signed(2),
            10,
            DivinationType::Bazi,
            1,
            1,
            b"Cid2".to_vec(),
            false
        ));

        // 验证类型统计
        let meihua_stats = DivinationMarket::type_stats(DivinationType::Meihua);
        assert_eq!(meihua_stats.order_count, 1);

        let bazi_stats = DivinationMarket::type_stats(DivinationType::Bazi);
        assert_eq!(bazi_stats.order_count, 1);
    });
}

// ==================== 提供者等级测试 ====================

/// 测试提供者等级计算
#[test]
fn provider_tier_calculation() {
    assert_eq!(ProviderTier::Novice.min_orders(), 0);
    assert_eq!(ProviderTier::Certified.min_orders(), 10);
    assert_eq!(ProviderTier::Senior.min_orders(), 50);
    assert_eq!(ProviderTier::Expert.min_orders(), 200);
    assert_eq!(ProviderTier::Master.min_orders(), 500);

    assert_eq!(ProviderTier::Novice.platform_fee_rate(), 2000);
    assert_eq!(ProviderTier::Master.platform_fee_rate(), 800);
}

// ==================== 悬赏问答测试 ====================

/// 辅助函数：创建模拟占卜结果
fn setup_divination_result(result_id: u64, creator: u64) {
    MockDivinationProvider::add_result(
        DivinationType::Meihua,
        result_id,
        creator,
        RarityInput::common(),
    );
}


/// 测试创建悬赏问题
#[test]
fn create_bounty_works() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        let creator = 1u64;
        let bounty_amount = 10000u64;
        let deadline = 1000u64;

        let initial_balance = Balances::free_balance(creator);
        let platform_initial = Balances::free_balance(999);

        // 创建悬赏
        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(creator),
            DivinationType::Meihua,
            1, // result_id
            b"QmQuestionCid".to_vec(),
            bounty_amount,
            deadline,
            1,     // min_answers
            10,    // max_answers
            None,  // specialty
            false, // certified_only
            true   // allow_voting
        ));

        // 验证悬赏已创建
        let bounty = DivinationMarket::bounty_questions(0).expect("Bounty should exist");
        assert_eq!(bounty.creator, creator);
        assert_eq!(bounty.bounty_amount, bounty_amount);
        assert_eq!(bounty.divination_type, DivinationType::Meihua);
        assert_eq!(bounty.status, BountyStatus::Open);
        assert_eq!(bounty.min_answers, 1);
        assert_eq!(bounty.max_answers, 10);
        assert!(bounty.allow_voting);

        // 验证资金转移到平台账户托管
        assert_eq!(Balances::free_balance(creator), initial_balance - bounty_amount);
        assert_eq!(Balances::free_balance(999), platform_initial + bounty_amount);

        // 验证统计
        let stats = DivinationMarket::bounty_stats();
        assert_eq!(stats.total_bounties, 1);
        assert_eq!(stats.active_bounties, 1);
        assert_eq!(stats.total_bounty_amount, bounty_amount);

        // 验证用户索引
        let user_bounties = DivinationMarket::user_bounties(creator);
        assert_eq!(user_bounties.len(), 1);
        assert_eq!(user_bounties[0], 0);
    });
}

/// 测试悬赏金额过低失败
#[test]
fn create_bounty_amount_too_low_fails() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_noop!(
            DivinationMarket::create_bounty(
                RuntimeOrigin::signed(1),
                DivinationType::Meihua,
                1, // result_id
                b"Cid".to_vec(),
                50, // 低于 MinServicePrice (100)
                1000,
                1,
                10,
                None,
                false,
                false
            ),
            Error::<Test>::BountyAmountTooLow
        );
    });
}

/// 测试悬赏截止时间无效失败
#[test]
fn create_bounty_invalid_deadline_fails() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        System::set_block_number(100);

        assert_noop!(
            DivinationMarket::create_bounty(
                RuntimeOrigin::signed(1),
                DivinationType::Meihua,
                1, // result_id
                b"Cid".to_vec(),
                1000,
                50, // 截止时间已过
                1,
                10,
                None,
                false,
                false
            ),
            Error::<Test>::InvalidBountyDeadline
        );
    });
}

/// 测试提交悬赏回答
#[test]
fn submit_bounty_answer_works() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        // 创建悬赏
        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1,
            10,
            None,
            false,
            true
        ));

        // 提交回答
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"QmAnswerCid".to_vec()
        ));

        // 验证回答已创建
        let answer = DivinationMarket::bounty_answers(0).expect("Answer should exist");
        assert_eq!(answer.bounty_id, 0);
        assert_eq!(answer.answerer, 2);
        assert_eq!(answer.status, BountyAnswerStatus::Pending);
        assert_eq!(answer.votes, 0);

        // 验证悬赏回答数更新
        let bounty = DivinationMarket::bounty_questions(0).unwrap();
        assert_eq!(bounty.answer_count, 1);

        // 验证索引
        let answer_ids = DivinationMarket::bounty_answer_ids(0);
        assert_eq!(answer_ids.len(), 1);

        let user_answers = DivinationMarket::user_bounty_answers(2);
        assert_eq!(user_answers.len(), 1);

        // 验证统计
        let stats = DivinationMarket::bounty_stats();
        assert_eq!(stats.total_answers, 1);
    });
}

/// 测试不能回答自己的悬赏
#[test]
fn cannot_answer_own_bounty() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1,
            10,
            None,
            false,
            false
        ));

        assert_noop!(
            DivinationMarket::submit_bounty_answer(
                RuntimeOrigin::signed(1), // 悬赏创建者自己
                0,
                b"Answer".to_vec()
            ),
            Error::<Test>::CannotAnswerOwnBounty
        );
    });
}

/// 测试不能重复回答
#[test]
fn cannot_answer_twice() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1,
            10,
            None,
            false,
            false
        ));

        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer1".to_vec()
        ));

        assert_noop!(
            DivinationMarket::submit_bounty_answer(
                RuntimeOrigin::signed(2), // 同一用户再次回答
                0,
                b"Answer2".to_vec()
            ),
            Error::<Test>::AlreadyAnswered
        );
    });
}

/// 测试悬赏回答数上限
#[test]
fn bounty_answer_limit_reached() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1,
            2, // 最大回答数 2
            None,
            false,
            false
        ));

        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer1".to_vec()
        ));

        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(3),
            0,
            b"Answer2".to_vec()
        ));

        assert_noop!(
            DivinationMarket::submit_bounty_answer(RuntimeOrigin::signed(4), 0, b"Answer3".to_vec()),
            Error::<Test>::BountyAnswerLimitReached
        );
    });
}

/// 测试关闭悬赏
#[test]
fn close_bounty_works() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1, // min_answers = 1
            10,
            None,
            false,
            false
        ));

        // 提交一个回答
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer".to_vec()
        ));

        // 关闭悬赏
        assert_ok!(DivinationMarket::close_bounty(RuntimeOrigin::signed(1), 0));

        let bounty = DivinationMarket::bounty_questions(0).unwrap();
        assert_eq!(bounty.status, BountyStatus::Closed);
        assert!(bounty.closed_at.is_some());

        // 验证统计更新
        let stats = DivinationMarket::bounty_stats();
        assert_eq!(stats.active_bounties, 0);
    });
}

/// 测试回答不足不能关闭
#[test]
fn close_bounty_not_enough_answers_fails() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            3, // min_answers = 3
            10,
            None,
            false,
            false
        ));

        // 只有一个回答
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer".to_vec()
        ));

        assert_noop!(
            DivinationMarket::close_bounty(RuntimeOrigin::signed(1), 0),
            Error::<Test>::NotEnoughAnswers
        );
    });
}

/// 测试投票功能
#[test]
fn vote_bounty_answer_works() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1,
            10,
            None,
            false,
            true // allow_voting
        ));

        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer".to_vec()
        ));

        // 投票
        assert_ok!(DivinationMarket::vote_bounty_answer(
            RuntimeOrigin::signed(3),
            0,
            0
        ));

        // 验证投票已记录
        let vote = DivinationMarket::bounty_votes(0, 3).expect("Vote should exist");
        assert_eq!(vote.answer_id, 0);

        // 验证答案票数
        let answer = DivinationMarket::bounty_answers(0).unwrap();
        assert_eq!(answer.votes, 1);

        // 验证悬赏总票数
        let bounty = DivinationMarket::bounty_questions(0).unwrap();
        assert_eq!(bounty.total_votes, 1);
    });
}

/// 测试不能重复投票
#[test]
fn cannot_vote_twice() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1,
            10,
            None,
            false,
            true
        ));

        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer".to_vec()
        ));

        assert_ok!(DivinationMarket::vote_bounty_answer(
            RuntimeOrigin::signed(3),
            0,
            0
        ));

        assert_noop!(
            DivinationMarket::vote_bounty_answer(RuntimeOrigin::signed(3), 0, 0),
            Error::<Test>::AlreadyVoted
        );
    });
}

/// 测试采纳答案
#[test]
fn adopt_bounty_answers_works() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1,
            10,
            None,
            false,
            false
        ));

        // 提交三个回答
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer1".to_vec()
        ));
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(3),
            0,
            b"Answer2".to_vec()
        ));
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(4),
            0,
            b"Answer3".to_vec()
        ));

        // 采纳答案
        assert_ok!(DivinationMarket::adopt_bounty_answers(
            RuntimeOrigin::signed(1),
            0,
            0,       // 第一名
            Some(1), // 第二名
            Some(2)  // 第三名
        ));

        let bounty = DivinationMarket::bounty_questions(0).unwrap();
        assert_eq!(bounty.status, BountyStatus::Adopted);
        assert_eq!(bounty.adopted_answer_id, Some(0));
        assert_eq!(bounty.second_place_id, Some(1));
        assert_eq!(bounty.third_place_id, Some(2));

        // 验证答案状态
        assert_eq!(
            DivinationMarket::bounty_answers(0).unwrap().status,
            BountyAnswerStatus::Adopted
        );
        assert_eq!(
            DivinationMarket::bounty_answers(1).unwrap().status,
            BountyAnswerStatus::Selected
        );
        assert_eq!(
            DivinationMarket::bounty_answers(2).unwrap().status,
            BountyAnswerStatus::Selected
        );
    });
}

/// 测试结算悬赏（方案B - 多人奖励）
#[test]
fn settle_bounty_works() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        let bounty_amount = 10000u64;

        // 记录初始余额
        let answerer1_initial = Balances::free_balance(2);
        let answerer2_initial = Balances::free_balance(3);
        let answerer3_initial = Balances::free_balance(4);
        let answerer4_initial = Balances::free_balance(5);

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            bounty_amount,
            1000,
            1,
            10,
            None,
            false,
            false
        ));

        // 提交四个回答
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer1".to_vec()
        ));
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(3),
            0,
            b"Answer2".to_vec()
        ));
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(4),
            0,
            b"Answer3".to_vec()
        ));
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(5),
            0,
            b"Answer4".to_vec()
        ));

        // 采纳答案
        assert_ok!(DivinationMarket::adopt_bounty_answers(
            RuntimeOrigin::signed(1),
            0,
            0,       // 第一名
            Some(1), // 第二名
            Some(2)  // 第三名
        ));

        // 结算
        assert_ok!(DivinationMarket::settle_bounty(RuntimeOrigin::signed(99), 0));

        // 验证状态
        let bounty = DivinationMarket::bounty_questions(0).unwrap();
        assert_eq!(bounty.status, BountyStatus::Settled);
        assert!(bounty.settled_at.is_some());

        // 验证奖励分配（方案B）
        // 第一名 60% = 6000
        assert_eq!(Balances::free_balance(2), answerer1_initial + 6000);
        assert_eq!(DivinationMarket::bounty_answers(0).unwrap().reward_amount, 6000);

        // 第二名 15% = 1500
        assert_eq!(Balances::free_balance(3), answerer2_initial + 1500);
        assert_eq!(DivinationMarket::bounty_answers(1).unwrap().reward_amount, 1500);

        // 第三名 5% = 500
        assert_eq!(Balances::free_balance(4), answerer3_initial + 500);
        assert_eq!(DivinationMarket::bounty_answers(2).unwrap().reward_amount, 500);

        // 参与奖 5% = 500，只有1人参与，所以全部给第4位
        assert_eq!(Balances::free_balance(5), answerer4_initial + 500);
        assert_eq!(DivinationMarket::bounty_answers(3).unwrap().reward_amount, 500);
        assert_eq!(
            DivinationMarket::bounty_answers(3).unwrap().status,
            BountyAnswerStatus::Participated
        );

        // 平台费 15% = 1500 保留在平台账户

        // 验证统计
        let stats = DivinationMarket::bounty_stats();
        assert_eq!(stats.settled_bounties, 1);
    });
}

/// 测试取消悬赏（无回答时）
#[test]
fn cancel_bounty_works() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        let bounty_amount = 10000u64;
        let creator_initial = Balances::free_balance(1);

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            bounty_amount,
            1000,
            1,
            10,
            None,
            false,
            false
        ));

        // 取消悬赏
        assert_ok!(DivinationMarket::cancel_bounty(RuntimeOrigin::signed(1), 0));

        let bounty = DivinationMarket::bounty_questions(0).unwrap();
        assert_eq!(bounty.status, BountyStatus::Cancelled);

        // 验证退款
        assert_eq!(Balances::free_balance(1), creator_initial);

        // 验证统计
        let stats = DivinationMarket::bounty_stats();
        assert_eq!(stats.active_bounties, 0);
    });
}

/// 测试有回答时不能取消悬赏
#[test]
fn cancel_bounty_with_answers_fails() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1,
            10,
            None,
            false,
            false
        ));

        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer".to_vec()
        ));

        assert_noop!(
            DivinationMarket::cancel_bounty(RuntimeOrigin::signed(1), 0),
            Error::<Test>::BountyCannotCancel
        );
    });
}

/// 测试过期悬赏处理（无回答时退款）
#[test]
fn expire_bounty_no_answers_works() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        let bounty_amount = 10000u64;
        let creator_initial = Balances::free_balance(1);

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            bounty_amount,
            100, // deadline
            1,
            10,
            None,
            false,
            false
        ));

        // 设置区块超过 deadline
        System::set_block_number(101);

        // 处理过期
        assert_ok!(DivinationMarket::expire_bounty(RuntimeOrigin::signed(99), 0));

        let bounty = DivinationMarket::bounty_questions(0).unwrap();
        assert_eq!(bounty.status, BountyStatus::Expired);

        // 验证退款
        assert_eq!(Balances::free_balance(1), creator_initial);
    });
}

/// 测试过期悬赏处理（有回答时关闭）
#[test]
fn expire_bounty_with_answers_closes() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            100, // deadline
            1,
            10,
            None,
            false,
            false
        ));

        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(2),
            0,
            b"Answer".to_vec()
        ));

        // 设置区块超过 deadline
        System::set_block_number(101);

        // 处理过期（有回答，只关闭不退款）
        assert_ok!(DivinationMarket::expire_bounty(RuntimeOrigin::signed(99), 0));

        let bounty = DivinationMarket::bounty_questions(0).unwrap();
        assert_eq!(bounty.status, BountyStatus::Closed);
        // 资金仍在平台账户，等待创建者采纳
    });
}

/// 测试仅限认证提供者回答
#[test]
fn certified_only_bounty_works() {
    new_test_ext().execute_with(|| {
        setup_divination_result(1, 1); // Setup divination result for bounty test

        // 注册认证提供者
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(10),
            b"Certified Provider".to_vec(),
            b"Bio".to_vec(),
            0b00000001,
            0b00000001
        ));

        // 手动设置提供者等级为 Certified（实际项目中通过完成订单升级）
        crate::Providers::<Test>::mutate(10, |maybe_provider| {
            if let Some(p) = maybe_provider {
                p.tier = ProviderTier::Certified;
            }
        });

        // 创建仅限认证提供者的悬赏
        assert_ok!(DivinationMarket::create_bounty(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1, // result_id
            b"Question".to_vec(),
            10000,
            1000,
            1,
            10,
            None,
            true, // certified_only
            false
        ));

        // 非认证用户回答失败
        assert_noop!(
            DivinationMarket::submit_bounty_answer(RuntimeOrigin::signed(2), 0, b"Answer".to_vec()),
            Error::<Test>::CertifiedProviderOnly
        );

        // 认证提供者回答成功
        assert_ok!(DivinationMarket::submit_bounty_answer(
            RuntimeOrigin::signed(10),
            0,
            b"Answer".to_vec()
        ));

        // 验证回答包含认证信息
        let answer = DivinationMarket::bounty_answers(0).unwrap();
        assert!(answer.is_certified);
        assert_eq!(answer.provider_tier, Some(ProviderTier::Certified));
    });
}

/// 测试奖励分配比例验证
#[test]
fn reward_distribution_validation() {
    let valid_dist = RewardDistribution::default();
    assert!(valid_dist.is_valid());

    let invalid_dist = RewardDistribution {
        first_place: 7000,       // 增加了10%
        second_place: 1500,
        third_place: 500,
        platform_fee: 1500,
        participation_pool: 500,
    };
    assert!(!invalid_dist.is_valid()); // 总和 11000 != 10000
}

// ==================== 举报系统测试 ====================

/// 辅助函数：设置测试环境（提供者已注册）
fn setup_provider_for_report(provider: u64) {
    assert_ok!(DivinationMarket::register_provider(
        RuntimeOrigin::signed(provider),
        b"Provider".to_vec(),
        b"Bio".to_vec(),
        0b00000001,
        0b00000001
    ));
}

// ==================== 举报功能测试（已移除，待重新实现） ====================
// 以下测试引用了已删除的举报功能，暂时注释掉
/*
/// 测试提交举报
#[test]
fn submit_report_works() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        let reporter = 1u64;
        let provider = 10u64;
        let reporter_initial = Balances::free_balance(reporter);
        let platform_initial = Balances::free_balance(999);

        // 提交举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(reporter),
            provider,
            ReportType::Abuse,  // 辱骂
            b"QmEvidenceCid".to_vec(),
            b"Provider was abusive".to_vec(),
            None,  // related_order_id
            None,  // related_bounty_id
            None,  // related_answer_id
            false  // not anonymous
        ));

        // 验证举报已创建
        let report = DivinationMarket::reports(0).expect("Report should exist");
        assert_eq!(report.reporter, reporter);
        assert_eq!(report.provider, provider);
        assert_eq!(report.report_type, ReportType::Abuse);
        assert_eq!(report.status, ReportStatus::Pending);
        assert!(!report.is_anonymous);

        // 验证押金已扣除（Abuse 类型是 0.8x 倍率，1000 * 80 / 100 = 800）
        let expected_deposit = 800u64;
        assert_eq!(report.reporter_deposit, expected_deposit);
        assert_eq!(Balances::free_balance(reporter), reporter_initial - expected_deposit);
        assert_eq!(Balances::free_balance(999), platform_initial + expected_deposit);

        // 验证索引已更新
        let provider_reports = DivinationMarket::provider_reports(provider);
        assert_eq!(provider_reports.len(), 1);
        assert_eq!(provider_reports[0], 0);

        let user_reports = DivinationMarket::user_reports(reporter);
        assert_eq!(user_reports.len(), 1);

        let pending = DivinationMarket::pending_reports();
        assert_eq!(pending.len(), 1);

        // 验证统计
        let stats = DivinationMarket::report_stats();
        assert_eq!(stats.total_reports, 1);
        assert_eq!(stats.pending_reports, 1);

        // 验证大师举报档案
        let profile = DivinationMarket::provider_report_profiles(provider);
        assert_eq!(profile.total_reported, 1);
    });
}

/// 测试不能举报自己
#[test]
fn submit_report_cannot_report_self() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        assert_noop!(
            DivinationMarket::submit_report(
                RuntimeOrigin::signed(10),  // 提供者举报自己
                10,
                ReportType::Abuse,
                b"Evidence".to_vec(),
                b"Description".to_vec(),
                None,
                None,
                None,
                false
            ),
            Error::<Test>::CannotReportSelf
        );
    });
}

/// 测试举报不存在的大师失败
#[test]
fn submit_report_provider_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            DivinationMarket::submit_report(
                RuntimeOrigin::signed(1),
                99,  // 不存在的提供者
                ReportType::Abuse,
                b"Evidence".to_vec(),
                b"Description".to_vec(),
                None,
                None,
                None,
                false
            ),
            Error::<Test>::ProviderNotFound
        );
    });
}

/// 测试举报冷却期
#[test]
fn submit_report_cooldown_works() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        // 第一次举报成功
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(1),
            10,
            ReportType::Abuse,
            b"Evidence1".to_vec(),
            b"Description1".to_vec(),
            None,
            None,
            None,
            false
        ));

        // 立即第二次举报失败（冷却期中）
        assert_noop!(
            DivinationMarket::submit_report(
                RuntimeOrigin::signed(1),
                10,
                ReportType::Fraud,
                b"Evidence2".to_vec(),
                b"Description2".to_vec(),
                None,
                None,
                None,
                false
            ),
            Error::<Test>::ReportCooldownActive
        );

        // 推进区块超过冷却期（100 区块）
        System::set_block_number(102);

        // 冷却期后可以再次举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(1),
            10,
            ReportType::Fraud,
            b"Evidence2".to_vec(),
            b"Description2".to_vec(),
            None,
            None,
            None,
            false
        ));
    });
}

/// 测试不同举报类型的押金计算
#[test]
fn report_deposit_calculation() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);
        setup_provider_for_report(11);

        // Abuse (0.8x): 1000 * 80 / 100 = 800
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(1),
            10,
            ReportType::Abuse,
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None, false
        ));
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.reporter_deposit, 800);

        // Other (2x): 1000 * 200 / 100 = 2000
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(2),
            11,
            ReportType::Other,
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None, false
        ));
        let report = DivinationMarket::reports(1).unwrap();
        assert_eq!(report.reporter_deposit, 2000);
    });
}

/// 测试撤回举报
#[test]
fn withdraw_report_works() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        let reporter = 1u64;
        let reporter_initial = Balances::free_balance(reporter);

        // 提交举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(reporter),
            10,
            ReportType::Abuse,
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None, false
        ));

        let deposit = 800u64;  // Abuse 类型押金
        assert_eq!(Balances::free_balance(reporter), reporter_initial - deposit);

        // 撤回举报（在窗口期内）
        assert_ok!(DivinationMarket::withdraw_report(
            RuntimeOrigin::signed(reporter),
            0
        ));

        // 验证状态更新
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.status, ReportStatus::Withdrawn);

        // 验证 80% 押金退还
        let refund = deposit * 80 / 100;  // 640
        assert_eq!(Balances::free_balance(reporter), reporter_initial - deposit + refund);

        // 验证从待处理队列移除
        let pending = DivinationMarket::pending_reports();
        assert_eq!(pending.len(), 0);

        // 验证统计更新
        let stats = DivinationMarket::report_stats();
        assert_eq!(stats.pending_reports, 0);
    });
}

/// 测试撤回窗口过期后不能撤回
#[test]
fn withdraw_report_window_expired() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        // 提交举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(1),
            10,
            ReportType::Abuse,
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None, false
        ));

        // 推进区块超过撤回窗口（50 区块）
        System::set_block_number(52);

        // 撤回失败
        assert_noop!(
            DivinationMarket::withdraw_report(RuntimeOrigin::signed(1), 0),
            Error::<Test>::WithdrawWindowExpired
        );
    });
}

/// 测试非举报者不能撤回
#[test]
fn withdraw_report_not_reporter() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(1),
            10,
            ReportType::Abuse,
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None, false
        ));

        // 他人尝试撤回
        assert_noop!(
            DivinationMarket::withdraw_report(RuntimeOrigin::signed(2), 0),
            Error::<Test>::NotReporter
        );
    });
}

/// 测试审核举报成立
#[test]
fn resolve_report_upheld_works() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        let reporter = 1u64;
        let provider = 10u64;
        let reporter_initial = Balances::free_balance(reporter);
        let treasury_initial = Balances::free_balance(888);

        // 提交举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(reporter),
            provider,
            ReportType::FalseAdvertising,  // 虚假宣传，30% 罚金
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None, false
        ));

        let report = DivinationMarket::reports(0).unwrap();
        let reporter_deposit = report.reporter_deposit;  // 1200 (1.2x)

        // Root 权限审核举报成立
        assert_ok!(DivinationMarket::resolve_report(
            RuntimeOrigin::root(),
            0,
            ReportStatus::Upheld,
            Some(b"QmResolutionCid".to_vec()),
            None  // 使用默认惩罚比例
        ));

        // 验证状态
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.status, ReportStatus::Upheld);
        assert!(report.resolved_at.is_some());
        assert!(report.resolution_cid.is_some());

        // 计算惩罚金额
        // 大师押金 10000，虚假宣传罚金比例 30%
        let provider_deposit = 10000u64;
        let penalty_rate = 3000u16;  // 30%
        let penalty_amount = provider_deposit * penalty_rate as u64 / 10000;  // 3000

        // 举报者奖励 30% of 罚金
        let reward_rate = 3000u16;  // 30%
        let reporter_reward = penalty_amount * reward_rate as u64 / 10000;  // 900

        // 验证举报者收到奖励 + 退还押金
        let expected_reporter_balance = reporter_initial - reporter_deposit + reporter_reward + reporter_deposit;
        assert_eq!(Balances::free_balance(reporter), expected_reporter_balance);

        // 验证国库收到剩余罚金
        let treasury_income = penalty_amount - reporter_reward;  // 2100
        assert_eq!(Balances::free_balance(888), treasury_initial + treasury_income);

        // 验证统计
        let stats = DivinationMarket::report_stats();
        assert_eq!(stats.upheld_reports, 1);
        assert_eq!(stats.pending_reports, 0);

        // 验证大师举报档案
        let profile = DivinationMarket::provider_report_profiles(provider);
        assert_eq!(profile.upheld_count, 1);
    });
}

/// 测试审核举报驳回
#[test]
fn resolve_report_rejected_works() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        let reporter = 1u64;
        let reporter_initial = Balances::free_balance(reporter);

        // 提交举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(reporter),
            10,
            ReportType::Abuse,
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None, false
        ));

        let report = DivinationMarket::reports(0).unwrap();
        let _reporter_deposit = report.reporter_deposit;

        // 审核驳回
        assert_ok!(DivinationMarket::resolve_report(
            RuntimeOrigin::root(),
            0,
            ReportStatus::Rejected,
            None,
            None
        ));

        // 验证状态
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.status, ReportStatus::Rejected);

        // 验证全额退还押金
        assert_eq!(Balances::free_balance(reporter), reporter_initial);

        // 验证统计
        let stats = DivinationMarket::report_stats();
        assert_eq!(stats.rejected_reports, 1);
    });
}

/// 测试审核恶意举报
#[test]
fn resolve_report_malicious_works() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        let reporter = 1u64;
        let reporter_initial = Balances::free_balance(reporter);
        let treasury_initial = Balances::free_balance(888);

        // 提交举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(reporter),
            10,
            ReportType::Abuse,
            b"FakeEvidence".to_vec(),
            b"Malicious report".to_vec(),
            None, None, None, false
        ));

        let report = DivinationMarket::reports(0).unwrap();
        let reporter_deposit = report.reporter_deposit;  // 800

        // 审核为恶意举报
        assert_ok!(DivinationMarket::resolve_report(
            RuntimeOrigin::root(),
            0,
            ReportStatus::Malicious,
            None,
            None
        ));

        // 验证状态
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.status, ReportStatus::Malicious);

        // 验证押金被没收到国库
        assert_eq!(Balances::free_balance(reporter), reporter_initial - reporter_deposit);
        assert_eq!(Balances::free_balance(888), treasury_initial + reporter_deposit);

        // 验证统计
        let stats = DivinationMarket::report_stats();
        assert_eq!(stats.malicious_reports, 1);
        assert_eq!(stats.total_confiscated_deposits, reporter_deposit);
    });
}

/// 测试举报过期处理
#[test]
fn expire_report_works() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        let reporter = 1u64;
        let reporter_initial = Balances::free_balance(reporter);

        // 提交举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(reporter),
            10,
            ReportType::Abuse,
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None, false
        ));

        let report = DivinationMarket::reports(0).unwrap();
        let _reporter_deposit = report.reporter_deposit;

        // 推进区块超过超时时间（2000 区块）
        System::set_block_number(2002);

        // 任何人可调用过期处理
        assert_ok!(DivinationMarket::expire_report(
            RuntimeOrigin::signed(99),
            0
        ));

        // 验证状态
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.status, ReportStatus::Expired);

        // 验证全额退还押金给举报者
        assert_eq!(Balances::free_balance(reporter), reporter_initial);

        // 验证统计
        let stats = DivinationMarket::report_stats();
        assert_eq!(stats.pending_reports, 0);
    });
}

/// 测试举报未过期不能调用过期处理
#[test]
fn expire_report_not_expired() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(1),
            10,
            ReportType::Abuse,
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None, false
        ));

        // 时间未超过超时
        System::set_block_number(100);

        assert_noop!(
            DivinationMarket::expire_report(RuntimeOrigin::signed(99), 0),
            Error::<Test>::ReportNotExpired
        );
    });
}

/// 测试匿名举报
#[test]
fn anonymous_report_works() {
    new_test_ext().execute_with(|| {
        setup_provider_for_report(10);

        // 提交匿名举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(1),
            10,
            ReportType::Abuse,
            b"Evidence".to_vec(),
            b"Description".to_vec(),
            None, None, None,
            true  // anonymous
        ));

        let report = DivinationMarket::reports(0).unwrap();
        assert!(report.is_anonymous);
        // 注意：即使是匿名举报，reporter 字段仍然存储（用于退还押金等）
        // 但在事件中会显示为 None
    });
}

/// 测试举报类型的永久封禁触发
#[test]
fn report_type_permanent_ban() {
    // 验证哪些类型触发永久封禁
    assert!(ReportType::Drugs.triggers_permanent_ban());
    assert!(ReportType::Fraud.triggers_permanent_ban());
    assert!(!ReportType::Abuse.triggers_permanent_ban());
    assert!(!ReportType::FalseAdvertising.triggers_permanent_ban());
}

/// 测试举报类型的配置参数
#[test]
fn report_type_configurations() {
    // 测试押金倍率
    assert_eq!(ReportType::Abuse.deposit_multiplier(), 80);  // 0.8x
    assert_eq!(ReportType::Fraud.deposit_multiplier(), 150); // 1.5x
    assert_eq!(ReportType::Other.deposit_multiplier(), 200); // 2x

    // 测试罚金比例
    assert_eq!(ReportType::Drugs.provider_penalty_rate(), 10000);      // 100%
    assert_eq!(ReportType::FalseAdvertising.provider_penalty_rate(), 3000); // 30%

    // 测试奖励比例
    assert_eq!(ReportType::Fraud.reporter_reward_rate(), 5000);  // 50%
    assert_eq!(ReportType::Superstition.reporter_reward_rate(), 2000); // 20%

    // 测试信用扣分
    assert_eq!(ReportType::Drugs.credit_deduction(), 500);
    assert_eq!(ReportType::Superstition.credit_deduction(), 50);
}
*/
