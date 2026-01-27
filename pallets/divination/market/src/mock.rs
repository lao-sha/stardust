//! 通用占卜服务市场 Pallet 测试模拟环境

use crate as pallet_divination_market;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU16, ConstU32, ConstU64},
};
use pallet_divination_common::{DivinationProvider, DivinationType, RarityInput};
use sp_runtime::BuildStorage;
use sp_std::vec::Vec;

type Block = frame_system::mocking::MockBlock<Test>;

// 构建模拟运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        DivinationMarket: pallet_divination_market,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<u64>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
    type Balance = u64;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<1>;
    type WeightInfo = ();
}

/// 模拟 DivinationProvider 用于测试
pub struct MockDivinationProvider;

// 用于测试的模拟数据
thread_local! {
    static MOCK_RESULTS: std::cell::RefCell<std::collections::HashMap<(DivinationType, u64), MockResult>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

#[derive(Clone)]
pub struct MockResult {
    pub creator: u64,
    pub rarity_input: RarityInput,
}

impl MockDivinationProvider {
    /// 添加模拟的占卜结果
    pub fn add_result(
        divination_type: DivinationType,
        result_id: u64,
        creator: u64,
        rarity_input: RarityInput,
    ) {
        MOCK_RESULTS.with(|r| {
            r.borrow_mut().insert(
                (divination_type, result_id),
                MockResult {
                    creator,
                    rarity_input,
                },
            );
        });
    }

    /// 清除所有模拟数据
    pub fn clear() {
        MOCK_RESULTS.with(|r| r.borrow_mut().clear());
    }
}

impl DivinationProvider<u64> for MockDivinationProvider {
    fn result_exists(divination_type: DivinationType, result_id: u64) -> bool {
        MOCK_RESULTS.with(|r| r.borrow().contains_key(&(divination_type, result_id)))
    }

    fn result_creator(divination_type: DivinationType, result_id: u64) -> Option<u64> {
        MOCK_RESULTS.with(|r| {
            r.borrow()
                .get(&(divination_type, result_id))
                .map(|m| m.creator)
        })
    }

    fn rarity_data(divination_type: DivinationType, result_id: u64) -> Option<RarityInput> {
        MOCK_RESULTS.with(|r| {
            r.borrow()
                .get(&(divination_type, result_id))
                .map(|m| m.rarity_input.clone())
        })
    }

    fn result_summary(_divination_type: DivinationType, _result_id: u64) -> Option<Vec<u8>> {
        Some(b"mock summary".to_vec())
    }

    fn is_nftable(_divination_type: DivinationType, _result_id: u64) -> bool {
        true
    }

    fn mark_as_nfted(_divination_type: DivinationType, _result_id: u64) {
        // no-op
    }
}

parameter_types! {
    pub PlatformAccount: u64 = 999;
    pub TreasuryAccount: u64 = 888;
}

/// 模拟治理权限
pub struct MockGovernanceOrigin;

impl<O: Into<Result<frame_system::RawOrigin<u64>, O>> + From<frame_system::RawOrigin<u64>>>
    frame_support::traits::EnsureOrigin<O> for MockGovernanceOrigin
{
    type Success = ();

    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            frame_system::RawOrigin::Root => Ok(()),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(frame_system::RawOrigin::Root))
    }
}

/// 模拟举报审核委员会权限（返回审核人 AccountId）
pub struct MockReportReviewOrigin;

impl<O: Into<Result<frame_system::RawOrigin<u64>, O>> + From<frame_system::RawOrigin<u64>>>
    frame_support::traits::EnsureOrigin<O> for MockReportReviewOrigin
{
    type Success = u64; // 返回审核人账户

    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            // Root 权限下，模拟审核人为账户 100
            frame_system::RawOrigin::Root => Ok(100u64),
            // 签名账户直接返回账户 ID
            frame_system::RawOrigin::Signed(who) => Ok(who),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(frame_system::RawOrigin::Root))
    }
}

/// 模拟 IPFS 内容注册
pub struct MockContentRegistry;

impl pallet_storage_service::ContentRegistry for MockContentRegistry {
    fn register_content(
        _domain: Vec<u8>,
        _subject_id: u64,
        _cid: Vec<u8>,
        _tier: pallet_storage_service::PinTier,
    ) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn is_domain_registered(_domain: &[u8]) -> bool {
        true
    }

    fn get_domain_subject_type(_domain: &[u8]) -> Option<pallet_storage_service::SubjectType> {
        Some(pallet_storage_service::SubjectType::General)
    }
}

/// 模拟定价接口
pub struct MockPricing;

impl pallet_trading_common::PricingProvider<u64> for MockPricing {
    fn get_dust_to_usd_rate() -> Option<u64> {
        Some(1_000_000) // 1 DUST = 1 USD
    }

    fn report_swap_order(_timestamp: u64, _price_usdt: u64, _dust_qty: u128) -> sp_runtime::DispatchResult {
        Ok(())
    }
}

/// 模拟联盟分成
pub struct MockAffiliateDistributor;

impl pallet_affiliate::types::AffiliateDistributor<u64, u128, u64> for MockAffiliateDistributor {
    fn distribute_rewards(
        _buyer: &u64,
        _amount: u128,
        _target: Option<(u8, u64)>,
    ) -> Result<u128, sp_runtime::DispatchError> {
        Ok(0)
    }
}

/// 模拟聊天权限管理
pub struct MockChatPermission;

impl pallet_chat_permission::SceneAuthorizationManager<u64, u64> for MockChatPermission {
    fn grant_scene_authorization(
        _source: [u8; 8],
        _from: &u64,
        _to: &u64,
        _scene_type: pallet_chat_permission::SceneType,
        _scene_id: pallet_chat_permission::SceneId,
        _duration: Option<u64>,
        _metadata: Vec<u8>,
    ) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn grant_bidirectional_scene_authorization(
        _source: [u8; 8],
        _user1: &u64,
        _user2: &u64,
        _scene_type: pallet_chat_permission::SceneType,
        _scene_id: pallet_chat_permission::SceneId,
        _duration: Option<u64>,
        _metadata: Vec<u8>,
    ) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn revoke_scene_authorization(
        _source: [u8; 8],
        _from: &u64,
        _to: &u64,
        _scene_type: pallet_chat_permission::SceneType,
        _scene_id: pallet_chat_permission::SceneId,
    ) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn revoke_all_by_source(
        _source: [u8; 8],
        _user1: &u64,
        _user2: &u64,
    ) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn extend_scene_authorization(
        _source: [u8; 8],
        _from: &u64,
        _to: &u64,
        _scene_type: pallet_chat_permission::SceneType,
        _scene_id: pallet_chat_permission::SceneId,
        _additional_duration: u64,
    ) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn has_any_valid_scene_authorization(_from: &u64, _to: &u64) -> bool {
        false
    }

    fn get_valid_scene_authorizations(
        _user1: &u64,
        _user2: &u64,
    ) -> Vec<pallet_chat_permission::SceneAuthorization<u64>> {
        Vec::new()
    }
}

impl pallet_divination_market::Config for Test {
    type Currency = Balances;
    type DivinationProvider = MockDivinationProvider;
    type ContentRegistry = MockContentRegistry;
    type MinDeposit = ConstU64<10_000>;
    type MinDepositUsd = ConstU64<100_000_000>;
    type Pricing = MockPricing;
    type MinServicePrice = ConstU64<100>;
    type MaxServicePrice = ConstU64<100_000_000>;
    type OrderTimeout = ConstU64<1000>;
    type AcceptTimeout = ConstU64<100>;
    type ReviewPeriod = ConstU64<500>;
    type WithdrawalCooldown = ConstU64<100>;
    type MaxNameLength = ConstU32<64>;
    type MaxBioLength = ConstU32<256>;
    type MaxDescriptionLength = ConstU32<512>;
    type MaxCidLength = ConstU32<64>;
    type MaxPackagesPerProvider = ConstU32<10>;
    type MaxFollowUpsPerOrder = ConstU32<5>;
    type PlatformAccount = PlatformAccount;
    type GovernanceOrigin = MockGovernanceOrigin;
    type TreasuryAccount = TreasuryAccount;
    // 🆕 联盟计酬
    type AffiliateDistributor = MockAffiliateDistributor;
    // 🆕 解读修改窗口
    type InterpretationEditWindow = ConstU64<28800>;
    // 🆕 聊天权限集成
    type ChatPermission = MockChatPermission;
    type OrderChatDuration = ConstU64<432000>;
}

/// 构建测试外部状态
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000),    // 客户1
            (2, 1_000_000),    // 客户2
            (10, 1_000_000),   // 提供者1
            (11, 1_000_000),   // 提供者2
            (100, 1_000_000),  // 举报审核人（委员会成员）
            (888, 10_000_000), // 国库账户
            (999, 10_000_000), // 平台账户
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        MockDivinationProvider::clear();
    });
    ext
}

/// 推进区块
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}
