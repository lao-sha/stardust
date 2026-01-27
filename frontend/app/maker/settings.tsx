/**
 * 做市商设置页面
 * 路径: /maker/settings
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  TextInput,
  Switch,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore } from '@/stores/maker.store';
import { MakerService, ApplicationStatus } from '@/services/maker.service';
import { PremiumSlider } from '@/features/maker/components';
import { PageHeader } from '@/components/PageHeader';
import { Card, LoadingSpinner } from '@/components/common';

export default function SettingsPage() {
  const router = useRouter();
  const {
    makerApp,
    isLoading,
    fetchMakerInfo,
  } = useMakerStore();

  const [servicePaused, setServicePaused] = useState(false);
  const [buyPremiumBps, setBuyPremiumBps] = useState(0);
  const [sellPremiumBps, setSellPremiumBps] = useState(0);
  const [minAmount, setMinAmount] = useState('20');
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    fetchMakerInfo();
  }, []);

  useEffect(() => {
    if (makerApp) {
      setServicePaused(makerApp.servicePaused);
      setBuyPremiumBps(makerApp.buyPremiumBps);
      setSellPremiumBps(makerApp.sellPremiumBps);
      setMinAmount((Number(makerApp.minAmount) / 1e6).toString());
    }
  }, [makerApp]);

  const handleServiceToggle = (value: boolean) => {
    setServicePaused(value);
    setHasChanges(true);
    // TODO: 调用链上方法暂停/恢复服务
    Alert.alert(
      value ? '暂停服务' : '恢复服务',
      value ? '您的做市商服务已暂停，将不会接收新订单' : '您的做市商服务已恢复',
    );
  };

  const handleSave = () => {
    // TODO: 调用链上方法保存设置
    Alert.alert('保存成功', '设置已更新');
    setHasChanges(false);
  };

  if (isLoading && !makerApp) {
    return (
      <View style={styles.loadingContainer}>
        <LoadingSpinner text="加载中..." />
      </View>
    );
  }

  if (!makerApp || makerApp.status !== ApplicationStatus.Active) {
    return (
      <View style={styles.container}>
        <PageHeader title="做市商设置" showBack />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>您还不是做市商</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="做市商设置" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 服务状态 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>服务状态</Text>
          <Card style={styles.section}>
            <View style={styles.switchRow}>
              <View>
                <Text style={styles.switchLabel}>
                  当前状态: {servicePaused ? '🔴 已暂停' : '🟢 服务中'}
                </Text>
                <Text style={styles.switchDesc}>
                  {servicePaused ? '暂停后将不会接收新订单' : '正在接收订单'}
                </Text>
              </View>
              <Switch
                value={servicePaused}
                onValueChange={handleServiceToggle}
                trackColor={{ false: '#4CD964', true: '#FF3B30' }}
                thumbColor="#FFFFFF"
              />
            </View>
          </Card>
        </View>

        {/* 溢价设置 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>溢价设置</Text>
          <Card style={styles.section}>
            <PremiumSlider
              label="买入溢价 (Bridge)"
              value={buyPremiumBps}
              onChange={(v) => {
                setBuyPremiumBps(v);
                setHasChanges(true);
              }}
            />
            <PremiumSlider
              label="卖出溢价 (OTC)"
              value={sellPremiumBps}
              onChange={(v) => {
                setSellPremiumBps(v);
                setHasChanges(true);
              }}
            />
          </Card>
        </View>

        {/* 交易限额 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>交易限额</Text>
          <Card style={styles.section}>
            <Text style={styles.inputLabel}>最小交易金额</Text>
            <View style={styles.inputContainer}>
              <TextInput
                style={styles.input}
                value={minAmount}
                onChangeText={(v) => {
                  setMinAmount(v);
                  setHasChanges(true);
                }}
                keyboardType="decimal-pad"
              />
              <Text style={styles.inputSuffix}>USD</Text>
            </View>
          </Card>
        </View>

        {/* 收款信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>收款信息</Text>
          <Card style={styles.section}>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>TRON 地址</Text>
              <View style={styles.infoValueContainer}>
                <Text style={styles.infoValue} numberOfLines={1}>
                  {makerApp.tronAddress.slice(0, 10)}...{makerApp.tronAddress.slice(-8)}
                </Text>
                <TouchableOpacity style={styles.editButton}>
                  <Text style={styles.editButtonText}>修改</Text>
                </TouchableOpacity>
              </View>
            </View>

            <View style={styles.divider} />

            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>微信号</Text>
              <View style={styles.infoValueContainer}>
                <Text style={styles.infoValue}>{makerApp.wechatId}</Text>
                <TouchableOpacity style={styles.editButton}>
                  <Text style={styles.editButtonText}>修改</Text>
                </TouchableOpacity>
              </View>
            </View>

            {makerApp.epayNo && (
              <>
                <View style={styles.divider} />
                <View style={styles.infoRow}>
                  <Text style={styles.infoLabel}>EPAY 商户号</Text>
                  <View style={styles.infoValueContainer}>
                    <Text style={styles.infoValue}>{makerApp.epayNo}</Text>
                    <TouchableOpacity style={styles.editButton}>
                      <Text style={styles.editButtonText}>修改</Text>
                    </TouchableOpacity>
                  </View>
                </View>
              </>
            )}
          </Card>
        </View>

        {/* 做市商信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>做市商信息</Text>
          <Card style={styles.section}>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>做市商 ID</Text>
              <Text style={styles.infoValue}>#{makerApp.id}</Text>
            </View>
            <View style={styles.divider} />
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>姓名</Text>
              <Text style={styles.infoValue}>{makerApp.maskedFullName}</Text>
            </View>
            <View style={styles.divider} />
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>已服务用户</Text>
              <Text style={styles.infoValue}>{makerApp.usersServed.toLocaleString()}</Text>
            </View>
            <View style={styles.divider} />
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>注册时间</Text>
              <Text style={styles.infoValue}>
                {new Date(makerApp.createdAt * 1000).toLocaleDateString('zh-CN')}
              </Text>
            </View>
          </Card>
        </View>

        {/* 保存按钮 */}
        {hasChanges && (
          <TouchableOpacity style={styles.saveButton} onPress={handleSave}>
            <Text style={styles.saveButtonText}>保存设置</Text>
          </TouchableOpacity>
        )}

        {/* 危险操作 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>危险操作</Text>
          <TouchableOpacity
            style={styles.dangerButton}
            onPress={() => {
              Alert.alert(
                '注销做市商',
                '注销后押金将在冷却期后退还。确定要注销吗？',
                [
                  { text: '取消', style: 'cancel' },
                  { text: '确定注销', style: 'destructive', onPress: () => {} },
                ]
              );
            }}
          >
            <Text style={styles.dangerButtonText}>注销做市商</Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#8E8E93',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  section: {
    marginBottom: 24,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#8E8E93',
    marginBottom: 8,
    marginLeft: 4,
  },
  section: {
    marginBottom: 16,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  switchLabel: {
    fontSize: 15,
    fontWeight: '500',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  switchDesc: {
    fontSize: 13,
    color: '#8E8E93',
  },
  inputLabel: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 8,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 12,
  },
  input: {
    flex: 1,
    fontSize: 16,
    color: '#1C1C1E',
    paddingVertical: 12,
  },
  inputSuffix: {
    fontSize: 14,
    color: '#8E8E93',
    marginLeft: 8,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 4,
  },
  infoLabel: {
    fontSize: 14,
    color: '#8E8E93',
  },
  infoValueContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  infoValue: {
    fontSize: 14,
    color: '#1C1C1E',
    maxWidth: 150,
  },
  editButton: {
    marginLeft: 8,
    paddingHorizontal: 8,
    paddingVertical: 4,
  },
  editButtonText: {
    fontSize: 13,
    color: '#007AFF',
  },
  divider: {
    height: 1,
    backgroundColor: '#F2F2F7',
    marginVertical: 12,
  },
  saveButton: {
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginBottom: 24,
  },
  saveButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  dangerButton: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#FF3B30',
  },
  dangerButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FF3B30',
  },
});
