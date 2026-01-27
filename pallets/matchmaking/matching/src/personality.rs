//! # 性格匹配算法
//!
//! 基于八字解盘的性格特征进行匹配分析。

use pallet_bazi_chart::interpretation::{CompactXingGe, XingGeTrait};
use frame_support::pallet_prelude::*;

/// 性格匹配结果
#[derive(Clone, Debug, Default)]
pub struct PersonalityMatchResult {
    pub complementary_score: u8,
    pub conflict_score: u8,
    pub common_strengths_score: u8,
    pub overall: u8,
}

/// 互补性格对照表
const COMPLEMENTARY_PAIRS: [(XingGeTrait, XingGeTrait); 8] = [
    (XingGeTrait::GuoDuan, XingGeTrait::WenHe),
    (XingGeTrait::ReQing, XingGeTrait::WenZhong),
    (XingGeTrait::JiJiXiangShang, XingGeTrait::NeiLian),
    (XingGeTrait::YouLingDaoLi, XingGeTrait::ShiYingXingQiang),
    (XingGeTrait::KaiLang, XingGeTrait::XiXin),
    (XingGeTrait::YouZhuJian, XingGeTrait::BaoRong),
    (XingGeTrait::ZhiXingLiQiang, XingGeTrait::ShanYuXieTiao),
    (XingGeTrait::YouChuangZaoLi, XingGeTrait::WenZhong),
];

/// 冲突性格对照表
const CONFLICT_PAIRS: [(XingGeTrait, XingGeTrait); 6] = [
    (XingGeTrait::GuZhi, XingGeTrait::GuZhi),
    (XingGeTrait::JiZao, XingGeTrait::JiZao),
    (XingGeTrait::QingXuHua, XingGeTrait::QingXuHua),
    (XingGeTrait::GangYing, XingGeTrait::GangYing),
    (XingGeTrait::YouRouGuaDuan, XingGeTrait::YouRouGuaDuan),
    (XingGeTrait::QueFaNaiXin, XingGeTrait::QueFaNaiXin),
];

/// 检查是否包含某个性格特征
fn has_trait<const N: u32>(traits: &BoundedVec<XingGeTrait, ConstU32<N>>, target: XingGeTrait) -> bool {
    traits.iter().any(|t| *t == target)
}

/// 计算互补性格评分
pub fn calculate_complementary_score(xingge1: &CompactXingGe, xingge2: &CompactXingGe) -> u8 {
    let mut score = 40u8;

    for (trait1, trait2) in COMPLEMENTARY_PAIRS.iter() {
        let match1 = (has_trait(&xingge1.zhu_yao_te_dian, *trait1)
            || has_trait(&xingge1.you_dian, *trait1))
            && (has_trait(&xingge2.zhu_yao_te_dian, *trait2)
                || has_trait(&xingge2.you_dian, *trait2));

        let match2 = (has_trait(&xingge1.zhu_yao_te_dian, *trait2)
            || has_trait(&xingge1.you_dian, *trait2))
            && (has_trait(&xingge2.zhu_yao_te_dian, *trait1)
                || has_trait(&xingge2.you_dian, *trait1));

        if match1 || match2 {
            score = score.saturating_add(12);
        }
    }

    score.min(100)
}

/// 计算冲突性格评分
pub fn calculate_conflict_score(xingge1: &CompactXingGe, xingge2: &CompactXingGe) -> u8 {
    let mut score = 90u8;

    for (trait1, trait2) in CONFLICT_PAIRS.iter() {
        let has_conflict = (has_trait(&xingge1.que_dian, *trait1)
            || has_trait(&xingge1.zhu_yao_te_dian, *trait1))
            && (has_trait(&xingge2.que_dian, *trait2)
                || has_trait(&xingge2.zhu_yao_te_dian, *trait2));

        if has_conflict {
            score = score.saturating_sub(15);
        }
    }

    score
}

/// 计算共同优点评分
pub fn calculate_common_strengths_score(xingge1: &CompactXingGe, xingge2: &CompactXingGe) -> u8 {
    let mut score = 30u8;
    let mut common_count = 0u8;

    for trait1 in xingge1.you_dian.iter() {
        if has_trait(&xingge2.you_dian, *trait1) {
            common_count = common_count.saturating_add(1);
        }
    }

    score = score.saturating_add(common_count.saturating_mul(15));

    for trait1 in xingge1.zhu_yao_te_dian.iter() {
        if has_trait(&xingge2.zhu_yao_te_dian, *trait1) {
            score = score.saturating_add(10);
        }
    }

    score.min(100)
}

/// 计算性格匹配综合评分
pub fn calculate_personality_compatibility(
    xingge1: &CompactXingGe,
    xingge2: &CompactXingGe,
) -> PersonalityMatchResult {
    let complementary_score = calculate_complementary_score(xingge1, xingge2);
    let conflict_score = calculate_conflict_score(xingge1, xingge2);
    let common_strengths_score = calculate_common_strengths_score(xingge1, xingge2);

    let overall = ((complementary_score as u32 * 40
        + conflict_score as u32 * 35
        + common_strengths_score as u32 * 25)
        / 100) as u8;

    PersonalityMatchResult {
        complementary_score,
        conflict_score,
        common_strengths_score,
        overall,
    }
}

/// 计算默认性格匹配评分
pub fn calculate_default_personality_score() -> PersonalityMatchResult {
    PersonalityMatchResult {
        complementary_score: 50,
        conflict_score: 70,
        common_strengths_score: 50,
        overall: 55,
    }
}
