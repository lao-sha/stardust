//! # Affiliate Pallet Benchmarking
//!
//! 函数级中文注释：Affiliate Pallet 基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::BoundedVec;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn bind_sponsor() {
        // 设置：创建推荐码
        let sponsor: T::AccountId = whitelisted_caller();
        let caller: T::AccountId = account("caller", 1, 0);
        
        // 为 sponsor 创建推荐码
        let code: BoundedVec<u8, T::MaxCodeLen> = b"SPONSOR1".to_vec().try_into().unwrap();
        pallet_affiliate_referral::AccountByCode::<T>::insert(&code, &sponsor);
        pallet_affiliate_referral::CodeByAccount::<T>::insert(&sponsor, &code);

        #[extrinsic_call]
        bind_sponsor(RawOrigin::Signed(caller), b"SPONSOR1".to_vec());
    }

    #[benchmark]
    fn claim_code() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        claim_code(RawOrigin::Signed(caller), b"MYCODE01".to_vec());
    }

    #[benchmark]
    fn set_settlement_mode() {
        #[extrinsic_call]
        set_settlement_mode(RawOrigin::Root, 0, 0, 0);
    }

    #[benchmark]
    fn set_weekly_percents() {
        let percents = vec![20, 10, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];

        #[extrinsic_call]
        set_weekly_percents(RawOrigin::Root, percents);
    }

    #[benchmark]
    fn set_blocks_per_week() {
        #[extrinsic_call]
        set_blocks_per_week(RawOrigin::Root, 100800u32.into());
    }

    #[benchmark]
    fn settle_cycle() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        settle_cycle(RawOrigin::Signed(caller), 1, 10);
    }

    #[benchmark]
    fn propose_percentage_adjustment() {
        let caller: T::AccountId = whitelisted_caller();
        let new_percentages: crate::types::LevelPercents = 
            vec![30, 25, 15, 10, 7, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1].try_into().unwrap();
        let title_cid: BoundedVec<u8, ConstU32<64>> = b"QmTitle".to_vec().try_into().unwrap();
        let desc_cid: BoundedVec<u8, ConstU32<64>> = b"QmDesc".to_vec().try_into().unwrap();
        let rationale_cid: BoundedVec<u8, ConstU32<64>> = b"QmRationale".to_vec().try_into().unwrap();

        #[extrinsic_call]
        propose_percentage_adjustment(
            RawOrigin::Signed(caller),
            new_percentages,
            title_cid,
            desc_cid,
            rationale_cid
        );
    }

    #[benchmark]
    fn emergency_pause_governance() {
        let reason: BoundedVec<u8, ConstU32<64>> = b"Security".to_vec().try_into().unwrap();

        #[extrinsic_call]
        emergency_pause_governance(RawOrigin::Root, reason);
    }

    #[benchmark]
    fn resume_governance() {
        // 先暂停
        GovernancePaused::<T>::put(true);

        #[extrinsic_call]
        resume_governance(RawOrigin::Root);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
