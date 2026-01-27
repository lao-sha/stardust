/**
 * 用户资料状态管理
 */

import { create } from 'zustand';
import { getChatService, initChatService } from '@/services/chat.service';
import type {
  ChatUserId,
  ChatUserProfile,
  UserStatus,
  PrivacySettings,
} from '@/features/chat/types';

interface UserState {
  // 状态
  isRegistered: boolean;
  myChatUserId: ChatUserId | null;
  myProfile: ChatUserProfile | null;
  isLoading: boolean;
  error: string | null;

  // 操作
  initialize: (address: string) => Promise<void>;
  register: (nickname?: string) => Promise<ChatUserId>;
  updateProfile: (params: {
    nickname?: string;
    avatarCid?: string;
    signature?: string;
  }) => Promise<void>;
  setStatus: (status: UserStatus) => Promise<void>;
  updatePrivacySettings: (settings: Partial<PrivacySettings>) => Promise<void>;
  searchUser: (chatUserId: ChatUserId) => Promise<ChatUserProfile | null>;
  refresh: () => Promise<void>;
}

export const useUserStore = create<UserState>()((set, get) => ({
  isRegistered: false,
  myChatUserId: null,
  myProfile: null,
  isLoading: false,
  error: null,

  initialize: async (address: string) => {
    set({ isLoading: true, error: null });

    try {
      const service = initChatService(address);
      await service.init();

      // 从链上查询当前用户的 ChatUserId
      const chatUserId = await service.getMyChatUserId();
      
      if (chatUserId !== null) {
        // 用户已注册，获取完整资料
        const profile = await service.getMyProfile();
        set({
          isRegistered: true,
          myChatUserId: chatUserId,
          myProfile: profile,
          isLoading: false,
        });
      } else {
        // 用户未注册
        set({
          isRegistered: false,
          myChatUserId: null,
          myProfile: null,
          isLoading: false,
        });
      }
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  register: async (nickname?: string) => {
    set({ isLoading: true, error: null });

    try {
      const service = getChatService();
      const chatUserId = await service.registerChatUser(nickname);
      const profile = await service.getUserProfile(chatUserId);

      set({
        isRegistered: true,
        myChatUserId: chatUserId,
        myProfile: profile,
      });

      return chatUserId;
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  updateProfile: async (params) => {
    set({ isLoading: true, error: null });

    try {
      const service = getChatService();
      await service.updateProfile(params);

      // 刷新资料
      const { myChatUserId } = get();
      if (myChatUserId) {
        const profile = await service.getUserProfile(myChatUserId);
        set({ myProfile: profile });
      }
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  setStatus: async (status: UserStatus) => {
    try {
      const service = getChatService();
      await service.setUserStatus(status);

      set((state) => ({
        myProfile: state.myProfile
          ? { ...state.myProfile, status }
          : null,
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  updatePrivacySettings: async (settings: Partial<PrivacySettings>) => {
    set({ isLoading: true, error: null });

    try {
      const service = getChatService();
      await service.updatePrivacySettings(settings);

      set((state) => ({
        myProfile: state.myProfile
          ? {
              ...state.myProfile,
              privacySettings: {
                ...state.myProfile.privacySettings,
                ...settings,
              },
            }
          : null,
      }));
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  searchUser: async (chatUserId: ChatUserId) => {
    try {
      const service = getChatService();
      return await service.getUserProfile(chatUserId);
    } catch (error) {
      set({ error: (error as Error).message });
      return null;
    }
  },

  refresh: async () => {
    const { myChatUserId } = get();
    if (!myChatUserId) return;

    try {
      const service = getChatService();
      const profile = await service.getUserProfile(myChatUserId);
      set({ myProfile: profile });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },
}));
