/**
 * 星尘玄鉴 - 交易确认对话框
 * 
 * 安全特性：
 * - 显示完整交易详情（类型、金额、接收方、Gas费）
 * - 交易模拟预览（预估结果）
 * - 风险提示
 * - 密码确认
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  TouchableOpacity,
  Modal,
  ActivityIndicator,
  ScrollView,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';

// ==================== 类型定义 ====================

/**
 * 交易详情
 */
export interface TransactionDetails {
  /** 交易类型（用于显示） */
  type: string;
  /** 交易描述 */
  description: string;
  /** 调用的模块 */
  module: string;
  /** 调用的方法 */
  method: string;
  /** 交易参数（用于显示） */
  params: TransactionParam[];
  /** 预估 Gas 费（DUST） */
  estimatedFee?: string;
  /** 转账金额（如果有） */
  amount?: string;
  /** 接收方地址（如果有） */
  recipient?: string;
  /** 风险等级 */
  riskLevel: 'low' | 'medium' | 'high';
  /** 风险提示 */
  riskWarnings?: string[];
}

/**
 * 交易参数
 */
export interface TransactionParam {
  name: string;
  value: string;
  /** 是否为敏感信息（如地址，需要截断显示） */
  sensitive?: boolean;
}

/**
 * 模拟结果
 */
export interface SimulationResult {
  success: boolean;
  /** 预估 Gas 费 */
  estimatedFee: string;
  /** 预估结果描述 */
  resultDescription?: string;
  /** 错误信息（如果模拟失败） */
  error?: string;
}

/**
 * 对话框属性
 */
interface TransactionConfirmDialogProps {
  visible: boolean;
  transaction: TransactionDetails | null;
  simulation?: SimulationResult | null;
  isSimulating?: boolean;
  onConfirm: (password: string) => void;
  onCancel: () => void;
}

// ==================== 主题色 ====================

const THEME = {
  primary: '#B2955D',
  background: '#FFFFFF',
  surface: '#F5F5F7',
  text: '#1A1A1A',
  textSecondary: '#666666',
  border: '#E5E5EA',
  success: '#34C759',
  warning: '#FF9500',
  error: '#FF3B30',
};

// ==================== 组件实现 ====================

export const TransactionConfirmDialog: React.FC<TransactionConfirmDialogProps> = ({
  visible,
  transaction,
  simulation,
  isSimulating = false,
  onConfirm,
  onCancel,
}) => {
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isConfirming, setIsConfirming] = useState(false);

  // 重置状态
  useEffect(() => {
    if (!visible) {
      setPassword('');
      setShowPassword(false);
      setIsConfirming(false);
    }
  }, [visible]);

  const handleConfirm = async () => {
    if (!password) return;
    
    setIsConfirming(true);
    try {
      await onConfirm(password);
    } finally {
      setIsConfirming(false);
    }
  };

  const getRiskColor = (level: 'low' | 'medium' | 'high') => {
    switch (level) {
      case 'low': return THEME.success;
      case 'medium': return THEME.warning;
      case 'high': return THEME.error;
    }
  };

  const getRiskLabel = (level: 'low' | 'medium' | 'high') => {
    switch (level) {
      case 'low': return '低风险';
      case 'medium': return '中风险';
      case 'high': return '高风险';
    }
  };

  const formatAddress = (address: string) => {
    if (address.length <= 16) return address;
    return `${address.slice(0, 8)}...${address.slice(-8)}`;
  };

  if (!transaction) return null;

  return (
    <Modal
      visible={visible}
      transparent
      animationType="slide"
      onRequestClose={onCancel}
    >
      <View style={styles.overlay}>
        <View style={styles.dialog}>
          {/* 标题栏 */}
          <View style={styles.header}>
            <View style={styles.headerLeft}>
              <Ionicons name="shield-checkmark" size={24} color={THEME.primary} />
              <Text style={styles.title}>确认交易</Text>
            </View>
            <TouchableOpacity onPress={onCancel} style={styles.closeButton}>
              <Ionicons name="close" size={24} color={THEME.textSecondary} />
            </TouchableOpacity>
          </View>

          <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
            {/* 交易类型 */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>交易类型</Text>
              <View style={styles.typeCard}>
                <Text style={styles.typeText}>{transaction.type}</Text>
                <Text style={styles.typeDescription}>{transaction.description}</Text>
              </View>
            </View>

            {/* 交易详情 */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>交易详情</Text>
              <View style={styles.detailsCard}>
                {/* 模块和方法 */}
                <View style={styles.detailRow}>
                  <Text style={styles.detailLabel}>调用</Text>
                  <Text style={styles.detailValue}>
                    {transaction.module}.{transaction.method}
                  </Text>
                </View>

                {/* 金额（如果有） */}
                {transaction.amount && (
                  <View style={styles.detailRow}>
                    <Text style={styles.detailLabel}>金额</Text>
                    <Text style={[styles.detailValue, styles.amountText]}>
                      {transaction.amount} DUST
                    </Text>
                  </View>
                )}

                {/* 接收方（如果有） */}
                {transaction.recipient && (
                  <View style={styles.detailRow}>
                    <Text style={styles.detailLabel}>接收方</Text>
                    <Text style={styles.detailValue}>
                      {formatAddress(transaction.recipient)}
                    </Text>
                  </View>
                )}

                {/* 参数列表 */}
                {transaction.params.map((param, index) => (
                  <View key={index} style={styles.detailRow}>
                    <Text style={styles.detailLabel}>{param.name}</Text>
                    <Text style={styles.detailValue} numberOfLines={1}>
                      {param.sensitive ? formatAddress(param.value) : param.value}
                    </Text>
                  </View>
                ))}
              </View>
            </View>

            {/* Gas 费预估 */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>费用预估</Text>
              <View style={styles.feeCard}>
                {isSimulating ? (
                  <View style={styles.simulatingRow}>
                    <ActivityIndicator size="small" color={THEME.primary} />
                    <Text style={styles.simulatingText}>正在模拟交易...</Text>
                  </View>
                ) : simulation ? (
                  <>
                    <View style={styles.feeRow}>
                      <Text style={styles.feeLabel}>预估 Gas 费</Text>
                      <Text style={styles.feeValue}>{simulation.estimatedFee} DUST</Text>
                    </View>
                    {simulation.resultDescription && (
                      <Text style={styles.simulationResult}>
                        {simulation.resultDescription}
                      </Text>
                    )}
                    {simulation.error && (
                      <View style={styles.simulationError}>
                        <Ionicons name="warning" size={16} color={THEME.error} />
                        <Text style={styles.simulationErrorText}>
                          模拟失败: {simulation.error}
                        </Text>
                      </View>
                    )}
                  </>
                ) : (
                  <View style={styles.feeRow}>
                    <Text style={styles.feeLabel}>预估 Gas 费</Text>
                    <Text style={styles.feeValue}>
                      {transaction.estimatedFee || '计算中...'} DUST
                    </Text>
                  </View>
                )}
              </View>
            </View>

            {/* 风险提示 */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>风险评估</Text>
              <View style={[
                styles.riskCard,
                { borderLeftColor: getRiskColor(transaction.riskLevel) }
              ]}>
                <View style={styles.riskHeader}>
                  <View style={[
                    styles.riskBadge,
                    { backgroundColor: getRiskColor(transaction.riskLevel) }
                  ]}>
                    <Text style={styles.riskBadgeText}>
                      {getRiskLabel(transaction.riskLevel)}
                    </Text>
                  </View>
                </View>
                {transaction.riskWarnings && transaction.riskWarnings.length > 0 && (
                  <View style={styles.riskWarnings}>
                    {transaction.riskWarnings.map((warning, index) => (
                      <View key={index} style={styles.warningRow}>
                        <Ionicons 
                          name="alert-circle" 
                          size={14} 
                          color={getRiskColor(transaction.riskLevel)} 
                        />
                        <Text style={styles.warningText}>{warning}</Text>
                      </View>
                    ))}
                  </View>
                )}
              </View>
            </View>

            {/* 密码输入 */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>确认密码</Text>
              <View style={styles.passwordContainer}>
                <TextInput
                  style={styles.passwordInput}
                  value={password}
                  onChangeText={setPassword}
                  placeholder="请输入钱包密码"
                  placeholderTextColor={THEME.textSecondary}
                  secureTextEntry={!showPassword}
                  autoCapitalize="none"
                  autoCorrect={false}
                />
                <TouchableOpacity
                  style={styles.eyeButton}
                  onPress={() => setShowPassword(!showPassword)}
                >
                  <Ionicons
                    name={showPassword ? 'eye-off' : 'eye'}
                    size={20}
                    color={THEME.textSecondary}
                  />
                </TouchableOpacity>
              </View>
            </View>
          </ScrollView>

          {/* 底部按钮 */}
          <View style={styles.footer}>
            <TouchableOpacity
              style={styles.cancelButton}
              onPress={onCancel}
              disabled={isConfirming}
            >
              <Text style={styles.cancelButtonText}>取消</Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={[
                styles.confirmButton,
                (!password || isConfirming || (simulation?.success === false)) && 
                  styles.confirmButtonDisabled
              ]}
              onPress={handleConfirm}
              disabled={!password || isConfirming || simulation?.success === false}
            >
              {isConfirming ? (
                <ActivityIndicator color="#FFFFFF" size="small" />
              ) : (
                <>
                  <Ionicons name="checkmark-circle" size={20} color="#FFFFFF" />
                  <Text style={styles.confirmButtonText}>确认签名</Text>
                </>
              )}
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </Modal>
  );
};

// ==================== 样式 ====================

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'flex-end',
  },
  dialog: {
    backgroundColor: THEME.background,
    borderTopLeftRadius: 24,
    borderTopRightRadius: 24,
    maxHeight: '90%',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: THEME.border,
  },
  headerLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
  },
  closeButton: {
    padding: 4,
  },
  content: {
    padding: 16,
    maxHeight: 400,
  },
  section: {
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.textSecondary,
    marginBottom: 8,
  },
  typeCard: {
    backgroundColor: THEME.surface,
    borderRadius: 12,
    padding: 16,
  },
  typeText: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 4,
  },
  typeDescription: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  detailsCard: {
    backgroundColor: THEME.surface,
    borderRadius: 12,
    padding: 12,
  },
  detailRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: THEME.border,
  },
  detailLabel: {
    fontSize: 14,
    color: THEME.textSecondary,
    flex: 1,
  },
  detailValue: {
    fontSize: 14,
    color: THEME.text,
    fontWeight: '500',
    flex: 2,
    textAlign: 'right',
  },
  amountText: {
    color: THEME.primary,
    fontWeight: '600',
  },
  feeCard: {
    backgroundColor: THEME.surface,
    borderRadius: 12,
    padding: 16,
  },
  feeRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  feeLabel: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  feeValue: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.text,
  },
  simulatingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  simulatingText: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  simulationResult: {
    fontSize: 13,
    color: THEME.success,
    marginTop: 8,
  },
  simulationError: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    marginTop: 8,
  },
  simulationErrorText: {
    fontSize: 13,
    color: THEME.error,
    flex: 1,
  },
  riskCard: {
    backgroundColor: THEME.surface,
    borderRadius: 12,
    padding: 16,
    borderLeftWidth: 4,
  },
  riskHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  riskBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  riskBadgeText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  riskWarnings: {
    marginTop: 12,
    gap: 8,
  },
  warningRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: 6,
  },
  warningText: {
    fontSize: 13,
    color: THEME.textSecondary,
    flex: 1,
  },
  passwordContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.surface,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  passwordInput: {
    flex: 1,
    padding: 14,
    fontSize: 16,
    color: THEME.text,
  },
  eyeButton: {
    padding: 14,
  },
  footer: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
    borderTopWidth: 1,
    borderTopColor: THEME.border,
  },
  cancelButton: {
    flex: 1,
    backgroundColor: THEME.surface,
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  cancelButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.textSecondary,
  },
  confirmButton: {
    flex: 2,
    backgroundColor: THEME.primary,
    borderRadius: 12,
    paddingVertical: 14,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
  },
  confirmButtonDisabled: {
    opacity: 0.5,
  },
  confirmButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
});

export default TransactionConfirmDialog;
