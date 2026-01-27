//! # 玄学占卜 NFT Pallet
//!
//! 本模块实现通用的占卜结果 NFT 功能，支持多种玄学系统：
//! - 梅花易数卦象 NFT
//! - 八字命盘 NFT
//! - 六爻占卜 NFT（预留）
//! - 奇门遁甲 NFT（预留）
//!
//! ## 核心功能
//!
//! 1. **NFT 铸造**: 基于占卜结果自动判定稀有度，支持元数据配置
//! 2. **交易市场**: 定价挂单、议价出价、安全交易
//! 3. **收藏展示**: 个人收藏集、公开展示
//! 4. **版税机制**: 创作者在每次转售时获得版税
//!
//! ## 架构说明
//!
//! 本模块通过 `DivinationProvider` trait 与各玄学核心模块解耦：
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   pallet-divination-nft                 │
//! │    (通用 NFT 铸造、交易、收藏功能)                        │
//! └──────────────────────────┬──────────────────────────────┘
//!                            │ DivinationProvider trait
//!                            ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │              Runtime: CombinedDivinationProvider        │
//! └───────┬─────────────────────────────────┬───────────────┘
//!         │                                 │
//!         ▼                                 ▼
//! ┌───────────────┐                 ┌───────────────┐
//! │ pallet-meihua │                 │ pallet-bazi   │
//! │   (梅花易数)   │                 │   (八字排盘)   │
//! └───────────────┘                 └───────────────┘
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use crate::types::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use pallet_divination_common::{DivinationProvider, DivinationType, Rarity};
    use sp_runtime::traits::{Saturating, Zero};
    use sp_std::prelude::*;

    /// Pallet 配置
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// 货币类型
        type NftCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 占卜结果查询接口
        ///
        /// 各玄学系统（梅花、八字等）需实现此 trait，
        /// 在 Runtime 中组合为统一的 Provider。
        type DivinationProvider: DivinationProvider<Self::AccountId>;

        /// IPFS 内容注册接口（用于自动 Pin NFT 媒体）
        type ContentRegistry: pallet_storage_service::ContentRegistry;

        /// 最大名称长度
        #[pallet::constant]
        type MaxNameLength: Get<u32>;

        /// 最大 CID 长度
        #[pallet::constant]
        type MaxCidLength: Get<u32>;

        /// 每个用户最大收藏集数量
        #[pallet::constant]
        type MaxCollectionsPerUser: Get<u32>;

        /// 每个收藏集最大 NFT 数量
        #[pallet::constant]
        type MaxNftsPerCollection: Get<u32>;

        /// 每个 NFT 最大出价数量
        #[pallet::constant]
        type MaxOffersPerNft: Get<u32>;

        /// 基础铸造费用
        #[pallet::constant]
        type BaseMintFee: Get<BalanceOf<Self>>;

        /// 平台交易手续费率（万分比）
        #[pallet::constant]
        type PlatformFeeRate: Get<u16>;

        /// 最大版税比例（万分比）
        #[pallet::constant]
        type MaxRoyaltyRate: Get<u16>;

        /// 出价有效期（区块数）
        #[pallet::constant]
        type OfferValidityPeriod: Get<BlockNumberFor<Self>>;

        /// 平台收款账户
        #[pallet::constant]
        type PlatformAccount: Get<Self::AccountId>;

        /// 治理权限来源
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::NftCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// NFT 类型别名
    pub type DivinationNftOf<T> = DivinationNft<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
        <T as Config>::MaxNameLength,
    >;

    /// 挂单类型别名
    pub type ListingOf<T> = Listing<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;

    /// 出价类型别名
    pub type OfferOf<T> = Offer<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;

    /// 收藏集类型别名
    pub type CollectionOf<T> = Collection<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        <T as Config>::MaxNameLength,
        <T as Config>::MaxCidLength,
    >;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 下一个 NFT ID
    #[pallet::storage]
    #[pallet::getter(fn next_nft_id)]
    pub type NextNftId<T> = StorageValue<_, u64, ValueQuery>;

    /// 下一个出价 ID
    #[pallet::storage]
    #[pallet::getter(fn next_offer_id)]
    pub type NextOfferId<T> = StorageValue<_, u64, ValueQuery>;

    /// 下一个收藏集 ID
    #[pallet::storage]
    #[pallet::getter(fn next_collection_id)]
    pub type NextCollectionId<T> = StorageValue<_, u32, ValueQuery>;

    /// NFT 存储
    #[pallet::storage]
    #[pallet::getter(fn nfts)]
    pub type Nfts<T: Config> = StorageMap<_, Blake2_128Concat, u64, DivinationNftOf<T>>;

    /// 占卜结果 -> NFT 映射（确保每个结果只能铸造一个 NFT）
    ///
    /// 使用 (DivinationType, result_id) 作为复合键
    #[pallet::storage]
    #[pallet::getter(fn result_nft)]
    pub type ResultNftMapping<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        DivinationType,
        Blake2_128Concat,
        u64,
        u64,
        OptionQuery,
    >;

    /// 用户拥有的 NFT 列表
    /// 
    /// 🆕 存储膨胀防护：上限从 10000 → 1000 → 200
    /// 超出部分建议通过链下索引服务查询
    #[pallet::storage]
    #[pallet::getter(fn user_nfts)]
    pub type UserNfts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<200>>,
        ValueQuery,
    >;

    /// 挂单存储
    #[pallet::storage]
    #[pallet::getter(fn listings)]
    pub type Listings<T: Config> = StorageMap<_, Blake2_128Concat, u64, ListingOf<T>>;

    /// 出价存储
    #[pallet::storage]
    #[pallet::getter(fn offers)]
    pub type Offers<T: Config> = StorageMap<_, Blake2_128Concat, u64, OfferOf<T>>;

    /// NFT 的出价列表
    #[pallet::storage]
    #[pallet::getter(fn nft_offers)]
    pub type NftOffers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<u64, T::MaxOffersPerNft>,
        ValueQuery,
    >;

    /// 收藏集存储
    #[pallet::storage]
    #[pallet::getter(fn collections)]
    pub type Collections<T: Config> = StorageMap<_, Blake2_128Concat, u32, CollectionOf<T>>;

    /// 用户的收藏集列表
    #[pallet::storage]
    #[pallet::getter(fn user_collections)]
    pub type UserCollections<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u32, T::MaxCollectionsPerUser>,
        ValueQuery,
    >;

    /// 收藏集内的 NFT 列表
    #[pallet::storage]
    #[pallet::getter(fn collection_nfts)]
    pub type CollectionNfts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        BoundedVec<u64, T::MaxNftsPerCollection>,
        ValueQuery,
    >;

    /// NFT 全局统计数据
    #[pallet::storage]
    #[pallet::getter(fn nft_stats)]
    pub type NftStatistics<T: Config> = StorageValue<_, NftStats<BalanceOf<T>>, ValueQuery>;

    /// 按占卜类型的统计数据
    #[pallet::storage]
    #[pallet::getter(fn type_stats)]
    pub type TypeStatistics<T: Config> =
        StorageMap<_, Blake2_128Concat, DivinationType, TypeStats, ValueQuery>;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// NFT 已铸造
        NftMinted {
            nft_id: u64,
            divination_type: DivinationType,
            result_id: u64,
            owner: T::AccountId,
            rarity: Rarity,
            mint_fee: BalanceOf<T>,
        },

        /// NFT 已转移
        NftTransferred {
            nft_id: u64,
            from: T::AccountId,
            to: T::AccountId,
        },

        /// NFT 已销毁
        NftBurned { nft_id: u64, owner: T::AccountId },

        /// NFT 已挂单
        NftListed {
            nft_id: u64,
            seller: T::AccountId,
            price: BalanceOf<T>,
        },

        /// 挂单已取消
        ListingCancelled { nft_id: u64 },

        /// NFT 已售出
        NftSold {
            nft_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            price: BalanceOf<T>,
            royalty: BalanceOf<T>,
            platform_fee: BalanceOf<T>,
        },

        /// 出价已提交
        OfferMade {
            offer_id: u64,
            nft_id: u64,
            bidder: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 出价已取消
        OfferCancelled { offer_id: u64 },

        /// 出价已接受
        OfferAccepted {
            offer_id: u64,
            nft_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 收藏集已创建
        CollectionCreated {
            collection_id: u32,
            creator: T::AccountId,
        },

        /// NFT 已添加到收藏集
        NftAddedToCollection { nft_id: u64, collection_id: u32 },

        /// NFT 已从收藏集移除
        NftRemovedFromCollection { nft_id: u64, collection_id: u32 },

        /// 收藏集已删除
        CollectionDeleted {
            collection_id: u32,
            owner: T::AccountId,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 占卜结果不存在
        DivinationResultNotFound,
        /// 不是占卜结果所有者
        NotResultOwner,
        /// 占卜结果已铸造过 NFT
        ResultAlreadyMinted,
        /// 占卜结果不可铸造
        ResultNotMintable,
        /// 稀有度数据不可用
        RarityDataNotAvailable,
        /// NFT 不存在
        NftNotFound,
        /// 不是 NFT 所有者
        NotNftOwner,
        /// NFT 状态无效
        InvalidNftStatus,
        /// NFT 已挂单
        NftAlreadyListed,
        /// NFT 未挂单
        NftNotListed,
        /// 挂单不存在
        ListingNotFound,
        /// 出价不存在
        OfferNotFound,
        /// 出价已过期
        OfferExpired,
        /// 出价无效
        InvalidOffer,
        /// 余额不足
        InsufficientBalance,
        /// 名称过长
        NameTooLong,
        /// CID 过长
        CidTooLong,
        /// 收藏集不存在
        CollectionNotFound,
        /// 不是收藏集所有者
        NotCollectionOwner,
        /// 收藏集已满
        CollectionFull,
        /// 收藏集数量已达上限
        TooManyCollections,
        /// NFT 不在收藏集中
        NftNotInCollection,
        /// 版税比例过高
        RoyaltyTooHigh,
        /// 出价列表已满
        TooManyOffers,
        /// NFT 列表已满
        NftListFull,
        /// 不能购买自己的 NFT
        CannotBuySelfNft,
        /// 稀有度供应已达上限
        RaritySupplyExceeded,
        /// 收藏集不为空（需要先移除所有 NFT）
        CollectionNotEmpty,
    }

    // ==================== 可调用函数 ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 铸造 NFT
        ///
        /// 将占卜结果（卦象、命盘等）转化为 NFT，自动判定稀有度。
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型（梅花、八字等）
        /// - `result_id`: 占卜结果 ID（卦象 ID、命盘 ID 等）
        /// - `name`: NFT 名称
        /// - `image_cid`: 图片 IPFS CID
        /// - `description_cid`: 描述 IPFS CID（可选）
        /// - `animation_cid`: 动画 IPFS CID（可选）
        /// - `royalty_rate`: 版税比例（万分比）
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(80_000_000, 0))]
        pub fn mint_nft(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            name: Vec<u8>,
            image_cid: Vec<u8>,
            description_cid: Option<Vec<u8>>,
            animation_cid: Option<Vec<u8>>,
            royalty_rate: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证占卜结果存在
            ensure!(
                T::DivinationProvider::result_exists(divination_type, result_id),
                Error::<T>::DivinationResultNotFound
            );

            // 验证调用者是结果创建者
            let creator = T::DivinationProvider::result_creator(divination_type, result_id)
                .ok_or(Error::<T>::DivinationResultNotFound)?;
            ensure!(creator == who, Error::<T>::NotResultOwner);

            // 验证结果可以铸造
            ensure!(
                T::DivinationProvider::is_nftable(divination_type, result_id),
                Error::<T>::ResultNotMintable
            );

            // 确保未被铸造过
            ensure!(
                !ResultNftMapping::<T>::contains_key(divination_type, result_id),
                Error::<T>::ResultAlreadyMinted
            );

            // 验证版税比例
            ensure!(
                royalty_rate <= T::MaxRoyaltyRate::get(),
                Error::<T>::RoyaltyTooHigh
            );

            // 获取稀有度数据并计算
            let rarity_input = T::DivinationProvider::rarity_data(divination_type, result_id)
                .ok_or(Error::<T>::RarityDataNotAvailable)?;
            let rarity = rarity_input.calculate_rarity();

            // 检查稀有度供应
            let stats = NftStatistics::<T>::get();
            if let Some(max) = Self::max_supply_for_rarity(&rarity) {
                let current = match rarity {
                    Rarity::Common => stats.common_count,
                    Rarity::Rare => stats.rare_count,
                    Rarity::Epic => stats.epic_count,
                    Rarity::Legendary => stats.legendary_count,
                };
                ensure!(current < max, Error::<T>::RaritySupplyExceeded);
            }

            // 计算铸造费用
            let base_fee = T::BaseMintFee::get();
            let multiplier = rarity.fee_multiplier();
            let mint_fee = base_fee.saturating_mul(multiplier.into()) / 100u32.into();

            // 扣除铸造费用
            T::NftCurrency::transfer(
                &who,
                &T::PlatformAccount::get(),
                mint_fee,
                ExistenceRequirement::KeepAlive,
            )?;

            // 构建元数据
            let name_bounded: BoundedVec<u8, T::MaxNameLength> =
                BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;
            let image_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(image_cid.clone()).map_err(|_| Error::<T>::CidTooLong)?;
            let description_cid_bounded = description_cid
                .clone()
                .map(|cid| BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;
            let animation_cid_bounded = animation_cid
                .clone()
                .map(|cid| BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            // 创建 NFT
            let nft_id = NextNftId::<T>::get();
            NextNftId::<T>::put(nft_id.saturating_add(1));

            // 🆕 自动 Pin NFT 主图到 IPFS (Critical 层级 - 5副本)
            // NFT 主图是高价值数字资产，需要最高可靠性
            <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                b"divination-nft".to_vec(),
                nft_id,
                image_cid,
                pallet_storage_service::PinTier::Critical,
            )?;

            // 🆕 如果有描述，Pin 描述 (Standard 层级)
            if let Some(ref desc_cid) = description_cid {
                // 描述 Pin 失败不影响主流程
                let _ = <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                    b"divination-nft".to_vec(),
                    nft_id.saturating_add(1000000),
                    desc_cid.clone(),
                    pallet_storage_service::PinTier::Standard,
                ).ok();
            }

            // 🆕 如果有动画，Pin 动画 (Standard 层级)
            if let Some(ref anim_cid) = animation_cid {
                // 动画 Pin 失败不影响主流程
                let _ = <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                    b"divination-nft".to_vec(),
                    nft_id.saturating_add(2000000),
                    anim_cid.clone(),
                    pallet_storage_service::PinTier::Standard,
                ).ok();
            }

            let metadata = NftMetadata {
                name: name_bounded,
                description_cid: description_cid_bounded,
                image_cid: image_cid_bounded,
                animation_cid: animation_cid_bounded,
                external_url_cid: None,
            };

            let block_number = <frame_system::Pallet<T>>::block_number();

            let nft = DivinationNft {
                id: nft_id,
                divination_type,
                result_id,
                owner: who.clone(),
                creator: who.clone(),
                rarity,
                status: NftStatus::Normal,
                metadata,
                minted_at: block_number,
                mint_fee,
                royalty_rate,
                transfer_count: 0,
            };

            // 存储
            Nfts::<T>::insert(nft_id, nft);
            ResultNftMapping::<T>::insert(divination_type, result_id, nft_id);

            UserNfts::<T>::try_mutate(&who, |list| {
                list.try_push(nft_id).map_err(|_| Error::<T>::NftListFull)
            })?;

            // 通知占卜模块已铸造
            T::DivinationProvider::mark_as_nfted(divination_type, result_id);

            // 更新统计
            NftStatistics::<T>::mutate(|s| {
                s.total_minted += 1;
                match rarity {
                    Rarity::Common => s.common_count += 1,
                    Rarity::Rare => s.rare_count += 1,
                    Rarity::Epic => s.epic_count += 1,
                    Rarity::Legendary => s.legendary_count += 1,
                }
            });

            TypeStatistics::<T>::mutate(divination_type, |s| {
                s.minted_count += 1;
            });

            Self::deposit_event(Event::NftMinted {
                nft_id,
                divination_type,
                result_id,
                owner: who,
                rarity,
                mint_fee,
            });

            Ok(())
        }

        /// 转移 NFT
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn transfer_nft(
            origin: OriginFor<T>,
            nft_id: u64,
            to: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Nfts::<T>::try_mutate(nft_id, |maybe_nft| {
                let nft = maybe_nft.as_mut().ok_or(Error::<T>::NftNotFound)?;
                ensure!(nft.owner == who, Error::<T>::NotNftOwner);
                ensure!(nft.status == NftStatus::Normal, Error::<T>::InvalidNftStatus);

                let from = nft.owner.clone();
                nft.owner = to.clone();
                nft.transfer_count += 1;

                // 更新用户 NFT 列表
                UserNfts::<T>::mutate(&from, |list| {
                    list.retain(|&id| id != nft_id);
                });
                UserNfts::<T>::try_mutate(&to, |list| {
                    list.try_push(nft_id).map_err(|_| Error::<T>::NftListFull)
                })?;

                Self::deposit_event(Event::NftTransferred { nft_id, from, to });

                Ok(())
            })
        }

        /// 销毁 NFT
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn burn_nft(origin: OriginFor<T>, nft_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Nfts::<T>::try_mutate(nft_id, |maybe_nft| {
                let nft = maybe_nft.as_mut().ok_or(Error::<T>::NftNotFound)?;
                ensure!(nft.owner == who, Error::<T>::NotNftOwner);
                ensure!(nft.status == NftStatus::Normal, Error::<T>::InvalidNftStatus);

                let divination_type = nft.divination_type;
                let result_id = nft.result_id;

                nft.status = NftStatus::Burned;

                // 从用户列表移除
                UserNfts::<T>::mutate(&who, |list| {
                    list.retain(|&id| id != nft_id);
                });

                // 移除结果映射
                ResultNftMapping::<T>::remove(divination_type, result_id);

                // 更新统计
                NftStatistics::<T>::mutate(|s| {
                    s.total_burned += 1;
                });

                TypeStatistics::<T>::mutate(divination_type, |s| {
                    s.burned_count += 1;
                });

                Self::deposit_event(Event::NftBurned { nft_id, owner: who });

                Ok(())
            })
        }

        /// 挂单出售 NFT
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn list_nft(
            origin: OriginFor<T>,
            nft_id: u64,
            price: BalanceOf<T>,
            expires_in: Option<BlockNumberFor<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Nfts::<T>::try_mutate(nft_id, |maybe_nft| {
                let nft = maybe_nft.as_mut().ok_or(Error::<T>::NftNotFound)?;
                ensure!(nft.owner == who, Error::<T>::NotNftOwner);
                ensure!(nft.status == NftStatus::Normal, Error::<T>::InvalidNftStatus);

                nft.status = NftStatus::Listed;

                Ok::<_, DispatchError>(())
            })?;

            let block_number = <frame_system::Pallet<T>>::block_number();
            let expires_at = expires_in.map(|blocks| block_number.saturating_add(blocks));

            let listing = Listing {
                nft_id,
                seller: who.clone(),
                price,
                listed_at: block_number,
                expires_at,
            };

            Listings::<T>::insert(nft_id, listing);

            // 更新统计
            NftStatistics::<T>::mutate(|s| {
                s.active_listings += 1;
            });

            Self::deposit_event(Event::NftListed {
                nft_id,
                seller: who,
                price,
            });

            Ok(())
        }

        /// 取消挂单
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn cancel_listing(origin: OriginFor<T>, nft_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let listing = Listings::<T>::get(nft_id).ok_or(Error::<T>::ListingNotFound)?;
            ensure!(listing.seller == who, Error::<T>::NotNftOwner);

            // 恢复 NFT 状态
            Nfts::<T>::mutate(nft_id, |maybe_nft| {
                if let Some(nft) = maybe_nft {
                    nft.status = NftStatus::Normal;
                }
            });

            Listings::<T>::remove(nft_id);

            // 更新统计
            NftStatistics::<T>::mutate(|s| {
                s.active_listings = s.active_listings.saturating_sub(1);
            });

            Self::deposit_event(Event::ListingCancelled { nft_id });

            Ok(())
        }

        /// 购买挂单的 NFT
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(80_000_000, 0))]
        pub fn buy_nft(origin: OriginFor<T>, nft_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let listing = Listings::<T>::get(nft_id).ok_or(Error::<T>::ListingNotFound)?;
            ensure!(listing.seller != who, Error::<T>::CannotBuySelfNft);

            // 检查过期
            if let Some(expires_at) = listing.expires_at {
                let current = <frame_system::Pallet<T>>::block_number();
                ensure!(current <= expires_at, Error::<T>::OfferExpired);
            }

            let nft = Nfts::<T>::get(nft_id).ok_or(Error::<T>::NftNotFound)?;

            // 计算费用分配
            let platform_fee =
                listing.price.saturating_mul(T::PlatformFeeRate::get().into()) / 10000u32.into();
            let royalty = if nft.creator != listing.seller {
                listing.price.saturating_mul(nft.royalty_rate.into()) / 10000u32.into()
            } else {
                Zero::zero()
            };
            let seller_amount = listing
                .price
                .saturating_sub(platform_fee)
                .saturating_sub(royalty);

            // 转账
            T::NftCurrency::transfer(
                &who,
                &listing.seller,
                seller_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            if !platform_fee.is_zero() {
                T::NftCurrency::transfer(
                    &who,
                    &T::PlatformAccount::get(),
                    platform_fee,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            if !royalty.is_zero() {
                T::NftCurrency::transfer(
                    &who,
                    &nft.creator,
                    royalty,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            let divination_type = nft.divination_type;

            // 转移 NFT
            Nfts::<T>::mutate(nft_id, |maybe_nft| {
                if let Some(n) = maybe_nft {
                    n.owner = who.clone();
                    n.status = NftStatus::Normal;
                    n.transfer_count += 1;
                }
            });

            // 更新用户列表
            UserNfts::<T>::mutate(&listing.seller, |list| {
                list.retain(|&id| id != nft_id);
            });
            UserNfts::<T>::try_mutate(&who, |list| {
                list.try_push(nft_id).map_err(|_| Error::<T>::NftListFull)
            })?;

            // 移除挂单
            Listings::<T>::remove(nft_id);

            // 更新统计
            NftStatistics::<T>::mutate(|s| {
                s.total_trades += 1;
                s.total_volume = s.total_volume.saturating_add(listing.price);
                s.active_listings = s.active_listings.saturating_sub(1);
            });

            TypeStatistics::<T>::mutate(divination_type, |s| {
                s.trade_count += 1;
            });

            Self::deposit_event(Event::NftSold {
                nft_id,
                seller: listing.seller,
                buyer: who,
                price: listing.price,
                royalty,
                platform_fee,
            });

            Ok(())
        }

        /// 提交出价
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn make_offer(
            origin: OriginFor<T>,
            nft_id: u64,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let nft = Nfts::<T>::get(nft_id).ok_or(Error::<T>::NftNotFound)?;
            ensure!(nft.owner != who, Error::<T>::CannotBuySelfNft);
            ensure!(nft.status != NftStatus::Burned, Error::<T>::InvalidNftStatus);

            // 锁定出价金额
            T::NftCurrency::reserve(&who, amount)?;

            let offer_id = NextOfferId::<T>::get();
            NextOfferId::<T>::put(offer_id.saturating_add(1));

            let block_number = <frame_system::Pallet<T>>::block_number();
            let expires_at = block_number.saturating_add(T::OfferValidityPeriod::get());

            let offer = Offer {
                id: offer_id,
                nft_id,
                bidder: who.clone(),
                amount,
                offered_at: block_number,
                expires_at,
                is_active: true,
            };

            Offers::<T>::insert(offer_id, offer);

            NftOffers::<T>::try_mutate(nft_id, |list| {
                list.try_push(offer_id).map_err(|_| Error::<T>::TooManyOffers)
            })?;

            Self::deposit_event(Event::OfferMade {
                offer_id,
                nft_id,
                bidder: who,
                amount,
            });

            Ok(())
        }

        /// 取消出价
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn cancel_offer(origin: OriginFor<T>, offer_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Offers::<T>::try_mutate(offer_id, |maybe_offer| {
                let offer = maybe_offer.as_mut().ok_or(Error::<T>::OfferNotFound)?;
                ensure!(offer.bidder == who, Error::<T>::InvalidOffer);
                ensure!(offer.is_active, Error::<T>::InvalidOffer);

                // 解锁金额
                T::NftCurrency::unreserve(&who, offer.amount);

                offer.is_active = false;

                // 从 NFT 出价列表移除
                NftOffers::<T>::mutate(offer.nft_id, |list| {
                    list.retain(|&id| id != offer_id);
                });

                Self::deposit_event(Event::OfferCancelled { offer_id });

                Ok(())
            })
        }

        /// 接受出价
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(80_000_000, 0))]
        pub fn accept_offer(origin: OriginFor<T>, offer_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let offer = Offers::<T>::get(offer_id).ok_or(Error::<T>::OfferNotFound)?;
            ensure!(offer.is_active, Error::<T>::InvalidOffer);

            // 检查过期
            let current = <frame_system::Pallet<T>>::block_number();
            ensure!(current <= offer.expires_at, Error::<T>::OfferExpired);

            let nft = Nfts::<T>::get(offer.nft_id).ok_or(Error::<T>::NftNotFound)?;
            ensure!(nft.owner == who, Error::<T>::NotNftOwner);
            ensure!(
                nft.status == NftStatus::Normal || nft.status == NftStatus::Listed,
                Error::<T>::InvalidNftStatus
            );

            // 解锁出价金额
            T::NftCurrency::unreserve(&offer.bidder, offer.amount);

            // 计算费用分配
            let platform_fee =
                offer.amount.saturating_mul(T::PlatformFeeRate::get().into()) / 10000u32.into();
            let royalty = if nft.creator != who {
                offer.amount.saturating_mul(nft.royalty_rate.into()) / 10000u32.into()
            } else {
                Zero::zero()
            };
            let seller_amount = offer
                .amount
                .saturating_sub(platform_fee)
                .saturating_sub(royalty);

            // 转账
            T::NftCurrency::transfer(
                &offer.bidder,
                &who,
                seller_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            if !platform_fee.is_zero() {
                T::NftCurrency::transfer(
                    &offer.bidder,
                    &T::PlatformAccount::get(),
                    platform_fee,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            if !royalty.is_zero() {
                T::NftCurrency::transfer(
                    &offer.bidder,
                    &nft.creator,
                    royalty,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            let divination_type = nft.divination_type;

            // 转移 NFT
            Nfts::<T>::mutate(offer.nft_id, |maybe_nft| {
                if let Some(n) = maybe_nft {
                    n.owner = offer.bidder.clone();
                    n.status = NftStatus::Normal;
                    n.transfer_count += 1;
                }
            });

            // 更新用户列表
            UserNfts::<T>::mutate(&who, |list| {
                list.retain(|&id| id != offer.nft_id);
            });
            UserNfts::<T>::try_mutate(&offer.bidder, |list| {
                list.try_push(offer.nft_id).map_err(|_| Error::<T>::NftListFull)
            })?;

            // 如果有挂单，移除
            if Listings::<T>::contains_key(offer.nft_id) {
                Listings::<T>::remove(offer.nft_id);
                NftStatistics::<T>::mutate(|s| {
                    s.active_listings = s.active_listings.saturating_sub(1);
                });
            }

            // 标记出价为无效
            Offers::<T>::mutate(offer_id, |maybe_offer| {
                if let Some(o) = maybe_offer {
                    o.is_active = false;
                }
            });

            // 从 NFT 出价列表移除
            NftOffers::<T>::mutate(offer.nft_id, |list| {
                list.retain(|&id| id != offer_id);
            });

            // 更新统计
            NftStatistics::<T>::mutate(|s| {
                s.total_trades += 1;
                s.total_volume = s.total_volume.saturating_add(offer.amount);
            });

            TypeStatistics::<T>::mutate(divination_type, |s| {
                s.trade_count += 1;
            });

            Self::deposit_event(Event::OfferAccepted {
                offer_id,
                nft_id: offer.nft_id,
                seller: who,
                buyer: offer.bidder,
                amount: offer.amount,
            });

            Ok(())
        }

        /// 创建收藏集
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn create_collection(
            origin: OriginFor<T>,
            name: Vec<u8>,
            description_cid: Option<Vec<u8>>,
            cover_cid: Option<Vec<u8>>,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查收藏集数量限制
            let user_collections = UserCollections::<T>::get(&who);
            ensure!(
                user_collections.len() < T::MaxCollectionsPerUser::get() as usize,
                Error::<T>::TooManyCollections
            );

            let name_bounded: BoundedVec<u8, T::MaxNameLength> =
                BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;
            let description_cid_bounded = description_cid
                .clone()
                .map(|cid| BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;
            let cover_cid_bounded = cover_cid
                .clone()
                .map(|cid| BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            let collection_id = NextCollectionId::<T>::get();
            NextCollectionId::<T>::put(collection_id.saturating_add(1));

            // 🆕 如果有描述，Pin 描述 (Temporary 层级)
            if let Some(ref desc_cid) = description_cid {
                // 描述 Pin 失败不影响主流程
                let _ = <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                    b"divination-nft".to_vec(),
                    (collection_id as u64).saturating_add(3000000),
                    desc_cid.clone(),
                    pallet_storage_service::PinTier::Temporary,
                ).ok();
            }

            // 🆕 如果有封面，Pin 封面 (Standard 层级)
            if let Some(ref cover) = cover_cid {
                // 封面 Pin 失败不影响主流程
                let _ = <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                    b"divination-nft".to_vec(),
                    (collection_id as u64).saturating_add(4000000),
                    cover.clone(),
                    pallet_storage_service::PinTier::Standard,
                ).ok();
            }

            let collection = Collection {
                id: collection_id,
                creator: who.clone(),
                name: name_bounded,
                description_cid: description_cid_bounded,
                cover_cid: cover_cid_bounded,
                nft_count: 0,
                created_at: <frame_system::Pallet<T>>::block_number(),
                is_public,
            };

            Collections::<T>::insert(collection_id, collection);

            UserCollections::<T>::try_mutate(&who, |list| {
                list.try_push(collection_id)
                    .map_err(|_| Error::<T>::TooManyCollections)
            })?;

            Self::deposit_event(Event::CollectionCreated {
                collection_id,
                creator: who,
            });

            Ok(())
        }

        /// 添加 NFT 到收藏集
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn add_to_collection(
            origin: OriginFor<T>,
            nft_id: u64,
            collection_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证 NFT 所有权
            let nft = Nfts::<T>::get(nft_id).ok_or(Error::<T>::NftNotFound)?;
            ensure!(nft.owner == who, Error::<T>::NotNftOwner);

            // 验证收藏集所有权
            let collection =
                Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;
            ensure!(collection.creator == who, Error::<T>::NotCollectionOwner);

            // 添加到收藏集
            CollectionNfts::<T>::try_mutate(collection_id, |list| {
                ensure!(!list.contains(&nft_id), Error::<T>::InvalidNftStatus);
                list.try_push(nft_id).map_err(|_| Error::<T>::CollectionFull)
            })?;

            // 更新收藏集 NFT 计数
            Collections::<T>::mutate(collection_id, |maybe_col| {
                if let Some(col) = maybe_col {
                    col.nft_count += 1;
                }
            });

            Self::deposit_event(Event::NftAddedToCollection {
                nft_id,
                collection_id,
            });

            Ok(())
        }

        /// 从收藏集移除 NFT
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn remove_from_collection(
            origin: OriginFor<T>,
            nft_id: u64,
            collection_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证收藏集所有权
            let collection =
                Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;
            ensure!(collection.creator == who, Error::<T>::NotCollectionOwner);

            // 从收藏集移除
            CollectionNfts::<T>::try_mutate(collection_id, |list| {
                let pos = list
                    .iter()
                    .position(|&id| id == nft_id)
                    .ok_or(Error::<T>::NftNotInCollection)?;
                list.remove(pos);
                Ok::<_, DispatchError>(())
            })?;

            // 更新收藏集 NFT 计数
            Collections::<T>::mutate(collection_id, |maybe_col| {
                if let Some(col) = maybe_col {
                    col.nft_count = col.nft_count.saturating_sub(1);
                }
            });

            Self::deposit_event(Event::NftRemovedFromCollection {
                nft_id,
                collection_id,
            });

            Ok(())
        }

        /// 删除收藏集
        ///
        /// 仅允许删除空收藏集（不包含任何 NFT）。
        /// 如需删除非空收藏集，请先调用 `remove_from_collection` 移除所有 NFT。
        ///
        /// # 参数
        /// - `collection_id`: 收藏集 ID
        ///
        /// # 错误
        /// - `CollectionNotFound`: 收藏集不存在
        /// - `NotCollectionOwner`: 不是收藏集所有者
        /// - `CollectionNotEmpty`: 收藏集不为空
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn delete_collection(
            origin: OriginFor<T>,
            collection_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证收藏集存在且为调用者所有
            let collection =
                Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;
            ensure!(collection.creator == who, Error::<T>::NotCollectionOwner);

            // 确保收藏集为空
            let nfts_in_collection = CollectionNfts::<T>::get(collection_id);
            ensure!(nfts_in_collection.is_empty(), Error::<T>::CollectionNotEmpty);

            // 从用户收藏集列表中移除
            UserCollections::<T>::mutate(&who, |list| {
                list.retain(|&id| id != collection_id);
            });

            // 移除收藏集中的 NFT 列表（应该已经为空，但确保清理）
            CollectionNfts::<T>::remove(collection_id);

            // 删除收藏集
            Collections::<T>::remove(collection_id);

            Self::deposit_event(Event::CollectionDeleted {
                collection_id,
                owner: who,
            });

            Ok(())
        }
    }

    // ==================== 辅助函数 ====================

    impl<T: Config> Pallet<T> {
        /// 获取稀有度对应的最大供应量
        fn max_supply_for_rarity(rarity: &Rarity) -> Option<u64> {
            match rarity {
                Rarity::Common => None,          // 无限制
                Rarity::Rare => Some(10000),     // 最多 10000 个
                Rarity::Epic => Some(1000),      // 最多 1000 个
                Rarity::Legendary => Some(100),  // 最多 100 个
            }
        }
    }
}
