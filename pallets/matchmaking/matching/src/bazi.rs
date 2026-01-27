//! # 八字合婚算法
//!
//! 基于日柱天干地支的合婚分析。
//!
//! ## 天干地支索引
//!
//! - 天干: 甲(0) 乙(1) 丙(2) 丁(3) 戊(4) 己(5) 庚(6) 辛(7) 壬(8) 癸(9)
//! - 地支: 子(0) 丑(1) 寅(2) 卯(3) 辰(4) 巳(5) 午(6) 未(7) 申(8) 酉(9) 戌(10) 亥(11)

use pallet_bazi_chart::types::{TianGan, DiZhi, WuXing, GanZhi};
use pallet_bazi_chart::interpretation::CoreInterpretation;

/// 日柱合婚结果
#[derive(Clone, Debug, Default)]
pub struct DayPillarMatchResult {
    pub stem_score: u8,
    pub branch_score: u8,
    pub is_stem_he: bool,
    pub is_branch_liuhe: bool,
    pub is_branch_liuchong: bool,
    pub overall: u8,
}

/// 五行互补结果
#[derive(Clone, Debug, Default)]
pub struct WuxingCompatibilityResult {
    pub yongshen_score: u8,
    pub jishen_score: u8,
    pub balance_score: u8,
    pub overall: u8,
}

/// 天干五合对照表
/// 甲己合、乙庚合、丙辛合、丁壬合、戊癸合
const TIANGAN_HE_PAIRS: [(u8, u8); 5] = [
    (0, 5),  // 甲己合
    (1, 6),  // 乙庚合
    (2, 7),  // 丙辛合
    (3, 8),  // 丁壬合
    (4, 9),  // 戊癸合
];

/// 地支六合对照表
/// 子丑合、寅亥合、卯戌合、辰酉合、巳申合、午未合
const DIZHI_LIUHE_PAIRS: [(u8, u8); 6] = [
    (0, 1),   // 子丑合
    (2, 11),  // 寅亥合
    (3, 10),  // 卯戌合
    (4, 9),   // 辰酉合
    (5, 8),   // 巳申合
    (6, 7),   // 午未合
];

/// 地支六冲对照表
/// 子午冲、丑未冲、寅申冲、卯酉冲、辰戌冲、巳亥冲
const DIZHI_LIUCHONG_PAIRS: [(u8, u8); 6] = [
    (0, 6),   // 子午冲
    (1, 7),   // 丑未冲
    (2, 8),   // 寅申冲
    (3, 9),   // 卯酉冲
    (4, 10),  // 辰戌冲
    (5, 11),  // 巳亥冲
];

/// 检查天干是否相合
pub fn is_tiangan_he(stem1: TianGan, stem2: TianGan) -> bool {
    for (a, b) in TIANGAN_HE_PAIRS.iter() {
        if (stem1.0 == *a && stem2.0 == *b) || (stem1.0 == *b && stem2.0 == *a) {
            return true;
        }
    }
    false
}

/// 检查地支是否六合
pub fn is_dizhi_liuhe(branch1: DiZhi, branch2: DiZhi) -> bool {
    for (a, b) in DIZHI_LIUHE_PAIRS.iter() {
        if (branch1.0 == *a && branch2.0 == *b) || (branch1.0 == *b && branch2.0 == *a) {
            return true;
        }
    }
    false
}

/// 检查地支是否六冲
pub fn is_dizhi_liuchong(branch1: DiZhi, branch2: DiZhi) -> bool {
    for (a, b) in DIZHI_LIUCHONG_PAIRS.iter() {
        if (branch1.0 == *a && branch2.0 == *b) || (branch1.0 == *b && branch2.0 == *a) {
            return true;
        }
    }
    false
}

/// 检查五行是否相生
pub fn is_wuxing_sheng(from: WuXing, to: WuXing) -> bool {
    matches!(
        (from, to),
        (WuXing::Mu, WuXing::Huo)
            | (WuXing::Huo, WuXing::Tu)
            | (WuXing::Tu, WuXing::Jin)
            | (WuXing::Jin, WuXing::Shui)
            | (WuXing::Shui, WuXing::Mu)
    )
}

/// 检查五行是否相克
pub fn is_wuxing_ke(from: WuXing, to: WuXing) -> bool {
    matches!(
        (from, to),
        (WuXing::Mu, WuXing::Tu)
            | (WuXing::Tu, WuXing::Shui)
            | (WuXing::Shui, WuXing::Huo)
            | (WuXing::Huo, WuXing::Jin)
            | (WuXing::Jin, WuXing::Mu)
    )
}

/// 计算天干合婚评分
pub fn calculate_stem_score(stem1: TianGan, stem2: TianGan) -> u8 {
    if is_tiangan_he(stem1, stem2) {
        return 100;
    }

    let wuxing1 = stem1.to_wuxing();
    let wuxing2 = stem2.to_wuxing();

    if wuxing1 == wuxing2 {
        70
    } else if is_wuxing_sheng(wuxing1, wuxing2) || is_wuxing_sheng(wuxing2, wuxing1) {
        80
    } else if is_wuxing_ke(wuxing1, wuxing2) {
        40
    } else if is_wuxing_ke(wuxing2, wuxing1) {
        30
    } else {
        50
    }
}

/// 计算地支合婚评分
pub fn calculate_branch_score(branch1: DiZhi, branch2: DiZhi) -> u8 {
    if is_dizhi_liuhe(branch1, branch2) {
        return 100;
    }

    if is_dizhi_liuchong(branch1, branch2) {
        return 20;
    }

    let wuxing1 = branch1.to_wuxing();
    let wuxing2 = branch2.to_wuxing();

    if wuxing1 == wuxing2 {
        65
    } else if is_wuxing_sheng(wuxing1, wuxing2) || is_wuxing_sheng(wuxing2, wuxing1) {
        75
    } else if is_wuxing_ke(wuxing1, wuxing2) || is_wuxing_ke(wuxing2, wuxing1) {
        45
    } else {
        50
    }
}

/// 计算日柱合婚评分
pub fn calculate_day_pillar_compatibility(
    day_ganzhi_1: GanZhi,
    day_ganzhi_2: GanZhi,
) -> DayPillarMatchResult {
    let stem1 = day_ganzhi_1.gan;
    let stem2 = day_ganzhi_2.gan;
    let branch1 = day_ganzhi_1.zhi;
    let branch2 = day_ganzhi_2.zhi;

    let stem_score = calculate_stem_score(stem1, stem2);
    let branch_score = calculate_branch_score(branch1, branch2);

    let is_stem_he = is_tiangan_he(stem1, stem2);
    let is_branch_liuhe = is_dizhi_liuhe(branch1, branch2);
    let is_branch_liuchong = is_dizhi_liuchong(branch1, branch2);

    let overall = ((stem_score as u32 * 40 + branch_score as u32 * 60) / 100) as u8;

    DayPillarMatchResult {
        stem_score,
        branch_score,
        is_stem_he,
        is_branch_liuhe,
        is_branch_liuchong,
        overall,
    }
}

/// 计算五行互补评分
pub fn calculate_wuxing_compatibility(
    interp1: &CoreInterpretation,
    interp2: &CoreInterpretation,
) -> WuxingCompatibilityResult {
    let mut yongshen_score = 50u8;

    // 用神与喜神配合
    if interp1.yong_shen == interp2.xi_shen {
        yongshen_score = yongshen_score.saturating_add(25);
    }
    if interp2.yong_shen == interp1.xi_shen {
        yongshen_score = yongshen_score.saturating_add(25);
    }

    // 用神相生
    if is_wuxing_sheng(interp1.yong_shen, interp2.yong_shen)
        || is_wuxing_sheng(interp2.yong_shen, interp1.yong_shen)
    {
        yongshen_score = yongshen_score.saturating_add(15);
    }

    // 用神同类
    if interp1.yong_shen == interp2.yong_shen {
        yongshen_score = yongshen_score.saturating_add(10);
    }

    yongshen_score = yongshen_score.min(100);

    // 忌神冲突评分
    let mut jishen_score = 80u8;

    if interp1.ji_shen == interp2.yong_shen {
        jishen_score = jishen_score.saturating_sub(20);
    }
    if interp2.ji_shen == interp1.yong_shen {
        jishen_score = jishen_score.saturating_sub(20);
    }
    if interp1.ji_shen == interp2.ji_shen {
        jishen_score = jishen_score.saturating_add(10);
    }

    jishen_score = jishen_score.min(100);

    // 五行平衡评分
    let mut balance_score = 60u8;

    if is_wuxing_sheng(interp1.xi_shen, interp2.xi_shen)
        || is_wuxing_sheng(interp2.xi_shen, interp1.xi_shen)
    {
        balance_score = balance_score.saturating_add(20);
    }

    if interp1.xi_shen == interp2.xi_shen {
        balance_score = balance_score.saturating_add(15);
    }

    let score_diff = (interp1.score as i16 - interp2.score as i16).unsigned_abs() as u8;
    if score_diff <= 10 {
        balance_score = balance_score.saturating_add(10);
    } else if score_diff <= 20 {
        balance_score = balance_score.saturating_add(5);
    }

    balance_score = balance_score.min(100);

    // 综合评分
    let overall = ((yongshen_score as u32 * 50
        + jishen_score as u32 * 30
        + balance_score as u32 * 20)
        / 100) as u8;

    WuxingCompatibilityResult {
        yongshen_score,
        jishen_score,
        balance_score,
        overall,
    }
}

// ============================================================================
// 神煞冲克分析
// ============================================================================

/// 神煞类型（简化版）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShenShaType {
    /// 吉神
    JiShen,
    /// 凶神
    XiongShen,
}

/// 神煞冲克结果
#[derive(Clone, Debug, Default)]
pub struct ShenShaConflictResult {
    /// 吉神配合评分
    pub jishen_match_score: u8,
    /// 凶神冲克评分
    pub xiongshen_conflict_score: u8,
    /// 综合评分
    pub overall: u8,
}

/// 吉神列表（索引）
/// 天乙贵人、文昌、驿马、将星、华盖、天德、月德、三奇、禄神、羊刃（有时吉）
const JISHEN_LIST: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

/// 凶神列表（索引）
/// 劫煞、亡神、桃花、咸池、孤辰、寡宿、丧门、吊客、白虎、天狗
const XIONGSHEN_LIST: [u8; 10] = [10, 11, 12, 13, 14, 15, 16, 17, 18, 19];

/// 判断神煞类型
pub fn get_shensha_type(shensha_index: u8) -> ShenShaType {
    if JISHEN_LIST.contains(&shensha_index) {
        ShenShaType::JiShen
    } else {
        ShenShaType::XiongShen
    }
}

/// 计算神煞冲克评分
/// 
/// 算法复杂度: O(n), n≤32
pub fn calculate_shensha_conflict(
    shensha_list_1: &[u8],
    shensha_list_2: &[u8],
) -> ShenShaConflictResult {
    let mut jishen_match_score = 50u8;
    let mut xiongshen_conflict_score = 80u8;
    
    // 统计双方吉神和凶神
    let mut jishen_count_1 = 0u8;
    let mut jishen_count_2 = 0u8;
    let mut xiongshen_count_1 = 0u8;
    let mut xiongshen_count_2 = 0u8;
    let mut common_jishen = 0u8;
    let mut common_xiongshen = 0u8;
    
    for &s1 in shensha_list_1.iter() {
        if get_shensha_type(s1) == ShenShaType::JiShen {
            jishen_count_1 += 1;
            // 检查是否有共同吉神
            if shensha_list_2.contains(&s1) {
                common_jishen += 1;
            }
        } else {
            xiongshen_count_1 += 1;
            if shensha_list_2.contains(&s1) {
                common_xiongshen += 1;
            }
        }
    }
    
    for &s2 in shensha_list_2.iter() {
        if get_shensha_type(s2) == ShenShaType::JiShen {
            jishen_count_2 += 1;
        } else {
            xiongshen_count_2 += 1;
        }
    }
    
    // 吉神配合评分
    // 共同吉神越多越好
    jishen_match_score = jishen_match_score.saturating_add(common_jishen.saturating_mul(10));
    // 双方吉神数量均衡加分
    let jishen_diff = (jishen_count_1 as i16 - jishen_count_2 as i16).unsigned_abs() as u8;
    if jishen_diff <= 2 {
        jishen_match_score = jishen_match_score.saturating_add(10);
    }
    jishen_match_score = jishen_match_score.min(100);
    
    // 凶神冲克评分
    // 共同凶神扣分
    xiongshen_conflict_score = xiongshen_conflict_score.saturating_sub(common_xiongshen.saturating_mul(8));
    // 凶神总数过多扣分
    let total_xiongshen = xiongshen_count_1.saturating_add(xiongshen_count_2);
    if total_xiongshen > 6 {
        xiongshen_conflict_score = xiongshen_conflict_score.saturating_sub((total_xiongshen - 6).saturating_mul(5));
    }
    
    // 综合评分
    let overall = ((jishen_match_score as u32 * 60 + xiongshen_conflict_score as u32 * 40) / 100) as u8;
    
    ShenShaConflictResult {
        jishen_match_score,
        xiongshen_conflict_score,
        overall,
    }
}

// ============================================================================
// 大运流年同步分析
// ============================================================================

/// 大运同步结果
#[derive(Clone, Debug, Default)]
pub struct DaYunSyncResult {
    /// 大运方向一致性评分
    pub direction_score: u8,
    /// 大运五行配合评分
    pub wuxing_sync_score: u8,
    /// 综合评分
    pub overall: u8,
}

/// 大运信息（简化版）
#[derive(Clone, Debug, Default)]
pub struct DaYunInfo {
    /// 当前大运五行
    pub current_wuxing: WuXing,
    /// 下一步大运五行
    pub next_wuxing: WuXing,
    /// 大运起运年龄
    pub start_age: u8,
}

/// 计算大运流年同步评分
/// 
/// 算法复杂度: O(m), m=12（最多比较12步大运）
pub fn calculate_dayun_sync(
    dayun_1: &[DaYunInfo],
    dayun_2: &[DaYunInfo],
) -> DaYunSyncResult {
    if dayun_1.is_empty() || dayun_2.is_empty() {
        return DaYunSyncResult {
            direction_score: 60,
            wuxing_sync_score: 60,
            overall: 60,
        };
    }
    
    let mut direction_score = 50u8;
    let mut wuxing_sync_score = 50u8;
    
    // 比较当前大运
    let current_1 = &dayun_1[0];
    let current_2 = &dayun_2[0];
    
    // 当前大运五行配合
    if current_1.current_wuxing == current_2.current_wuxing {
        wuxing_sync_score = wuxing_sync_score.saturating_add(20);
    } else if is_wuxing_sheng(current_1.current_wuxing, current_2.current_wuxing)
        || is_wuxing_sheng(current_2.current_wuxing, current_1.current_wuxing)
    {
        wuxing_sync_score = wuxing_sync_score.saturating_add(15);
    } else if is_wuxing_ke(current_1.current_wuxing, current_2.current_wuxing)
        || is_wuxing_ke(current_2.current_wuxing, current_1.current_wuxing)
    {
        wuxing_sync_score = wuxing_sync_score.saturating_sub(10);
    }
    
    // 比较未来大运方向（最多比较3步）
    let compare_count = dayun_1.len().min(dayun_2.len()).min(3);
    let mut sync_count = 0u8;
    
    for i in 0..compare_count {
        if dayun_1[i].current_wuxing == dayun_2[i].current_wuxing {
            sync_count += 1;
        }
    }
    
    direction_score = direction_score.saturating_add(sync_count.saturating_mul(15));
    
    // 起运年龄差异
    let age_diff = (current_1.start_age as i16 - current_2.start_age as i16).unsigned_abs() as u8;
    if age_diff <= 3 {
        direction_score = direction_score.saturating_add(10);
    }
    
    direction_score = direction_score.min(100);
    wuxing_sync_score = wuxing_sync_score.min(100);
    
    // 综合评分
    let overall = ((direction_score as u32 * 50 + wuxing_sync_score as u32 * 50) / 100) as u8;
    
    DaYunSyncResult {
        direction_score,
        wuxing_sync_score,
        overall,
    }
}

// ============================================================================
// 综合八字合婚算法
// ============================================================================

/// 完整八字合婚结果
#[derive(Clone, Debug, Default)]
pub struct BaziCompatibilityScore {
    /// 五行互补评分 (30%)
    pub wuxing_score: u8,
    /// 神煞冲克评分 (25%)
    pub shensha_score: u8,
    /// 用神喜忌评分 (20%)
    pub yongshen_score: u8,
    /// 大运同步评分 (15%)
    pub dayun_score: u8,
    /// 性格匹配评分 (10%)
    pub personality_score: u8,
    /// 综合评分
    pub overall: u8,
}

impl BaziCompatibilityScore {
    /// 计算加权综合评分
    /// 
    /// 权重分配：
    /// - 五行互补: 30%
    /// - 神煞冲克: 25%
    /// - 用神喜忌: 20%
    /// - 大运同步: 15%
    /// - 性格匹配: 10%
    pub fn calculate_weighted_overall(&self) -> u8 {
        let total = (self.wuxing_score as u32 * 30
            + self.shensha_score as u32 * 25
            + self.yongshen_score as u32 * 20
            + self.dayun_score as u32 * 15
            + self.personality_score as u32 * 10)
            / 100;
        total as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TianGan: 甲(0) 乙(1) 丙(2) 丁(3) 戊(4) 己(5) 庚(6) 辛(7) 壬(8) 癸(9)
    const JIA: TianGan = TianGan(0);
    const YI: TianGan = TianGan(1);
    const JI: TianGan = TianGan(5);

    // DiZhi: 子(0) 丑(1) 寅(2) 卯(3) 辰(4) 巳(5) 午(6) 未(7) 申(8) 酉(9) 戌(10) 亥(11)
    const ZI: DiZhi = DiZhi(0);
    const CHOU: DiZhi = DiZhi(1);
    const YIN: DiZhi = DiZhi(2);
    const WU: DiZhi = DiZhi(6);

    #[test]
    fn test_tiangan_he() {
        assert!(is_tiangan_he(JIA, JI));
        assert!(is_tiangan_he(JI, JIA));
        assert!(!is_tiangan_he(JIA, YI));
    }

    #[test]
    fn test_dizhi_liuhe() {
        assert!(is_dizhi_liuhe(ZI, CHOU));
        assert!(!is_dizhi_liuhe(ZI, YIN));
    }

    #[test]
    fn test_dizhi_liuchong() {
        assert!(is_dizhi_liuchong(ZI, WU));
        assert!(!is_dizhi_liuchong(ZI, CHOU));
    }

    #[test]
    fn test_stem_score() {
        assert_eq!(calculate_stem_score(JIA, JI), 100);
        assert!(calculate_stem_score(JIA, YI) < 100);
    }

    #[test]
    fn test_branch_score() {
        assert_eq!(calculate_branch_score(ZI, CHOU), 100);
        assert_eq!(calculate_branch_score(ZI, WU), 20);
    }
}
