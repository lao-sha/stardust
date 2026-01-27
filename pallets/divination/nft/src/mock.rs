//! Mock runtime for testing pallet-divination-nft

use crate as pallet_divination_nft;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU16, ConstU32, ConstU64},
};
use pallet_divination_common::{DivinationProvider, DivinationType, RarityInput};
use sp_runtime::BuildStorage;
use sp_std::vec::Vec;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        DivinationNftPallet: pallet_divination_nft,
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

/// Mock DivinationProvider for testing
pub struct MockDivinationProvider;

// 用于测试的模拟数据
thread_local! {
    static MOCK_RESULTS: std::cell::RefCell<std::collections::HashMap<(DivinationType, u64), MockResult>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    static NFTED_RESULTS: std::cell::RefCell<std::collections::HashSet<(DivinationType, u64)>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
}

#[derive(Clone)]
pub struct MockResult {
    pub creator: u64,
    pub rarity_input: RarityInput,
}

impl MockDivinationProvider {
    /// 添加模拟的占卜结果
    pub fn add_result(divination_type: DivinationType, result_id: u64, creator: u64, rarity_input: RarityInput) {
        MOCK_RESULTS.with(|r| {
            r.borrow_mut().insert((divination_type, result_id), MockResult { creator, rarity_input });
        });
    }

    /// 清除所有模拟数据
    pub fn clear() {
        MOCK_RESULTS.with(|r| r.borrow_mut().clear());
        NFTED_RESULTS.with(|r| r.borrow_mut().clear());
    }
}

impl DivinationProvider<u64> for MockDivinationProvider {
    fn result_exists(divination_type: DivinationType, result_id: u64) -> bool {
        MOCK_RESULTS.with(|r| r.borrow().contains_key(&(divination_type, result_id)))
    }

    fn result_creator(divination_type: DivinationType, result_id: u64) -> Option<u64> {
        MOCK_RESULTS.with(|r| {
            r.borrow().get(&(divination_type, result_id)).map(|m| m.creator)
        })
    }

    fn rarity_data(divination_type: DivinationType, result_id: u64) -> Option<RarityInput> {
        MOCK_RESULTS.with(|r| {
            r.borrow().get(&(divination_type, result_id)).map(|m| m.rarity_input.clone())
        })
    }

    fn result_summary(_divination_type: DivinationType, _result_id: u64) -> Option<Vec<u8>> {
        Some(b"mock summary".to_vec())
    }

    fn is_nftable(divination_type: DivinationType, result_id: u64) -> bool {
        // 结果存在且未被 NFT 化
        Self::result_exists(divination_type, result_id) &&
            !NFTED_RESULTS.with(|r| r.borrow().contains(&(divination_type, result_id)))
    }

    fn mark_as_nfted(divination_type: DivinationType, result_id: u64) {
        NFTED_RESULTS.with(|r| {
            r.borrow_mut().insert((divination_type, result_id));
        });
    }
}

parameter_types! {
    pub const PlatformAccount: u64 = 999;
}

/// Mock ContentRegistry 实现
pub struct MockContentRegistry;

impl pallet_storage_service::ContentRegistry for MockContentRegistry {
    fn register_content(
        _domain: sp_std::vec::Vec<u8>,
        _subject_id: u64,
        _cid: sp_std::vec::Vec<u8>,
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

impl pallet_divination_nft::Config for Test {
    type NftCurrency = Balances;
    type DivinationProvider = MockDivinationProvider;
    type ContentRegistry = MockContentRegistry;
    type MaxNameLength = ConstU32<64>;
    type MaxCidLength = ConstU32<128>;
    type MaxCollectionsPerUser = ConstU32<50>;
    type MaxNftsPerCollection = ConstU32<100>;
    type MaxOffersPerNft = ConstU32<100>;
    type BaseMintFee = ConstU64<1_000_000_000_000>; // 1 UNIT
    type PlatformFeeRate = ConstU16<250>; // 2.5%
    type MaxRoyaltyRate = ConstU16<2500>; // 25%
    type OfferValidityPeriod = ConstU64<100>;
    type PlatformAccount = PlatformAccount;
    type GovernanceOrigin = frame_system::EnsureRoot<u64>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 100_000_000_000_000), // Alice: 100 UNIT
            (2, 100_000_000_000_000), // Bob: 100 UNIT
            (3, 100_000_000_000_000), // Charlie: 100 UNIT
            (999, 1_000_000_000),     // Platform account: minimal balance to satisfy ED
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
