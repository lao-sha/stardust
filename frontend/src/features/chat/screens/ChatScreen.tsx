/**
 * 聊天详情页面
 */

import React, { useEffect, useCallback, useState } from 'react';
import {
  View,
  StyleSheet,
  KeyboardAvoidingView,
  Platform,
  Alert,
} from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useChatStore } from '@/stores/chat.store';
import { ChatHeader } from '../components/ChatHeader';
import { MessageList } from '../components/MessageList';
import { ChatInput } from '../components/ChatInput';
import { uploadImageToIpfs } from '@/services/ipfs.service';

export function ChatScreen() {
  const { sessionId } = useLocalSearchParams<{ sessionId: string }>();
  const router = useRouter();

  const {
    currentSession,
    messages,
    isLoading,
    selectSession,
    loadMessages,
    sendMessage,
    markSessionAsRead,
    retryMessage,
    removeFailedMessage,
  } = useChatStore();

  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);

  const sessionMessages = messages[sessionId || ''] || [];

  useEffect(() => {
    if (sessionId) {
      selectSession(sessionId);
      loadMessages(sessionId);
      markSessionAsRead(sessionId);
    }
  }, [sessionId, selectSession, loadMessages, markSessionAsRead]);

  const handleSend = useCallback(
    async (content: string, imageUri?: string) => {
      if (!currentSession) return;

      try {
        // 如果有图片，发送图片消息
        if (imageUri) {
          // 上传图片到 IPFS 并获取 CID
          try {
            const imageCid = await uploadImageToIpfs(imageUri);
            await sendMessage(currentSession.peerAddress, imageCid, 1); // MessageType.Image = 1
          } catch (uploadError) {
            console.error('Image upload failed:', uploadError);
            Alert.alert('图片上传失败', '无法上传图片到 IPFS，请稍后重试');
            return;
          }
        } else if (content) {
          // 发送文本消息
          await sendMessage(currentSession.peerAddress, content, 0); // MessageType.Text = 0
        }
      } catch (error) {
        console.error('Send message failed:', error);
        Alert.alert('发送失败', (error as Error).message);
      }
    },
    [currentSession, sendMessage]
  );

  const handleLoadMore = useCallback(async () => {
    if (!sessionId || isLoading || !hasMore) return;

    const newOffset = offset + 20;
    const newMessages = await loadMessages(sessionId, newOffset);

    if (newMessages.length < 20) {
      setHasMore(false);
    }
    setOffset(newOffset);
  }, [sessionId, offset, isLoading, hasMore, loadMessages]);

  const handleBack = useCallback(() => {
    router.back();
  }, [router]);

  const handleMore = useCallback(() => {
    Alert.alert('更多操作', undefined, [
      {
        text: '查看资料',
        onPress: () => {
          // TODO: 跳转到用户资料页
        },
      },
      {
        text: '清空聊天记录',
        style: 'destructive',
        onPress: () => {
          Alert.alert('确认清空', '确定要清空聊天记录吗？此操作不可恢复。', [
            { text: '取消', style: 'cancel' },
            {
              text: '清空',
              style: 'destructive',
              onPress: () => {
                // TODO: 实现清空聊天记录
              },
            },
          ]);
        },
      },
      {
        text: '拉黑用户',
        style: 'destructive',
        onPress: () => {
          Alert.alert('确认拉黑', '拉黑后将无法收到对方的消息。', [
            { text: '取消', style: 'cancel' },
            {
              text: '拉黑',
              style: 'destructive',
              onPress: () => {
                // TODO: 实现拉黑用户
              },
            },
          ]);
        },
      },
      { text: '取消', style: 'cancel' },
    ]);
  }, []);

  const handleRetry = useCallback(
    async (tempId: string) => {
      try {
        await retryMessage(tempId);
      } catch (error) {
        Alert.alert('重试失败', (error as Error).message);
      }
    },
    [retryMessage]
  );

  const handleDelete = useCallback(
    (tempId: string) => {
      Alert.alert('删除消息', '确定要删除这条消息吗？', [
        { text: '取消', style: 'cancel' },
        {
          text: '删除',
          style: 'destructive',
          onPress: () => removeFailedMessage(tempId),
        },
      ]);
    },
    [removeFailedMessage]
  );

  if (!currentSession) {
    return null;
  }

  const displayTitle =
    currentSession.peerAlias ||
    currentSession.peerProfile?.nickname ||
    (currentSession.peerChatId
      ? `ID: ${currentSession.peerChatId}`
      : currentSession.peerAddress);

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={Platform.OS === 'ios' ? 0 : 0}
    >
      <ChatHeader
        title={displayTitle}
        profile={currentSession.peerProfile}
        onBack={handleBack}
        onMore={handleMore}
      />

      <View style={styles.content}>
        <MessageList
          messages={sessionMessages}
          isLoading={isLoading}
          onLoadMore={handleLoadMore}
          hasMore={hasMore}
          onRetry={handleRetry}
          onDelete={handleDelete}
        />
      </View>

      <ChatInput onSend={handleSend} />
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  content: {
    flex: 1,
  },
});
