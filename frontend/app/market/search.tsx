// frontend/app/market/search.tsx

import React, { useState, useCallback, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  TouchableOpacity,
  ScrollView,
  SafeAreaView,
  StatusBar,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import storage from '@/lib/storage';
import { useMarketApi } from '@/divination/market/hooks';
import { ProviderCard } from '@/divination/market/components/ProviderCard';
import { LoadingSpinner, EmptyState } from '@/divination/market/components';
import { Button } from '@/components/common';
import { useAsync } from '@/hooks';
import { THEME } from '@/divination/market/theme';
import {
  SEARCH_HISTORY_KEY,
  MAX_HISTORY_ITEMS,
  HOT_KEYWORDS,
} from '@/divination/market/constants/market.constants';
import { Provider } from '@/divination/market/types';

export default function SearchScreen() {
  const router = useRouter();
  const { getProviders } = useMarketApi();
  const { execute, isLoading } = useAsync();

  const [keyword, setKeyword] = useState('');
  const [searchHistory, setSearchHistory] = useState<string[]>([]);
  const [searchResults, setSearchResults] = useState<Provider[]>([]);
  const [hasSearched, setHasSearched] = useState(false);

  // 加载搜索历史
  useEffect(() => {
    loadSearchHistory();
  }, []);

  const loadSearchHistory = async () => {
    try {
      const history = await storage.getItem(SEARCH_HISTORY_KEY);
      if (history) {
        setSearchHistory(JSON.parse(history));
      }
    } catch (err) {
      console.error('Load search history error:', err);
    }
  };

  const saveSearchHistory = async (newKeyword: string) => {
    try {
      const newHistory = [
        newKeyword,
        ...searchHistory.filter((k) => k !== newKeyword),
      ].slice(0, MAX_HISTORY_ITEMS);
      setSearchHistory(newHistory);
      await storage.setItem(SEARCH_HISTORY_KEY, JSON.stringify(newHistory));
    } catch (err) {
      console.error('Save search history error:', err);
    }
  };

  const clearSearchHistory = async () => {
    try {
      setSearchHistory([]);
      await storage.removeItem(SEARCH_HISTORY_KEY);
    } catch (err) {
      console.error('Clear search history error:', err);
    }
  };

  const handleSearch = useCallback(
    async (searchKeyword: string) => {
      if (!searchKeyword.trim()) return;

      setKeyword(searchKeyword);
      saveSearchHistory(searchKeyword);
      setHasSearched(true);

      await execute(async () => {
        const result = await getProviders({ keyword: searchKeyword });
        setSearchResults(result);
      });
    },
    [getProviders, execute]
  );

  const handleSubmit = () => {
    handleSearch(keyword);
  };

  const handleClearInput = () => {
    setKeyword('');
    setHasSearched(false);
    setSearchResults([]);
  };

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.card} />

      {/* 搜索栏 */}
      <View style={styles.searchHeader}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
          <Ionicons name="arrow-back" size={24} color={THEME.text} />
        </TouchableOpacity>
        <View style={styles.searchInputContainer}>
          <Ionicons name="search-outline" size={18} color={THEME.textTertiary} />
          <TextInput
            style={styles.searchInput}
            placeholder="搜索解卦师、擅长领域..."
            placeholderTextColor={THEME.textTertiary}
            value={keyword}
            onChangeText={setKeyword}
            onSubmitEditing={handleSubmit}
            returnKeyType="search"
            autoFocus
          />
          {keyword.length > 0 && (
            <TouchableOpacity onPress={handleClearInput}>
              <Ionicons name="close-circle" size={18} color={THEME.textTertiary} />
            </TouchableOpacity>
          )}
        </View>
        <TouchableOpacity onPress={handleSubmit} style={styles.searchBtn}>
          <Text style={styles.searchBtnText}>搜索</Text>
        </TouchableOpacity>
      </View>

      {!hasSearched ? (
        <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
          {/* 搜索历史 */}
          {searchHistory.length > 0 && (
            <View style={styles.section}>
              <View style={styles.sectionHeader}>
                <Text style={styles.sectionTitle}>搜索历史</Text>
                <TouchableOpacity onPress={clearSearchHistory}>
                  <Text style={styles.clearText}>清空</Text>
                </TouchableOpacity>
              </View>
              <View style={styles.tagsContainer}>
                {searchHistory.map((item, index) => (
                  <TouchableOpacity
                    key={index}
                    style={styles.tag}
                    onPress={() => handleSearch(item)}
                  >
                    <Text style={styles.tagText}>{item}</Text>
                  </TouchableOpacity>
                ))}
              </View>
            </View>
          )}

          {/* 热门搜索 */}
          <View style={styles.section}>
            <View style={styles.sectionHeader}>
              <Text style={styles.sectionTitle}>热门搜索</Text>
            </View>
            <View style={styles.tagsContainer}>
              {HOT_KEYWORDS.map((item, index) => (
                <TouchableOpacity
                  key={index}
                  style={styles.tag}
                  onPress={() => handleSearch(item)}
                >
                  <Text style={styles.tagText}>{item}</Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>
        </ScrollView>
      ) : (
        <ScrollView
          style={styles.content}
          contentContainerStyle={styles.resultsContent}
          showsVerticalScrollIndicator={false}
        >
          {isLoading ? (
            <LoadingSpinner text="搜索中..." />
          ) : searchResults.length === 0 ? (
            <EmptyState
              icon="search-outline"
              title="未找到结果"
              description={`未找到与"${keyword}"相关的解卦师`}
            />
          ) : (
            <>
              <Text style={styles.resultCount}>
                找到 {searchResults.length} 位解卦师
              </Text>
              {searchResults.map((provider) => (
                <ProviderCard key={provider.account} provider={provider} />
              ))}
            </>
          )}
        </ScrollView>
      )}
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  searchHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.card,
    paddingHorizontal: 12,
    paddingVertical: 10,
    gap: 8,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  backBtn: {
    padding: 4,
  },
  searchInputContainer: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.background,
    borderRadius: 8,
    paddingHorizontal: 10,
    height: 36,
    gap: 6,
  },
  searchInput: {
    flex: 1,
    fontSize: 14,
    color: THEME.text,
    paddingVertical: 0,
  },
  searchBtn: {
    paddingHorizontal: 12,
    paddingVertical: 8,
  },
  searchBtnText: {
    fontSize: 14,
    color: THEME.primary,
    fontWeight: '500',
  },
  content: {
    flex: 1,
  },
  section: {
    paddingHorizontal: 16,
    paddingTop: 20,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
  },
  clearText: {
    fontSize: 13,
    color: THEME.textTertiary,
  },
  tagsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 10,
  },
  tag: {
    backgroundColor: THEME.card,
    paddingHorizontal: 14,
    paddingVertical: 8,
    borderRadius: 16,
  },
  tagText: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  resultsContent: {
    padding: 16,
  },
  resultCount: {
    fontSize: 13,
    color: THEME.textTertiary,
    marginBottom: 12,
  },
});
