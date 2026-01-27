//! # 婚恋模块 - 匹配算法
//!
//! 本模块提供八字合婚和性格匹配算法。
//!
//! ## 功能概述
//!
//! - **八字合婚**：日柱天干地支分析、五行互补分析
//! - **性格匹配**：互补性格、冲突性格、共同优点
//! - **合婚请求管理**：创建、授权、生成报告
//!
//! ## 算法权重
//!
//! | 维度 | 权重 |
//! |------|------|
//! | 日柱合婚 | 30% |
//! | 五行互补 | 25% |
//! | 性格匹配 | 20% |
//! | 神煞分析 | 15% |
//! | 大运配合 | 10% |

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

pub mod bazi;
pub mod personality;

#[cfg(test)]
mod tests;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::traits::Saturating;

use pallet_matchmaking_common::{
    MatchStatus, MatchRecommendation, CompatibilityScoreDetail,
};
use bazi::{calculate_day_pillar_compatibility, calculate_wuxing_compatibility, WuxingCompatibilityResult};
use personality::{calculate_personality_compatibility, calculate_default_personality_score};

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 八字数据提供者
        type BaziProvider: BaziDataProvider<Self::AccountId>;

        /// 每个用户最大请求数
        #[pallet::constant]
        type MaxRequestsPerUser: Get<u32>;

        /// 请求过期时间（区块数）
        #[pallet::constant]
        type RequestExpiration: Get<BlockNumberFor<Self>>;

        /// 权重信息
        type WeightInfo: WeightInfo;
    }

    // ========================================================================
    // 类型定义
    // ========================================================================

    /// 合婚请求
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    #[scale_info(skip_type_params(T))]
    pub struct CompatibilityRequest<T: Config> {
        pub id: u64,
        pub party_a: T::AccountId,
        pub party_b: T::AccountId,
        pub party_a_bazi_id: u64,
        pub party_b_bazi_id: u64,
        pub status: MatchStatus,
        pub created_at: BlockNumberFor<T>,
        pub authorized_at: Option<BlockNumberFor<T>>,
    }

    /// 合婚报告
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    #[scale_info(skip_type_params(T))]
    pub struct CompatibilityReport<T: Config> {
        pub id: u64,
        pub request_id: u64,
        pub overall_score: u8,
        pub score_detail: CompatibilityScoreDetail,
        pub recommendation: MatchRecommendation,
        pub report_cid: Option<BoundedVec<u8, ConstU32<64>>>,
        pub generated_at: BlockNumberFor<T>,
        pub algorithm_version: u8,
    }

    // ========================================================================
    // 存储
    // ========================================================================

    /// 合婚请求存储
    #[pallet::storage]
    #[pallet::getter(fn requests)]
    pub type Requests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        CompatibilityRequest<T>,
    >;

    /// 合婚报告存储
    #[pallet::storage]
    #[pallet::getter(fn reports)]
    pub type Reports<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        CompatibilityReport<T>,
    >;

    /// 用户请求索引（甲方）
    #[pallet::storage]
    #[pallet::getter(fn user_requests_as_party_a)]
    pub type UserRequestsAsPartyA<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, T::MaxRequestsPerUser>,
        ValueQuery,
    >;

    /// 用户请求索引（乙方）
    #[pallet::storage]
    #[pallet::getter(fn user_requests_as_party_b)]
    pub type UserRequestsAsPartyB<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, T::MaxRequestsPerUser>,
        ValueQuery,
    >;

    /// 请求 ID 计数器
    #[pallet::storage]
    #[pallet::getter(fn next_request_id)]
    pub type NextRequestId<T: Config> = StorageValue<_, u64, ValueQuery>;

    // ========================================================================
    // 事件
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 合婚请求已创建
        RequestCreated {
            request_id: u64,
            party_a: T::AccountId,
            party_b: T::AccountId,
        },
        /// 合婚请求已授权
        RequestAuthorized {
            request_id: u64,
            party_b: T::AccountId,
        },
        /// 合婚请求已拒绝
        RequestRejected {
            request_id: u64,
            party_b: T::AccountId,
        },
        /// 合婚请求已取消
        RequestCancelled {
            request_id: u64,
            cancelled_by: T::AccountId,
        },
        /// 合婚报告已生成
        ReportGenerated {
            report_id: u64,
            request_id: u64,
            overall_score: u8,
            recommendation: MatchRecommendation,
        },
    }

    // ========================================================================
    // 错误
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 请求不存在
        RequestNotFound,
        /// 报告不存在
        ReportNotFound,
        /// 未授权
        NotAuthorized,
        /// 不是八字所有者
        NotBaziOwner,
        /// 八字不存在
        BaziNotFound,
        /// 请求已过期
        RequestExpired,
        /// 请求状态无效
        InvalidRequestStatus,
        /// 不能给自己创建请求
        CannotMatchSelf,
        /// 请求数已达上限
        TooManyRequests,
        /// 报告已存在
        ReportAlreadyExists,
    }

    // ========================================================================
    // Extrinsics
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建合婚请求
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_request())]
        pub fn create_request(
            origin: OriginFor<T>,
            party_b: T::AccountId,
            party_a_bazi_id: u64,
            party_b_bazi_id: u64,
        ) -> DispatchResult {
            let party_a = ensure_signed(origin)?;

            ensure!(party_a != party_b, Error::<T>::CannotMatchSelf);

            ensure!(
                T::BaziProvider::is_owner(&party_a, party_a_bazi_id),
                Error::<T>::NotBaziOwner
            );

            ensure!(
                T::BaziProvider::exists(party_a_bazi_id),
                Error::<T>::BaziNotFound
            );
            ensure!(
                T::BaziProvider::exists(party_b_bazi_id),
                Error::<T>::BaziNotFound
            );

            let mut party_a_requests = UserRequestsAsPartyA::<T>::get(&party_a);
            ensure!(
                party_a_requests.len() < T::MaxRequestsPerUser::get() as usize,
                Error::<T>::TooManyRequests
            );

            let request_id = NextRequestId::<T>::get();
            let current_block = frame_system::Pallet::<T>::block_number();

            let request = CompatibilityRequest {
                id: request_id,
                party_a: party_a.clone(),
                party_b: party_b.clone(),
                party_a_bazi_id,
                party_b_bazi_id,
                status: MatchStatus::PendingAuthorization,
                created_at: current_block,
                authorized_at: None,
            };

            Requests::<T>::insert(request_id, request);
            NextRequestId::<T>::put(request_id.saturating_add(1));

            party_a_requests
                .try_push(request_id)
                .map_err(|_| Error::<T>::TooManyRequests)?;
            UserRequestsAsPartyA::<T>::insert(&party_a, party_a_requests);

            let mut party_b_requests = UserRequestsAsPartyB::<T>::get(&party_b);
            let _ = party_b_requests.try_push(request_id);
            UserRequestsAsPartyB::<T>::insert(&party_b, party_b_requests);

            Self::deposit_event(Event::RequestCreated {
                request_id,
                party_a,
                party_b,
            });

            Ok(())
        }

        /// 授权合婚请求
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::authorize_request())]
        pub fn authorize_request(origin: OriginFor<T>, request_id: u64) -> DispatchResult {
            let party_b = ensure_signed(origin)?;

            Requests::<T>::try_mutate(request_id, |maybe_request| {
                let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;

                ensure!(request.party_b == party_b, Error::<T>::NotAuthorized);

                ensure!(
                    request.status == MatchStatus::PendingAuthorization,
                    Error::<T>::InvalidRequestStatus
                );

                ensure!(
                    T::BaziProvider::is_owner(&party_b, request.party_b_bazi_id),
                    Error::<T>::NotBaziOwner
                );

                let current_block = frame_system::Pallet::<T>::block_number();
                let expiration = request.created_at.saturating_add(T::RequestExpiration::get());
                ensure!(current_block <= expiration, Error::<T>::RequestExpired);

                request.status = MatchStatus::Authorized;
                request.authorized_at = Some(current_block);

                Self::deposit_event(Event::RequestAuthorized {
                    request_id,
                    party_b,
                });

                Ok(())
            })
        }

        /// 拒绝合婚请求
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::reject_request())]
        pub fn reject_request(origin: OriginFor<T>, request_id: u64) -> DispatchResult {
            let party_b = ensure_signed(origin)?;

            Requests::<T>::try_mutate(request_id, |maybe_request| {
                let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;

                ensure!(request.party_b == party_b, Error::<T>::NotAuthorized);

                ensure!(
                    request.status == MatchStatus::PendingAuthorization,
                    Error::<T>::InvalidRequestStatus
                );

                request.status = MatchStatus::Rejected;

                Self::deposit_event(Event::RequestRejected {
                    request_id,
                    party_b,
                });

                Ok(())
            })
        }

        /// 取消合婚请求
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::cancel_request())]
        pub fn cancel_request(origin: OriginFor<T>, request_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Requests::<T>::try_mutate(request_id, |maybe_request| {
                let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;

                ensure!(request.party_a == who, Error::<T>::NotAuthorized);

                ensure!(
                    request.status == MatchStatus::PendingAuthorization
                        || request.status == MatchStatus::Authorized,
                    Error::<T>::InvalidRequestStatus
                );

                request.status = MatchStatus::Cancelled;

                Self::deposit_event(Event::RequestCancelled {
                    request_id,
                    cancelled_by: who,
                });

                Ok(())
            })
        }

        /// 生成合婚报告
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::generate_report())]
        pub fn generate_report(origin: OriginFor<T>, request_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let request = Requests::<T>::get(request_id).ok_or(Error::<T>::RequestNotFound)?;

            ensure!(
                who == request.party_a || who == request.party_b,
                Error::<T>::NotAuthorized
            );

            ensure!(
                request.status == MatchStatus::Authorized,
                Error::<T>::InvalidRequestStatus
            );

            ensure!(
                !Reports::<T>::contains_key(request_id),
                Error::<T>::ReportAlreadyExists
            );

            // 获取八字数据
            let bazi_a = T::BaziProvider::get_sizhu_index(request.party_a_bazi_id)
                .ok_or(Error::<T>::BaziNotFound)?;
            let bazi_b = T::BaziProvider::get_sizhu_index(request.party_b_bazi_id)
                .ok_or(Error::<T>::BaziNotFound)?;

            // 计算日柱合婚评分
            let day_ganzhi_a = bazi_a.day_ganzhi();
            let day_ganzhi_b = bazi_b.day_ganzhi();
            let day_pillar_result = calculate_day_pillar_compatibility(day_ganzhi_a, day_ganzhi_b);

            // 计算五行互补评分
            let wuxing_result = if let (Some(interp_a), Some(interp_b)) = (
                T::BaziProvider::get_interpretation(request.party_a_bazi_id),
                T::BaziProvider::get_interpretation(request.party_b_bazi_id),
            ) {
                calculate_wuxing_compatibility(&interp_a, &interp_b)
            } else {
                WuxingCompatibilityResult {
                    yongshen_score: 50,
                    jishen_score: 70,
                    balance_score: 50,
                    overall: 55,
                }
            };

            // 计算性格匹配评分
            let personality_result = if let (Some(xingge_a), Some(xingge_b)) = (
                T::BaziProvider::get_personality(request.party_a_bazi_id),
                T::BaziProvider::get_personality(request.party_b_bazi_id),
            ) {
                calculate_personality_compatibility(&xingge_a, &xingge_b)
            } else {
                calculate_default_personality_score()
            };

            // 神煞和大运评分（预留）
            let shensha_score = 60u8;
            let dayun_score = 60u8;

            let score_detail = CompatibilityScoreDetail {
                day_pillar_score: day_pillar_result.overall,
                wuxing_score: wuxing_result.overall,
                personality_score: personality_result.overall,
                shensha_score,
                dayun_score,
            };

            let overall_score = score_detail.calculate_overall();
            let recommendation = MatchRecommendation::from_score(overall_score);

            let current_block = frame_system::Pallet::<T>::block_number();
            let report = CompatibilityReport {
                id: request_id,
                request_id,
                overall_score,
                score_detail,
                recommendation,
                report_cid: None,
                generated_at: current_block,
                algorithm_version: 1,
            };

            Reports::<T>::insert(request_id, report);

            Requests::<T>::mutate(request_id, |maybe_request| {
                if let Some(req) = maybe_request {
                    req.status = MatchStatus::Completed;
                }
            });

            Self::deposit_event(Event::ReportGenerated {
                report_id: request_id,
                request_id,
                overall_score,
                recommendation,
            });

            Ok(())
        }
    }
}

// ============================================================================
// Trait 定义
// ============================================================================

/// 八字数据提供者 Trait
pub trait BaziDataProvider<AccountId> {
    fn exists(bazi_id: u64) -> bool;
    fn is_owner(account: &AccountId, bazi_id: u64) -> bool;
    fn get_sizhu_index(bazi_id: u64) -> Option<pallet_bazi_chart::types::SiZhuIndex>;
    fn get_interpretation(bazi_id: u64) -> Option<pallet_bazi_chart::interpretation::CoreInterpretation>;
    fn get_personality(bazi_id: u64) -> Option<pallet_bazi_chart::interpretation::CompactXingGe>;
}

// WeightInfo trait 和实现已移至 weights.rs
