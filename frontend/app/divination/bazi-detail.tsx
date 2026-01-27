/**
 * 星尘玄鉴 - 八字命盘详情页
 * 展示完整的八字分析信息 + 8 个可扩展功能
 */

import { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { divinationService } from '@/services/divination.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';
import {
  calculateDaYun,
  calculateLiuNian,
  calculateShenSha,
  analyzeGeJu,
  analyzeLuWei,
  analyzeNaYin,
  analyzePillarRelationships,
  analyzeMingGong,
} from '@/lib/bazi-analysis';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

// 天干
const TIAN_GAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
const TIAN_GAN_WUXING = ['木', '木', '火', '火', '土', '土', '金', '金', '水', '水'];
const TIAN_GAN_YIN_YANG = ['阳', '阴', '阳', '阴', '阳', '阴', '阳', '阴', '阳', '阴'];

// 地支
const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];
const DI_ZHI_WUXING = ['水', '土', '木', '木', '土', '火', '火', '土', '金', '金', '土', '水'];
const DI_ZHI_YIN_YANG = ['阳', '阴', '阳', '阴', '阳', '阴', '阳', '阴', '阳', '阴', '阳', '阴'];

// 纳音五行
const NAYIN_WUXING: Record<string, string> = {
  '子': '水', '丑': '土', '寅': '木', '卯': '木', '辰': '土', '巳': '火',
  '午': '火', '未': '土', '申': '金', '酉': '金', '戌': '土', '亥': '水',
};

// 十神 (Ten Gods)
const SHI_SHEN_MAP: Record<string, string> = {
  '比肩': '同性相助',
  '劫财': '异性相克',
  '食神': '泄秀气',
  '伤官': '克制官星',
  '偏财': '横财运',
  '正财': '正财运',
  '七杀': '克身之力',
  '正官': '官运',
  '偏印': '背禄',
  '正印': '生身之力',
};

// 五行颜色
const WU_XING_COLORS: Record<string, string> = {
  '木': '#2E7D32',
  '火': '#C62828',
  '土': '#F57C00',
  '金': '#FDD835',
  '水': '#1565C0',
};

// 八字结果接口
interface BaziResult {
  id: number;
  name: string;
  birthYear: number;
  birthMonth: number;
  birthDay: number;
  birthHour: number;
  gender: 'male' | 'female';
  siZhu: {
    year: { gan: number; zhi: number };
    month: { gan: number; zhi: number };
    day: { gan: number; zhi: number };
    hour: { gan: number; zhi: number };
  };
  wuxingCount: Record<string, number>;
  dayMaster: number;
  shengxiao: string;
  createdAt: Date;
}

export default function BaziDetailPage() {
  const router = useRouter();
  const params = useLocalSearchParams();

  // 显示模式：传统模式 vs 分析模式
  const [viewMode, setViewMode] = useState<'traditional' | 'analysis'>('traditional');

  // 从参数中获取八字数据
  const [result] = useState<BaziResult | null>(() => {
    if (params.data && typeof params.data === 'string') {
      try {
        return JSON.parse(params.data);
      } catch {
        return null;
      }
    }
    return null;
  });

  const [fullInterpretation, setFullInterpretation] = useState<any>(null);
  const [loadingFull, setLoadingFull] = useState(false);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingAction, setPendingAction] = useState<'delete' | 'cache' | null>(null);

  // 获取完整解盘
  useEffect(() => {
    const fetchFull = async () => {
      if (result?.id && result.id > 1000000000) { // 简单判断是否为本地临时 ID
        return;
      }
      
      if (result?.id) {
        setLoadingFull(true);
        try {
          // 1. 尝试从链上获取完整解盘 (Runtime API)
          const data = await divinationService.getFullInterpretation(result.id);
          if (data) {
            setFullInterpretation(data);
            // 2. 异步缓存到链上 (Extrinsic)
            // 注意：这需要签名，通常建议在详情页由用户手动触发或仅显示
            // 这里我们先只显示
          }
        } catch (error) {
          console.error('获取完整解盘失败:', error);
        } finally {
          setLoadingFull(false);
        }
      }
    };
    fetchFull();
  }, [result?.id]);

  // 删除命盘
  const handleDelete = () => {
    if (!result?.id || result.id > 1000000000) {
      Alert.alert('提示', '该命盘为临时数据，无需删除');
      return;
    }

    Alert.alert(
      '确认删除',
      '删除后无法恢复，确定要删除该命盘吗？',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '删除',
          style: 'destructive',
          onPress: () => {
            if (!isSignerUnlocked()) {
              setPendingAction('delete');
              setShowUnlockDialog(true);
              return;
            }
            executeDelete();
          },
        },
      ]
    );
  };

  const executeDelete = async () => {
    if (!result?.id) return;

    setShowTxStatus(true);
    setTxStatus('正在删除命盘...');

    try {
      await divinationService.deleteBaziChart(result.id, (status) => setTxStatus(status));

      setTxStatus('删除成功！');
      setTimeout(() => {
        setShowTxStatus(false);
        router.back();
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('删除失败', error.message || '请稍后重试');
    }
  };

  // 缓存解盘结果
  const handleCacheInterpretation = () => {
    if (!result?.id || result.id > 1000000000) {
      Alert.alert('提示', '该命盘为临时数据，无法缓存');
      return;
    }

    if (!isSignerUnlocked()) {
      setPendingAction('cache');
      setShowUnlockDialog(true);
      return;
    }

    executeCacheInterpretation();
  };

  const executeCacheInterpretation = async () => {
    if (!result?.id) return;

    setShowTxStatus(true);
    setTxStatus('正在缓存解盘结果...');

    try {
      await divinationService.cacheInterpretation(result.id, (status) => setTxStatus(status));

      setTxStatus('缓存成功！');
      setTimeout(() => {
        setShowTxStatus(false);
      }, 1500);
    } catch (error: any) {
      setShowTxStatus(false);
      Alert.alert('缓存失败', error.message || '请稍后重试');
    }
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      if (pendingAction === 'delete') {
        await executeDelete();
      } else if (pendingAction === 'cache') {
        await executeCacheInterpretation();
      }
      setPendingAction(null);
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  if (!result) {
    return (
      <View style={styles.container}>
        <View style={styles.navBar}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Ionicons name="chevron-back" size={24} color="#333" />
          </Pressable>
          <Text style={styles.navTitle}>命盘详情</Text>
          <View style={{ width: 24 }} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>暂无数据</Text>
        </View>
      </View>
    );
  }

  // 计算十神
  const calculateShiShen = (ganIndex: number, dayMasterIndex: number): string => {
    const diff = (ganIndex - dayMasterIndex + 10) % 10;
    const shiShenList = ['比肩', '劫财', '食神', '伤官', '偏财', '正财', '七杀', '正官', '偏印', '正印'];
    return shiShenList[diff] || '未知';
  };

  // 获取藏干 (Hidden Stems in Earthly Branches)
  const getHiddenStems = (zhiIndex: number): number[] => {
    const hiddenMap: Record<number, number[]> = {
      0: [9], // 子 - 癸
      1: [5, 9, 1], // 丑 - 己、癸、乙
      2: [3, 7], // 寅 - 甲、丙
      3: [1, 5], // 卯 - 乙、己
      4: [1, 5, 7], // 辰 - 甲、戊、丙
      5: [7, 9, 5], // 巳 - 丙、庚、戊
      6: [7, 9], // 午 - 丙、己
      7: [5, 9, 1], // 未 - 己、丁、乙
      8: [7, 9, 1], // 申 - 庚、壬、甲
      9: [9, 5, 7], // 酉 - 辛、己、丁
      10: [7, 5, 9], // 戌 - 戊、辛、丙
      11: [9, 1], // 亥 - 壬、甲
    };
    return hiddenMap[zhiIndex] || [];
  };

  // 渲染基本信息
  const renderBasicInfo = () => (
    <View style={styles.card}>
      <Text style={styles.cardTitle}>基本信息</Text>
      <View style={styles.infoGrid}>
        <View style={styles.infoItem}>
          <Text style={styles.infoLabel}>姓名</Text>
          <Text style={styles.infoValue}>{result.name}</Text>
        </View>
        <View style={styles.infoItem}>
          <Text style={styles.infoLabel}>性别</Text>
          <Text style={styles.infoValue}>{result.gender === 'male' ? '男' : '女'}</Text>
        </View>
        <View style={styles.infoItem}>
          <Text style={styles.infoLabel}>生肖</Text>
          <Text style={styles.infoValue}>属{result.shengxiao}</Text>
        </View>
        <View style={styles.infoItem}>
          <Text style={styles.infoLabel}>出生时间</Text>
          <Text style={styles.infoValue}>
            {result.birthYear}年{result.birthMonth}月{result.birthDay}日 {result.birthHour}时
          </Text>
        </View>
      </View>
    </View>
  );

  // 渲染四柱详细信息 - 传统样式
  const renderSiZhuDetail = () => {
    if (!result) return null;

    const pillars = [
      { label: '年柱', data: result.siZhu.year, name: '年' },
      { label: '月柱', data: result.siZhu.month, name: '月' },
      { label: '日柱', data: result.siZhu.day, name: '日', isDay: true },
      { label: '时柱', data: result.siZhu.hour, name: '时' },
    ];

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>四柱八字</Text>

        {/* 传统横向排列 */}
        <View style={styles.traditionalSiZhu}>
          {pillars.map((pillar, index) => {
            const ganWuxing = TIAN_GAN_WUXING[pillar.data.gan] || '未知';
            const zhiWuxing = DI_ZHI_WUXING[pillar.data.zhi] || '未知';
            const shiShen = calculateShiShen(pillar.data.gan, result.dayMaster);
            const hiddenStems = getHiddenStems(pillar.data.zhi);

            return (
              <View key={index} style={styles.traditionalPillar}>
                {/* 十神标签 */}
                <View style={styles.shiShenTag}>
                  <Text style={styles.shiShenTagText}>{shiShen}</Text>
                </View>

                {/* 天干 */}
                <View style={[styles.traditionalGanBox, { borderColor: THEME_COLOR }]}>
                  <Text style={[styles.traditionalGanText, { color: WU_XING_COLORS[ganWuxing] || '#333' }]}>
                    {TIAN_GAN[pillar.data.gan] || '?'}
                  </Text>
                </View>

                {/* 地支 */}
                <View style={[styles.traditionalZhiBox, { borderColor: THEME_COLOR }]}>
                  <Text style={[styles.traditionalZhiText, { color: WU_XING_COLORS[zhiWuxing] || '#333' }]}>
                    {DI_ZHI[pillar.data.zhi] || '?'}
                  </Text>
                </View>

                {/* 藏干 */}
                {hiddenStems.length > 0 && (
                  <View style={styles.traditionalHiddenStems}>
                    {hiddenStems.map((stemIndex, i) => (
                      <Text key={i} style={styles.traditionalHiddenStemText}>
                        {TIAN_GAN[stemIndex] || '?'}
                      </Text>
                    ))}
                  </View>
                )}

                {/* 柱名 */}
                <Text style={styles.traditionalPillarLabel}>{pillar.name}</Text>

                {/* 日主标记 */}
                {pillar.isDay && (
                  <View style={styles.traditionalDayMasterMark}>
                    <Text style={styles.traditionalDayMasterText}>日主</Text>
                  </View>
                )}
              </View>
            );
          })}
        </View>

        {/* 详细分析 */}
        <View style={styles.detailedAnalysis}>
          <Text style={styles.analysisTitle}>详细分析</Text>
          {pillars.map((pillar, index) => {
            const ganWuxing = TIAN_GAN_WUXING[pillar.data.gan] || '未知';
            const zhiWuxing = DI_ZHI_WUXING[pillar.data.zhi] || '未知';
            const shiShen = calculateShiShen(pillar.data.gan, result.dayMaster);
            const hiddenStems = getHiddenStems(pillar.data.zhi);

            return (
              <View key={index} style={styles.analysisItem}>
                <Text style={styles.analysisLabel}>{pillar.label}：</Text>
                <View style={styles.analysisContent}>
                  <Text style={styles.analysisText}>
                    天干 {TIAN_GAN[pillar.data.gan]}（{ganWuxing}{TIAN_GAN_YIN_YANG[pillar.data.gan]}）- {shiShen}
                  </Text>
                  <Text style={styles.analysisText}>
                    地支 {DI_ZHI[pillar.data.zhi]}（{zhiWuxing}{DI_ZHI_YIN_YANG[pillar.data.zhi]}）
                  </Text>
                  {hiddenStems.length > 0 && (
                    <Text style={styles.analysisText}>
                      藏干：{hiddenStems.map(s => TIAN_GAN[s]).join('、')}
                    </Text>
                  )}
                </View>
              </View>
            );
          })}
        </View>
      </View>
    );
  };

  // 渲染五行分析
  const renderWuXingAnalysis = () => {
    if (!result) return null;

    const total = Object.values(result.wuxingCount).reduce((a, b) => a + (b || 0), 0);
    const wuXingList = ['木', '火', '土', '金', '水'];

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>五行分析</Text>
        <View style={styles.wuxingAnalysis}>
          {wuXingList.map(wx => {
            const count = result.wuxingCount[wx] || 0;
            const percent = total > 0 ? Math.round((count / total) * 100) : 0;
            const isLacking = count === 0;

            return (
              <View key={wx} style={styles.wuxingAnalysisItem}>
                <View style={styles.wuxingAnalysisHeader}>
                  <Text style={[styles.wuxingAnalysisName, { color: WU_XING_COLORS[wx] || '#999' }]}>
                    {wx}
                  </Text>
                  <Text style={styles.wuxingAnalysisCount}>
                    {count}个 ({percent}%)
                  </Text>
                </View>
                <View style={styles.wuxingAnalysisBar}>
                  <View
                    style={[
                      styles.wuxingAnalysisBarFill,
                      {
                        width: `${percent}%`,
                        backgroundColor: isLacking ? '#F0F0F0' : (WU_XING_COLORS[wx] || '#999'),
                      },
                    ]}
                  />
                </View>
                {isLacking && (
                  <Text style={styles.lackingText}>缺失</Text>
                )}
              </View>
            );
          })}
        </View>
      </View>
    );
  };

  // 渲染日主分析
  const renderDayMasterAnalysis = () => {
    if (!result) return null;

    const dayMasterGan = TIAN_GAN[result.dayMaster] || '?';
    const dayMasterWuxing = TIAN_GAN_WUXING[result.dayMaster] || '未知';

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>日主分析</Text>
        <View style={styles.dayMasterBox}>
          <View style={[styles.dayMasterCircle, { borderColor: WU_XING_COLORS[dayMasterWuxing] || '#999' }]}>
            <Text style={[styles.dayMasterText, { color: WU_XING_COLORS[dayMasterWuxing] || '#999' }]}>
              {dayMasterGan}
            </Text>
          </View>
          <View style={styles.dayMasterInfo}>
            <Text style={styles.dayMasterTitle}>日主天干: {dayMasterGan}</Text>
            <Text style={styles.dayMasterDesc}>五行属性: {dayMasterWuxing}</Text>
            <Text style={styles.dayMasterDesc}>阴阳属性: {TIAN_GAN_YIN_YANG[result.dayMaster] || '?'}</Text>
          </View>
        </View>
      </View>
    );
  };

  // 渲染十神分析
  const renderShiShenAnalysis = () => {
    if (!result) return null;

    const shiShenData = [
      { label: '年柱', gan: result.siZhu.year.gan },
      { label: '月柱', gan: result.siZhu.month.gan },
      { label: '日柱', gan: result.siZhu.day.gan },
      { label: '时柱', gan: result.siZhu.hour.gan },
    ];

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>十神分析</Text>
        <View style={styles.shiShenGrid}>
          {shiShenData.map((item, index) => {
            const shiShen = calculateShiShen(item.gan, result.dayMaster);
            const wuxing = TIAN_GAN_WUXING[item.gan] || '未知';

            return (
              <View key={index} style={styles.shiShenItem}>
                <Text style={styles.shiShenLabel}>{item.label}</Text>
                <View style={[styles.shiShenBox, { borderColor: WU_XING_COLORS[wuxing] || '#999' }]}>
                  <Text style={[styles.shiShenName, { color: WU_XING_COLORS[wuxing] || '#999' }]}>
                    {shiShen}
                  </Text>
                </View>
                <Text style={styles.shiShenDesc}>{SHI_SHEN_MAP[shiShen] || '未知'}</Text>
              </View>
            );
          })}
        </View>
      </View>
    );
  };

  // 渲染有利元素建议
  const renderFavorableElements = () => {
    if (!result) return null;

    const lackingElements = Object.entries(result.wuxingCount)
      .filter(([_, count]) => (count || 0) === 0)
      .map(([wx]) => wx);

    const weakElements = Object.entries(result.wuxingCount)
      .filter(([_, count]) => (count || 0) === 1)
      .map(([wx]) => wx);

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>五行建议</Text>

        {lackingElements.length > 0 && (
          <View style={styles.suggestionSection}>
            <Text style={styles.suggestionTitle}>缺失元素</Text>
            <View style={styles.elementsList}>
              {lackingElements.map(wx => (
                <View key={wx} style={[styles.elementTag, { backgroundColor: WU_XING_COLORS[wx] + '20' }]}>
                  <Text style={[styles.elementTagText, { color: WU_XING_COLORS[wx] }]}>
                    {wx}
                  </Text>
                </View>
              ))}
            </View>
            <Text style={styles.suggestionText}>
              建议在名字、穿衣、居住方向等方面补充{lackingElements.join('、')}元素
            </Text>
          </View>
        )}

        {weakElements.length > 0 && (
          <View style={styles.suggestionSection}>
            <Text style={styles.suggestionTitle}>弱势元素</Text>
            <View style={styles.elementsList}>
              {weakElements.map(wx => (
                <View key={wx} style={[styles.elementTag, { backgroundColor: WU_XING_COLORS[wx] + '20' }]}>
                  <Text style={[styles.elementTagText, { color: WU_XING_COLORS[wx] }]}>
                    {wx}
                  </Text>
                </View>
              ))}
            </View>
            <Text style={styles.suggestionText}>
              建议适当加强{weakElements.join('、')}元素的补充
            </Text>
          </View>
        )}
      </View>
    );
  };

  // ============================================================================
  // 8 个可扩展功能的渲染函数
  // ============================================================================

  // 1. 渲染大运分析
  const renderDaYun = () => {
    if (!result) return null;

    const daYunList = calculateDaYun(result.dayMaster, result.siZhu.month.gan);

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>大运分析 (Major Luck Periods)</Text>
        <Text style={styles.cardDescription}>每 10 年一个大运周期，影响该时期的整体运势</Text>
        <View style={styles.daYunList}>
          {daYunList.slice(0, 4).map((daYun, index) => (
            <View key={index} style={styles.daYunItem}>
              <View style={styles.daYunHeader}>
                <Text style={styles.daYunAge}>{daYun.startAge}-{daYun.endAge}岁</Text>
                <View style={[styles.daYunTag, { backgroundColor: WU_XING_COLORS[daYun.wuxing] + '30' }]}>
                  <Text style={[styles.daYunTagText, { color: WU_XING_COLORS[daYun.wuxing] }]}>
                    {daYun.wuxing}
                  </Text>
                </View>
              </View>
              <Text style={styles.daYunDesc}>{daYun.description}</Text>
            </View>
          ))}
        </View>
      </View>
    );
  };

  // 2. 渲染流年分析
  const renderLiuNian = () => {
    if (!result) return null;

    const currentYear = new Date().getFullYear();
    const liuNianList = calculateLiuNian(currentYear, 5);

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>流年分析 (Annual Luck)</Text>
        <Text style={styles.cardDescription}>每年的运势变化，影响该年的吉凶祸福</Text>
        <View style={styles.liuNianList}>
          {liuNianList.map((liuNian, index) => (
            <View key={index} style={styles.liuNianItem}>
              <View style={styles.liuNianYear}>
                <Text style={styles.liuNianYearText}>{liuNian.year}年</Text>
                <View style={[styles.liuNianWuxing, { backgroundColor: WU_XING_COLORS[liuNian.wuxing] + '30' }]}>
                  <Text style={[styles.liuNianWuxingText, { color: WU_XING_COLORS[liuNian.wuxing] }]}>
                    {liuNian.wuxing}
                  </Text>
                </View>
              </View>
            </View>
          ))}
        </View>
      </View>
    );
  };

  // 3. 渲染神煞分析
  const renderShenSha = () => {
    if (!result) return null;

    const shenShaList = calculateShenSha(
      result.siZhu.year.zhi,
      result.siZhu.month.zhi,
      result.siZhu.day.zhi,
      result.siZhu.hour.zhi
    );

    const auspicious = shenShaList.filter(s => s.type === 'auspicious');
    const inauspicious = shenShaList.filter(s => s.type === 'inauspicious');

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>神煞分析 (Auspicious/Inauspicious Stars)</Text>
        <Text style={styles.cardDescription}>命盘中的吉神凶煞，影响命主的吉凶运势</Text>

        {auspicious.length > 0 && (
          <View style={styles.shenShaSection}>
            <Text style={[styles.shenShaTitle, { color: '#27AE60' }]}>吉神</Text>
            {auspicious.map((star, index) => (
              <View key={index} style={styles.shenShaItem}>
                <View style={[styles.shenShaIndicator, { backgroundColor: '#27AE60' }]} />
                <View style={styles.shenShaContent}>
                  <Text style={styles.shenShaName}>{star.name}</Text>
                  <Text style={styles.shenShaDesc}>{star.description}</Text>
                </View>
              </View>
            ))}
          </View>
        )}

        {inauspicious.length > 0 && (
          <View style={styles.shenShaSection}>
            <Text style={[styles.shenShaTitle, { color: '#E74C3C' }]}>凶煞</Text>
            {inauspicious.map((star, index) => (
              <View key={index} style={styles.shenShaItem}>
                <View style={[styles.shenShaIndicator, { backgroundColor: '#E74C3C' }]} />
                <View style={styles.shenShaContent}>
                  <Text style={styles.shenShaName}>{star.name}</Text>
                  <Text style={styles.shenShaDesc}>{star.description}</Text>
                </View>
              </View>
            ))}
          </View>
        )}
      </View>
    );
  };

  // 4. 渲染格局判断
  const renderGeJu = () => {
    if (!result) return null;

    const geJu = analyzeGeJu(result.dayMaster, result.wuxingCount);

    const levelColors: Record<string, string> = {
      superior: '#27AE60',
      good: '#3498DB',
      average: '#F39C12',
      poor: '#E74C3C',
    };

    const levelLabels: Record<string, string> = {
      superior: '上等',
      good: '良好',
      average: '一般',
      poor: '较差',
    };

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>格局判断 (Chart Pattern)</Text>
        <View style={styles.geJuBox}>
          <View style={[styles.geJuLevel, { backgroundColor: levelColors[geJu.level] + '20' }]}>
            <Text style={[styles.geJuLevelText, { color: levelColors[geJu.level] }]}>
              {levelLabels[geJu.level]}格局
            </Text>
          </View>
          <Text style={styles.geJuName}>{geJu.name}</Text>
          <Text style={styles.geJuDesc}>{geJu.description}</Text>
          <View style={styles.geJuCharacteristics}>
            {geJu.characteristics.map((char, index) => (
              <View key={index} style={styles.geJuCharItem}>
                <Text style={styles.geJuCharText}>{char}</Text>
              </View>
            ))}
          </View>
        </View>
      </View>
    );
  };

  // 5. 渲染禄位分析
  const renderLuWei = () => {
    if (!result) return null;

    const luWei = analyzeLuWei(result.dayMaster);

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>禄位分析 (Wealth Position)</Text>
        <View style={styles.luWeiBox}>
          <Text style={styles.luWeiDesc}>{luWei.description}</Text>
          <View style={styles.luWeiPosition}>
            <Text style={styles.luWeiLabel}>禄位地支:</Text>
            <Text style={styles.luWeiValue}>{luWei.luPosition}</Text>
          </View>
        </View>
      </View>
    );
  };

  // 6. 渲染纳音五行分析
  const renderNaYin = () => {
    if (!result) return null;

    const naYin = analyzeNaYin(
      result.siZhu.year.gan,
      result.siZhu.year.zhi,
      result.siZhu.month.gan,
      result.siZhu.month.zhi,
      result.siZhu.day.gan,
      result.siZhu.day.zhi,
      result.siZhu.hour.gan,
      result.siZhu.hour.zhi
    );

    const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];
    const TIAN_GAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>纳音五行 (Nayin Five Elements)</Text>
        <Text style={styles.cardDescription}>{naYin.description}</Text>
        <View style={styles.nayinGrid}>
          {[
            { label: '年纳音', data: naYin.year },
            { label: '月纳音', data: naYin.month },
            { label: '日纳音', data: naYin.day },
            { label: '时纳音', data: naYin.hour },
          ].map((item, index) => (
            <View key={index} style={styles.nayinItem}>
              <Text style={styles.nayinLabel}>{item.label}</Text>
              <Text style={styles.nayinValue}>{item.data.nayin}</Text>
              <Text style={styles.nayinGanZhi}>
                {TIAN_GAN[item.data.gan]}{DI_ZHI[item.data.zhi]}
              </Text>
            </View>
          ))}
        </View>
      </View>
    );
  };

  // 7. 渲染柱间关系分析
  const renderPillarRelationships = () => {
    if (!result) return null;

    const relationships = analyzePillarRelationships(
      result.siZhu.year.zhi,
      result.siZhu.month.zhi,
      result.siZhu.day.zhi,
      result.siZhu.hour.zhi
    );

    const relationshipColors: Record<string, string> = {
      harmony: '#27AE60',
      conflict: '#E74C3C',
      punishment: '#E67E22',
      destruction: '#C0392B',
      none: '#95A5A6',
    };

    const relationshipLabels: Record<string, string> = {
      harmony: '相合',
      conflict: '相冲',
      punishment: '相刑',
      destruction: '相破',
      none: '无关',
    };

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>柱间关系 (Pillar Relationships)</Text>
        <View style={styles.relationshipList}>
          {relationships.map((rel, index) => (
            <View key={index} style={styles.relationshipItem}>
              <View style={styles.relationshipPillars}>
                <Text style={styles.relationshipPillar}>{rel.pillar1}</Text>
                <View style={[styles.relationshipType, { backgroundColor: relationshipColors[rel.relationship] + '30' }]}>
                  <Text style={[styles.relationshipTypeText, { color: relationshipColors[rel.relationship] }]}>
                    {relationshipLabels[rel.relationship]}
                  </Text>
                </View>
                <Text style={styles.relationshipPillar}>{rel.pillar2}</Text>
              </View>
              <Text style={styles.relationshipDesc}>{rel.description}</Text>
            </View>
          ))}
        </View>
      </View>
    );
  };

  // 8. 渲染命宫分析
  const renderMingGong = () => {
    if (!result) return null;

    const mingGong = analyzeMingGong(result.siZhu.month.zhi, result.siZhu.hour.zhi);
    const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

    const fortuneColors: Record<string, string> = {
      excellent: '#27AE60',
      good: '#3498DB',
      average: '#F39C12',
      poor: '#E74C3C',
    };

    const fortuneLabels: Record<string, string> = {
      excellent: '优秀',
      good: '良好',
      average: '一般',
      poor: '较差',
    };

    return (
      <View style={styles.card}>
        <Text style={styles.cardTitle}>命宫分析 (Life Palace)</Text>
        <View style={styles.mingGongBox}>
          <View style={styles.mingGongHeader}>
            <Text style={styles.mingGongZhi}>{DI_ZHI[mingGong.mingGongZhi]}</Text>
            <View style={[styles.mingGongFortune, { backgroundColor: fortuneColors[mingGong.fortuneLevel] + '20' }]}>
              <Text style={[styles.mingGongFortuneText, { color: fortuneColors[mingGong.fortuneLevel] }]}>
                {fortuneLabels[mingGong.fortuneLevel]}
              </Text>
            </View>
          </View>
          <Text style={styles.mingGongDesc}>{mingGong.description}</Text>
          <View style={styles.mingGongCharacteristics}>
            <Text style={styles.mingGongCharTitle}>性格特征:</Text>
            <View style={styles.mingGongCharList}>
              {mingGong.characteristics.map((char, index) => (
                <View key={index} style={styles.mingGongCharItem}>
                  <Text style={styles.mingGongCharText}>{char}</Text>
                </View>
              ))}
            </View>
          </View>
        </View>
      </View>
    );
  };

  // 9. 渲染链端完整解盘
  const renderChainInterpretation = () => {
    if (loadingFull) {
      return (
        <View style={styles.loadingCard}>
          <ActivityIndicator color={THEME_COLOR} />
          <Text style={styles.loadingText}>正在从链上获取深度解析...</Text>
        </View>
      );
    }

    if (!fullInterpretation) return null;

    return (
      <View style={[styles.card, { borderColor: THEME_COLOR, borderWeight: 1 }]}>
        <View style={styles.chainHeader}>
          <Ionicons name="shield-checkmark" size={18} color={THEME_COLOR} />
          <Text style={styles.chainTitle}>链上深度解析</Text>
        </View>
        
        {/* 这里展示链端返回的复杂解析数据，可以根据实际数据结构扩展 */}
        <View style={styles.interpretationBox}>
          <Text style={styles.interpretationText}>
            该命盘已在区块链上完成完整校验。{fullInterpretation.summary || '暂无概要'}
          </Text>
          
          {fullInterpretation.analysis && (
            <View style={styles.analysisSection}>
              <Text style={styles.analysisSectionTitle}>命理建议</Text>
              <Text style={styles.analysisSectionContent}>{fullInterpretation.analysis}</Text>
            </View>
          )}
        </View>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>命盘详情</Text>
        <Pressable style={styles.moreButton} onPress={handleDelete}>
          <Ionicons name="trash-outline" size={22} color="#FF6B6B" />
        </Pressable>
      </View>

      {/* 模式切换 */}
      <View style={styles.modeToggleContainer}>
        <Pressable
          style={[styles.modeToggleButton, viewMode === 'traditional' && styles.modeToggleButtonActive]}
          onPress={() => setViewMode('traditional')}
        >
          <Text style={[styles.modeToggleText, viewMode === 'traditional' && styles.modeToggleTextActive]}>
            传统模式
          </Text>
        </Pressable>
        <Pressable
          style={[styles.modeToggleButton, viewMode === 'analysis' && styles.modeToggleButtonActive]}
          onPress={() => setViewMode('analysis')}
        >
          <Text style={[styles.modeToggleText, viewMode === 'analysis' && styles.modeToggleTextActive]}>
            分析模式
          </Text>
        </Pressable>
      </View>

      {/* 内容区 */}
      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
      >
        {viewMode === 'traditional' ? (
          // 传统模式：只显示基本信息和四柱
          <>
            {renderBasicInfo()}
            {renderSiZhuDetail()}
          </>
        ) : (
          // 分析模式：显示所有详细分析
          <>
            {renderBasicInfo()}
            {renderSiZhuDetail()}
            {renderChainInterpretation()}
            {renderWuXingAnalysis()}
            {renderDayMasterAnalysis()}
            {renderShiShenAnalysis()}
            {renderFavorableElements()}

            {/* 8 个可扩展功能 */}
            {renderDaYun()}
            {renderLiuNian()}
            {renderShenSha()}
            {renderGeJu()}
            {renderLuWei()}
            {renderNaYin()}
            {renderPillarRelationships()}
            {renderMingGong()}
          </>
        )}
      </ScrollView>

      {/* 操作按钮 */}
      {result?.id && result.id < 1000000000 && (
        <View style={styles.actionBar}>
          <Pressable style={styles.cacheButton} onPress={handleCacheInterpretation}>
            <Ionicons name="cloud-upload-outline" size={18} color={THEME_COLOR} />
            <Text style={styles.cacheButtonText}>缓存解盘</Text>
          </Pressable>
        </View>
      )}

      <UnlockWalletDialog
        visible={showUnlockDialog}
        onClose={() => setShowUnlockDialog(false)}
        onUnlock={handleWalletUnlocked}
      />

      <TransactionStatusDialog
        visible={showTxStatus}
        status={txStatus}
        onClose={() => setShowTxStatus(false)}
      />

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="divination" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  navBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backButton: {
    padding: 4,
  },
  moreButton: {
    padding: 4,
  },
  navTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  modeToggleContainer: {
    flexDirection: 'row',
    backgroundColor: '#FFF',
    paddingHorizontal: 16,
    paddingVertical: 12,
    gap: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  modeToggleButton: {
    flex: 1,
    paddingVertical: 8,
    paddingHorizontal: 16,
    borderRadius: 6,
    backgroundColor: '#F5F5F7',
    alignItems: 'center',
    justifyContent: 'center',
  },
  modeToggleButtonActive: {
    backgroundColor: THEME_COLOR,
  },
  modeToggleText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#666',
  },
  modeToggleTextActive: {
    color: '#FFF',
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 12,
    paddingBottom: 100,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
  },
  card: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  infoGrid: {
    gap: 12,
  },
  infoItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
  },
  infoLabel: {
    fontSize: 14,
    color: '#999',
  },
  infoValue: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  // 传统四柱样式
  traditionalSiZhu: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'flex-start',
    paddingVertical: 20,
    backgroundColor: '#FFFEF8',
    borderRadius: 8,
    marginBottom: 16,
  },
  traditionalPillar: {
    alignItems: 'center',
    gap: 6,
    width: 80,
  },
  shiShenTag: {
    backgroundColor: THEME_COLOR_LIGHT,
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 3,
    marginBottom: 4,
  },
  shiShenTagText: {
    fontSize: 11,
    color: THEME_COLOR,
    fontWeight: '600',
  },
  traditionalGanBox: {
    width: 60,
    height: 60,
    borderWidth: 2.5,
    borderRadius: 4,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FFF',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  traditionalGanText: {
    fontSize: 36,
    fontWeight: 'bold',
  },
  traditionalZhiBox: {
    width: 60,
    height: 60,
    borderWidth: 2.5,
    borderRadius: 4,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FFF',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  traditionalZhiText: {
    fontSize: 36,
    fontWeight: 'bold',
  },
  traditionalHiddenStems: {
    flexDirection: 'row',
    gap: 2,
    marginTop: 4,
  },
  traditionalHiddenStemText: {
    fontSize: 10,
    color: '#999',
    fontWeight: '500',
  },
  traditionalPillarLabel: {
    fontSize: 12,
    color: '#666',
    fontWeight: '600',
    marginTop: 4,
  },
  traditionalDayMasterMark: {
    position: 'absolute',
    top: -8,
    right: 8,
    backgroundColor: '#D32F2F',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 3,
  },
  traditionalDayMasterText: {
    fontSize: 9,
    color: '#FFF',
    fontWeight: 'bold',
  },
  detailedAnalysis: {
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#E0E0E0',
  },
  analysisTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  analysisItem: {
    marginBottom: 12,
  },
  analysisLabel: {
    fontSize: 13,
    fontWeight: '600',
    color: THEME_COLOR,
    marginBottom: 4,
  },
  analysisContent: {
    paddingLeft: 12,
    gap: 3,
  },
  analysisText: {
    fontSize: 12,
    color: '#666',
    lineHeight: 18,
  },
  wuxingAnalysis: {
    gap: 12,
  },
  wuxingAnalysisItem: {
    gap: 6,
  },
  wuxingAnalysisHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  wuxingAnalysisName: {
    fontSize: 14,
    fontWeight: '600',
  },
  wuxingAnalysisCount: {
    fontSize: 12,
    color: '#999',
  },
  wuxingAnalysisBar: {
    height: 16,
    backgroundColor: '#F0F0F0',
    borderRadius: 8,
    overflow: 'hidden',
  },
  wuxingAnalysisBarFill: {
    height: '100%',
    borderRadius: 8,
  },
  lackingText: {
    fontSize: 11,
    color: '#E74C3C',
    fontWeight: '500',
  },
  dayMasterBox: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 16,
  },
  dayMasterCircle: {
    width: 80,
    height: 80,
    borderWidth: 3,
    borderRadius: 40,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FAFAFA',
  },
  dayMasterText: {
    fontSize: 32,
    fontWeight: 'bold',
  },
  dayMasterInfo: {
    flex: 1,
    gap: 6,
  },
  dayMasterTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
  },
  dayMasterDesc: {
    fontSize: 12,
    color: '#666',
  },
  shiShenGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 12,
  },
  shiShenItem: {
    width: '48%',
    alignItems: 'center',
    gap: 8,
  },
  shiShenLabel: {
    fontSize: 12,
    color: '#999',
  },
  shiShenBox: {
    width: '100%',
    paddingVertical: 12,
    borderWidth: 2,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FAFAFA',
  },
  shiShenName: {
    fontSize: 14,
    fontWeight: '600',
  },
  shiShenDesc: {
    fontSize: 11,
    color: '#999',
    textAlign: 'center',
  },
  suggestionSection: {
    marginBottom: 16,
    paddingBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  suggestionTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  elementsList: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    marginBottom: 8,
  },
  elementTag: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
  },
  elementTagText: {
    fontSize: 12,
    fontWeight: '500',
  },
  suggestionText: {
    fontSize: 12,
    color: '#666',
    lineHeight: 18,
  },

  // ============================================================================
  // 8 个可扩展功能的样式
  // ============================================================================

  // 1. 大运分析样式
  cardDescription: {
    fontSize: 12,
    color: '#999',
    marginBottom: 12,
  },
  daYunList: {
    gap: 12,
  },
  daYunItem: {
    paddingHorizontal: 12,
    paddingVertical: 10,
    backgroundColor: '#F9F9F9',
    borderRadius: 6,
    borderLeftWidth: 3,
    borderLeftColor: THEME_COLOR,
  },
  daYunHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 6,
  },
  daYunAge: {
    fontSize: 13,
    fontWeight: '600',
    color: '#333',
  },
  daYunTag: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 3,
  },
  daYunTagText: {
    fontSize: 11,
    fontWeight: '500',
  },
  daYunDesc: {
    fontSize: 12,
    color: '#666',
  },

  // 2. 流年分析样式
  liuNianList: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 10,
  },
  liuNianItem: {
    width: '48%',
    paddingHorizontal: 12,
    paddingVertical: 10,
    backgroundColor: '#F9F9F9',
    borderRadius: 6,
    alignItems: 'center',
  },
  liuNianYear: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  liuNianYearText: {
    fontSize: 13,
    fontWeight: '600',
    color: '#333',
  },
  liuNianWuxing: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 3,
  },
  liuNianWuxingText: {
    fontSize: 11,
    fontWeight: '500',
  },

  // 3. 神煞分析样式
  shenShaSection: {
    marginBottom: 16,
    paddingBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  shenShaTitle: {
    fontSize: 13,
    fontWeight: '600',
    marginBottom: 10,
  },
  shenShaItem: {
    flexDirection: 'row',
    gap: 10,
    marginBottom: 10,
    paddingHorizontal: 10,
    paddingVertical: 8,
    backgroundColor: '#F9F9F9',
    borderRadius: 6,
  },
  shenShaIndicator: {
    width: 4,
    borderRadius: 2,
  },
  shenShaContent: {
    flex: 1,
  },
  shenShaName: {
    fontSize: 13,
    fontWeight: '600',
    color: '#333',
    marginBottom: 2,
  },
  shenShaDesc: {
    fontSize: 11,
    color: '#666',
  },

  // 4. 格局判断样式
  geJuBox: {
    paddingHorizontal: 12,
    paddingVertical: 12,
    backgroundColor: '#F9F9F9',
    borderRadius: 6,
  },
  geJuLevel: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
    alignSelf: 'flex-start',
    marginBottom: 8,
  },
  geJuLevelText: {
    fontSize: 12,
    fontWeight: '600',
  },
  geJuName: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 6,
  },
  geJuDesc: {
    fontSize: 12,
    color: '#666',
    lineHeight: 18,
    marginBottom: 10,
  },
  geJuCharacteristics: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  geJuCharItem: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    backgroundColor: '#FFF',
    borderWidth: 1,
    borderColor: '#E0E0E0',
    borderRadius: 4,
  },
  geJuCharText: {
    fontSize: 11,
    color: '#666',
  },

  // 5. 禄位分析样式
  luWeiBox: {
    paddingHorizontal: 12,
    paddingVertical: 12,
    backgroundColor: '#F9F9F9',
    borderRadius: 6,
  },
  luWeiDesc: {
    fontSize: 12,
    color: '#666',
    lineHeight: 18,
    marginBottom: 12,
  },
  luWeiPosition: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  luWeiLabel: {
    fontSize: 12,
    color: '#999',
  },
  luWeiValue: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME_COLOR,
  },

  // 6. 纳音五行样式
  nayinGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 10,
  },
  nayinItem: {
    width: '48%',
    paddingHorizontal: 12,
    paddingVertical: 10,
    backgroundColor: '#F9F9F9',
    borderRadius: 6,
    alignItems: 'center',
  },
  nayinLabel: {
    fontSize: 11,
    color: '#999',
    marginBottom: 4,
  },
  nayinValue: {
    fontSize: 13,
    fontWeight: '600',
    color: '#333',
    marginBottom: 2,
  },
  nayinGanZhi: {
    fontSize: 11,
    color: '#666',
  },

  // 7. 柱间关系样式
  relationshipList: {
    gap: 10,
  },
  relationshipItem: {
    paddingHorizontal: 12,
    paddingVertical: 10,
    backgroundColor: '#F9F9F9',
    borderRadius: 6,
  },
  relationshipPillars: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 6,
  },
  relationshipPillar: {
    fontSize: 12,
    fontWeight: '600',
    color: '#333',
  },
  relationshipType: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 3,
  },
  relationshipTypeText: {
    fontSize: 11,
    fontWeight: '500',
  },
  relationshipDesc: {
    fontSize: 11,
    color: '#666',
  },

  // 8. 命宫分析样式
  mingGongBox: {
    paddingHorizontal: 12,
    paddingVertical: 12,
    backgroundColor: '#F9F9F9',
    borderRadius: 6,
  },
  mingGongHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
    marginBottom: 10,
  },
  mingGongZhi: {
    fontSize: 28,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  mingGongFortune: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
  },
  mingGongFortuneText: {
    fontSize: 12,
    fontWeight: '600',
  },
  mingGongDesc: {
    fontSize: 12,
    color: '#666',
    lineHeight: 18,
    marginBottom: 10,
  },
  mingGongCharacteristics: {
    paddingTopWidth: 10,
    borderTopWidth: 1,
    borderTopColor: '#E0E0E0',
  },
  mingGongCharTitle: {
    fontSize: 12,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  mingGongCharList: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  mingGongCharItem: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    backgroundColor: '#FFF',
    borderWidth: 1,
    borderColor: '#E0E0E0',
    borderRadius: 4,
  },
  mingGongCharText: {
    fontSize: 11,
    color: '#666',
  },
  loadingCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 24,
    marginBottom: 12,
    alignItems: 'center',
    justifyContent: 'center',
    gap: 12,
  },
  chainHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 12,
    paddingBottom: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  chainTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  interpretationBox: {
    gap: 12,
  },
  interpretationText: {
    fontSize: 13,
    color: '#333',
    lineHeight: 20,
  },
  analysisSection: {
    backgroundColor: '#F9F9F9',
    padding: 12,
    borderRadius: 6,
  },
  analysisSectionTitle: {
    fontSize: 12,
    fontWeight: '600',
    color: '#666',
    marginBottom: 4,
  },
  analysisSectionContent: {
    fontSize: 13,
    color: '#333',
    lineHeight: 18,
  },
  actionBar: {
    flexDirection: 'row',
    justifyContent: 'center',
    paddingVertical: 12,
    paddingHorizontal: 16,
    backgroundColor: '#FFF',
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  cacheButton: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F8F4E8',
    paddingHorizontal: 20,
    paddingVertical: 10,
    borderRadius: 20,
    gap: 6,
  },
  cacheButtonText: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
});
