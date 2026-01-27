//! # 婚恋模块 - 共享类型定义
//!
//! 定义婚恋系统所需的核心数据结构。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

// ============================================================================
// 性别
// ============================================================================

/// 性别
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum Gender {
    #[default]
    Male = 0,
    Female = 1,
    Other = 2,
}

// ============================================================================
// 隐私模式
// ============================================================================

/// 资料隐私模式
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum ProfilePrivacyMode {
    /// 公开 - 所有人可见
    #[default]
    Public = 0,
    /// 仅匹配可见 - 只有匹配成功的用户可见
    MatchOnly = 1,
    /// 完全私密 - 仅自己可见
    Private = 2,
}

// ============================================================================
// 教育水平
// ============================================================================

/// 教育水平
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum EducationLevel {
    /// 初中及以下
    JuniorHigh = 0,
    /// 高中/中专
    HighSchool = 1,
    /// 大专
    Associate = 2,
    /// 本科
    #[default]
    Bachelor = 3,
    /// 硕士
    Master = 4,
    /// 博士
    Doctorate = 5,
}

// ============================================================================
// 房产情况
// ============================================================================

/// 房产情况
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum PropertyStatus {
    /// 无房
    #[default]
    None = 0,
    /// 有房（有贷款）
    WithMortgage = 1,
    /// 有房（无贷款）
    Owned = 2,
    /// 多套房
    Multiple = 3,
}

// ============================================================================
// 车辆情况
// ============================================================================

/// 车辆情况
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum VehicleStatus {
    /// 无车
    #[default]
    None = 0,
    /// 有车
    Owned = 1,
    /// 多辆车
    Multiple = 2,
}

// ============================================================================
// 婚姻状况
// ============================================================================

/// 婚姻状况
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum MaritalStatus {
    /// 未婚
    #[default]
    Single = 0,
    /// 离异
    Divorced = 1,
    /// 丧偶
    Widowed = 2,
}

// ============================================================================
// 生活方式
// ============================================================================

/// 生活方式
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum Lifestyle {
    /// 居家型
    Homebody = 0,
    /// 社交型
    Social = 1,
    /// 运动型
    Athletic = 2,
    /// 文艺型
    Artistic = 3,
    /// 工作狂
    Workaholic = 4,
    /// 平衡型
    #[default]
    Balanced = 5,
}

// ============================================================================
// 可见性级别
// ============================================================================

/// 可见性级别
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum VisibilityLevel {
    /// 公开（所有人可见）
    #[default]
    Public = 0,
    /// 匹配可见（仅匹配用户可见）
    Matched = 1,
    /// 私密（完全不可见）
    Private = 2,
}

// ============================================================================
// 资料状态
// ============================================================================

/// 资料状态
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum ProfileStatus {
    /// 草稿（未完成）
    Draft = 0,
    /// 待审核
    Pending = 1,
    /// 已激活（正常显示）
    #[default]
    Active = 2,
    /// 已暂停（暂时隐藏）
    Paused = 3,
    /// 已封禁（违规）
    Banned = 4,
    /// 已暂停（违规处罚）
    Suspended = 5,
}

// ============================================================================
// 性格特征（用户自填）
// ============================================================================

/// 性格特征（用户自填，简化版）
/// 
/// 用于用户自我描述的性格标签，与八字解盘的性格分析互补
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum PersonalityTrait {
    /// 外向
    #[default]
    Extroverted = 0,
    /// 内向
    Introverted = 1,
    /// 理性
    Rational = 2,
    /// 感性
    Emotional = 3,
    /// 乐观
    Optimistic = 4,
    /// 谨慎
    Cautious = 5,
    /// 独立
    Independent = 6,
    /// 依赖
    Dependent = 7,
    /// 创造力
    Creative = 8,
    /// 务实
    Practical = 9,
    /// 领导力
    Leadership = 10,
    /// 合作
    Cooperative = 11,
    /// 温和
    Gentle = 12,
    /// 果断
    Decisive = 13,
    /// 细心
    Careful = 14,
    /// 大方
    Generous = 15,
    /// 幽默
    Humorous = 16,
    /// 稳重
    Steady = 17,
    /// 热情
    Passionate = 18,
    /// 善解人意
    Understanding = 19,
}

// ============================================================================
// 性格来源
// ============================================================================

/// 性格分析来源
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum PersonalitySource {
    /// 用户自填
    #[default]
    UserFilled = 0,
    /// 八字解盘
    BaziAnalysis = 1,
    /// 综合分析（自填+八字）
    Combined = 2,
}

// ============================================================================
// 八字性格特征（与 pallet-bazi-chart 对应）
// ============================================================================

/// 八字性格特征（与 pallet_bazi_chart::interpretation::XingGeTrait 对应）
/// 
/// 用于存储从八字解盘获取的性格分析结果
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum BaziPersonalityTrait {
    /// 正直
    #[default]
    ZhengZhi = 0,
    /// 有主见
    YouZhuJian = 1,
    /// 积极向上
    JiJiXiangShang = 2,
    /// 固执
    GuZhi = 3,
    /// 缺乏变通
    QueFaBianTong = 4,
    /// 温和
    WenHe = 5,
    /// 适应性强
    ShiYingXingQiang = 6,
    /// 有艺术天赋
    YouYiShuTianFu = 7,
    /// 优柔寡断
    YouRouGuaDuan = 8,
    /// 依赖性强
    YiLaiXingQiang = 9,
    /// 热情
    ReQing = 10,
    /// 开朗
    KaiLang = 11,
    /// 有领导力
    YouLingDaoLi = 12,
    /// 急躁
    JiZao = 13,
    /// 缺乏耐心
    QueFaNaiXin = 14,
    /// 细心
    XiXin = 15,
    /// 有创造力
    YouChuangZaoLi = 16,
    /// 善于沟通
    ShanYuGouTong = 17,
    /// 情绪化
    QingXuHua = 18,
    /// 敏感
    MinGan = 19,
    /// 稳重
    WenZhong = 20,
    /// 可靠
    KeLao = 21,
    /// 有责任心
    YouZeRenXin = 22,
    /// 保守
    BaoShou = 23,
    /// 变化慢
    BianHuaMan = 24,
    /// 包容
    BaoRong = 25,
    /// 细致
    XiZhi = 26,
    /// 善于协调
    ShanYuXieTiao = 27,
    /// 犹豫不决
    YouYuBuJue = 28,
    /// 缺乏魄力
    QueFaPoLi = 29,
    /// 果断
    GuoDuan = 30,
    /// 有正义感
    YouZhengYiGan = 31,
    /// 执行力强
    ZhiXingLiQiang = 32,
    /// 刚硬
    GangYing = 33,
    /// 不够圆滑
    BuGouYuanHua = 34,
    /// 精致
    JingZhi = 35,
    /// 有品味
    YouPinWei = 36,
    /// 善于表达
    ShanYuBiaoDa = 37,
    /// 挑剔
    TiaoTi = 38,
    /// 情绪波动大
    QingXuBoDongDa = 39,
    /// 智慧
    ZhiHui = 40,
    /// 灵活
    LingHuo = 41,
    /// 适应力强
    ShiYingLiQiang = 42,
    /// 多变
    DuoBian = 43,
    /// 缺乏恒心
    QueFaHengXin = 44,
    /// 内敛
    NeiLian = 45,
    /// 善于思考
    ShanYuSiKao = 46,
    /// 消极
    XiaoJi = 47,
    /// 缺乏自信
    QueFaZiXin = 48,
}

// ============================================================================
// 综合性格分析
// ============================================================================

/// 综合性格分析结果
/// 
/// 结合用户自填和八字解盘的性格分析
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(T))]
pub struct PersonalityAnalysis<T: PersonalityConfig> {
    // ========== 用户自填性格 ==========
    /// 用户自选的性格标签（最多 5 个）
    pub user_traits: BoundedVec<PersonalityTrait, T::MaxUserTraits>,
    /// 用户自我描述（可选）
    pub self_description: Option<BoundedVec<u8, T::MaxDescLen>>,
    
    // ========== 八字解盘性格 ==========
    /// 主要性格特点（来自八字解盘，最多 3 个）
    pub bazi_main_traits: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
    /// 性格优点（来自八字解盘，最多 3 个）
    pub bazi_strengths: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
    /// 性格缺点（来自八字解盘，最多 2 个）
    pub bazi_weaknesses: BoundedVec<BaziPersonalityTrait, ConstU32<2>>,
    
    // ========== 元数据 ==========
    /// 性格分析来源
    pub source: PersonalitySource,
    /// 八字命盘 ID（如果有八字分析）
    pub bazi_chart_id: Option<u64>,
    /// 分析更新时间（区块号）
    pub updated_at: u64,
}

/// 性格分析配置 trait
pub trait PersonalityConfig {
    /// 用户自选性格标签最大数量
    type MaxUserTraits: Get<u32>;
    /// 自我描述最大长度
    type MaxDescLen: Get<u32>;
}

// ============================================================================
// 互动类型
// ============================================================================

/// 互动类型
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum InteractionType {
    /// 点赞
    #[default]
    Like = 0,
    /// 超级喜欢（付费）
    SuperLike = 1,
    /// 跳过
    Pass = 2,
    /// 屏蔽
    Block = 3,
}

// ============================================================================
// 匹配状态
// ============================================================================

/// 匹配状态
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum MatchStatus {
    /// 待授权
    #[default]
    PendingAuthorization = 0,
    /// 已授权
    Authorized = 1,
    /// 已完成
    Completed = 2,
    /// 已取消
    Cancelled = 3,
    /// 已拒绝
    Rejected = 4,
}

// ============================================================================
// 匹配建议
// ============================================================================

/// 匹配建议
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default,
)]
pub enum MatchRecommendation {
    /// 天作之合（90-100分）
    PerfectMatch = 0,
    /// 良缘佳配（75-89分）
    GoodMatch = 1,
    /// 中等缘分（60-74分）
    #[default]
    AverageMatch = 2,
    /// 需要磨合（40-59分）
    NeedsWork = 3,
    /// 不建议（0-39分）
    NotRecommended = 4,
}

impl MatchRecommendation {
    /// 根据评分获取建议
    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => Self::PerfectMatch,
            75..=89 => Self::GoodMatch,
            60..=74 => Self::AverageMatch,
            40..=59 => Self::NeedsWork,
            _ => Self::NotRecommended,
        }
    }
}

// ============================================================================
// 用户资料（完整版）
// ============================================================================

/// 用户征婚资料（完整版）
/// 
/// 设计原则：
/// 1. 分层存储：敏感信息加密存储，公开信息链上存储
/// 2. 隐私控制：用户可控制每个字段的可见性
/// 3. 存储优化：照片等大文件存储在 IPFS，链上只存储 CID
/// 4. 可选字段：大部分字段为可选，降低用户填写门槛
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(T))]
pub struct UserProfile<T: ProfileConfig> {
    // ========== 基本信息（公开可见） ==========
    /// 昵称
    pub nickname: BoundedVec<u8, T::MaxNicknameLen>,
    /// 性别
    pub gender: Gender,
    /// 年龄（可选，用于匹配）
    pub age: Option<u8>,
    /// 出生日期（可选，用于八字合婚）
    pub birth_date: Option<BirthDate>,
    /// 出生时间（可选，用于精确八字计算）
    pub birth_time: Option<BirthTime>,
    /// 出生地（可选，用于真太阳时校正）
    pub birth_location: Option<BirthLocation<T>>,
    /// 当前所在地（可选）
    pub current_location: Option<BoundedVec<u8, T::MaxLocationLen>>,
    /// 头像 IPFS CID（可选）
    pub avatar_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
    /// 生活照片列表（IPFS CID 列表，最多 9 张）
    pub photo_cids: BoundedVec<BoundedVec<u8, T::MaxCidLen>, ConstU32<9>>,
    
    // ========== 个人条件（可选，用于匹配） ==========
    /// 身高（cm，可选）
    pub height: Option<u16>,
    /// 体重（kg，可选）
    pub weight: Option<u16>,
    /// 学历（可选）
    pub education: Option<EducationLevel>,
    /// 职业（可选）
    pub occupation: Option<BoundedVec<u8, T::MaxOccupationLen>>,
    /// 收入范围（可选，月收入，单位：元）
    pub income_range: Option<(u32, u32)>,
    /// 房产情况（可选）
    pub property_status: Option<PropertyStatus>,
    /// 车辆情况（可选）
    pub vehicle_status: Option<VehicleStatus>,
    /// 婚姻状况（可选）
    pub marital_status: Option<MaritalStatus>,
    /// 是否有孩子（可选）
    pub has_children: Option<bool>,
    /// 是否想要孩子（可选）
    pub wants_children: Option<bool>,
    
    // ========== 性格与兴趣（用于匹配） ==========
    /// 性格特点（基于八字解盘，可选）
    pub personality_traits: BoundedVec<PersonalityTrait, T::MaxTraits>,
    /// 兴趣爱好（可选）
    pub hobbies: BoundedVec<BoundedVec<u8, T::MaxHobbyLen>, T::MaxHobbies>,
    /// 生活方式（可选）
    pub lifestyle: Option<Lifestyle>,
    
    // ========== 玄学信息（Stardust 特色） ==========
    /// 八字命盘 ID（可选，用于合婚分析）
    pub bazi_chart_id: Option<u64>,
    /// 合婚偏好（可选）
    pub compatibility_preferences: Option<CompatibilityPreferences>,
    
    // ========== 择偶条件 ==========
    /// 择偶条件（可选）
    pub partner_preferences: Option<PartnerPreferences<T>>,
    
    // ========== 自我介绍 ==========
    /// 个人简介（可选）
    pub bio: Option<BoundedVec<u8, T::MaxBioLen>>,
    /// 理想对象描述（可选）
    pub ideal_partner_desc: Option<BoundedVec<u8, T::MaxDescLen>>,
    
    // ========== 隐私与权限 ==========
    /// 隐私模式
    pub privacy_mode: ProfilePrivacyMode,
    /// 字段级隐私设置（控制每个字段的可见性）
    pub field_privacy: FieldPrivacySettings,
    
    // ========== 状态与元数据 ==========
    /// 资料完整度（0-100）
    pub completeness: u8,
    /// 资料状态
    pub status: ProfileStatus,
    /// 是否已验证（KYC）
    pub verified: bool,
    /// 创建时间
    pub created_at: u64,
    /// 最后更新时间
    pub updated_at: u64,
    /// 最后活跃时间
    pub last_active_at: u64,
}

/// 出生日期
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct BirthDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    /// 是否为农历
    pub is_lunar: bool,
}

/// 出生时间
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct BirthTime {
    pub hour: u8,
    pub minute: u8,
    /// 是否使用真太阳时
    pub use_true_solar_time: bool,
}

/// 出生地信息（用于真太阳时校正）
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(T))]
pub struct BirthLocation<T: ProfileConfig> {
    /// 城市名称
    pub city: BoundedVec<u8, T::MaxLocationLen>,
    /// 经度（用于真太阳时校正，可选）
    pub longitude: Option<i32>,
    /// 纬度（可选）
    pub latitude: Option<i32>,
}

/// 合婚偏好
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct CompatibilityPreferences {
    /// 最低合婚评分要求（0-100）
    pub min_compatibility_score: u8,
    /// 是否要求五行相生
    pub require_wuxing_compatible: bool,
    /// 是否要求性格互补
    pub require_personality_complementary: bool,
    /// 是否要求性格相似
    pub require_personality_similar: bool,
}

/// 择偶条件（详细版）
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(T))]
pub struct PartnerPreferences<T: ProfileConfig> {
    // 基础条件
    /// 年龄范围
    pub age_range: Option<(u8, u8)>,
    /// 身高范围（cm）
    pub height_range: Option<(u16, u16)>,
    /// 地域偏好
    pub location_preference: Option<BoundedVec<u8, T::MaxLocationLen>>,
    /// 学历要求
    pub education_requirement: Option<EducationLevel>,
    /// 收入范围（月收入，单位：元）
    pub income_range: Option<(u32, u32)>,
    /// 房产要求
    pub property_requirement: Option<PropertyStatus>,
    /// 车辆要求
    pub vehicle_requirement: Option<VehicleStatus>,
    /// 婚姻状况要求
    pub marital_status_requirement: Option<MaritalStatus>,
    /// 是否接受有孩子
    pub accept_children: Option<bool>,
    
    // 性格与兴趣
    /// 期望的性格特征
    pub desired_personality_traits: BoundedVec<PersonalityTrait, T::MaxTraits>,
    /// 期望的生活方式
    pub desired_lifestyle: Option<Lifestyle>,
    
    // 玄学条件（Stardust 特色）
    /// 最低八字合婚评分要求
    pub min_bazi_compatibility: Option<u8>,
}

/// 字段级隐私设置
/// 
/// 控制每个字段的可见性：
/// - Public: 所有人可见
/// - Matched: 仅匹配用户可见
/// - Private: 完全私密
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct FieldPrivacySettings {
    /// 年龄可见性
    pub age_visibility: VisibilityLevel,
    /// 身高可见性
    pub height_visibility: VisibilityLevel,
    /// 收入可见性
    pub income_visibility: VisibilityLevel,
    /// 房产可见性
    pub property_visibility: VisibilityLevel,
    /// 照片可见性
    pub photo_visibility: VisibilityLevel,
    /// 八字信息可见性
    pub bazi_visibility: VisibilityLevel,
}

/// 资料配置 Trait
pub trait ProfileConfig {
    type MaxNicknameLen: Get<u32>;
    type MaxLocationLen: Get<u32>;
    type MaxCidLen: Get<u32>;
    type MaxBioLen: Get<u32>;
    type MaxDescLen: Get<u32>;
    type MaxOccupationLen: Get<u32>;
    type MaxTraits: Get<u32>;
    type MaxHobbies: Get<u32>;
    type MaxHobbyLen: Get<u32>;
}

// ============================================================================
// 匹配评分
// ============================================================================

/// 匹配评分
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct MatchingScore {
    /// 八字合婚评分 (0-100)
    pub bazi_compatibility: u8,
    /// 性格匹配评分 (0-100)
    pub personality_match: u8,
    /// 条件匹配评分 (0-100)
    pub condition_match: u8,
    /// 综合评分 (0-100)
    pub overall_score: u8,
}

impl MatchingScore {
    /// 计算综合评分
    /// 权重：八字 40%，性格 35%，条件 25%
    pub fn calculate_overall(&self) -> u8 {
        let weighted = self.bazi_compatibility as u32 * 40
            + self.personality_match as u32 * 35
            + self.condition_match as u32 * 25;
        (weighted / 100) as u8
    }
}

// ============================================================================
// 合婚请求
// ============================================================================

/// 合婚请求
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct CompatibilityRequest<AccountId, BlockNumber> {
    /// 请求 ID
    pub id: u64,
    /// 甲方（发起者）
    pub party_a: AccountId,
    /// 乙方
    pub party_b: AccountId,
    /// 甲方八字 ID
    pub party_a_bazi_id: u64,
    /// 乙方八字 ID
    pub party_b_bazi_id: u64,
    /// 状态
    pub status: MatchStatus,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 授权时间
    pub authorized_at: Option<BlockNumber>,
}

// ============================================================================
// 合婚报告
// ============================================================================

/// 合婚报告
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct CompatibilityReport<BlockNumber> {
    /// 报告 ID
    pub id: u64,
    /// 请求 ID
    pub request_id: u64,
    /// 综合评分
    pub overall_score: u8,
    /// 评分详情
    pub score_detail: CompatibilityScoreDetail,
    /// 匹配建议
    pub recommendation: MatchRecommendation,
    /// 报告 CID（IPFS 存储详细内容）
    pub report_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    /// 生成时间
    pub generated_at: BlockNumber,
    /// 算法版本
    pub algorithm_version: u8,
}

/// 合婚评分详情
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct CompatibilityScoreDetail {
    /// 日柱合婚评分 (0-100)
    pub day_pillar_score: u8,
    /// 五行互补评分 (0-100)
    pub wuxing_score: u8,
    /// 性格匹配评分 (0-100)
    pub personality_score: u8,
    /// 神煞评分 (0-100)
    pub shensha_score: u8,
    /// 大运配合评分 (0-100)
    pub dayun_score: u8,
}

impl CompatibilityScoreDetail {
    /// 计算加权综合评分
    pub fn calculate_overall(&self) -> u8 {
        let weighted = self.day_pillar_score as u32 * 30
            + self.wuxing_score as u32 * 25
            + self.personality_score as u32 * 20
            + self.shensha_score as u32 * 15
            + self.dayun_score as u32 * 10;
        (weighted / 100) as u8
    }
}

// ============================================================================
// 互动记录
// ============================================================================

/// 互动记录
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct Interaction<AccountId, BlockNumber> {
    /// 发起者
    pub from: AccountId,
    /// 接收者
    pub to: AccountId,
    /// 互动类型
    pub interaction_type: InteractionType,
    /// 时间戳
    pub timestamp: BlockNumber,
}

// ============================================================================
// 推荐结果
// ============================================================================

/// 推荐结果
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct RecommendationResult<AccountId> {
    /// 推荐用户
    pub user: AccountId,
    /// 匹配评分
    pub score: u8,
    /// 推荐理由（索引）
    pub reason_index: u8,
}
