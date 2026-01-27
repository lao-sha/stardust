/**
 * 聊天服务
 * 与后端 pallet-chat 交互，处理消息发送、接收、加密解密
 */

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/api';
import {
  deriveSharedKey,
  encryptMessage,
  decryptMessage,
  generateX25519KeyPair,
  publicKeyToHex,
  hexToPublicKey,
} from './crypto.service';
import { uploadToIpfs, downloadFromIpfs } from './ipfs.service';
import { getSecureValue, setSecureValue } from '@/lib/keystore';
import type {
  Message,
  Session,
  MessageType,
  ChatUserProfile,
  ChatUserId,
  UserStatus,
  PrivacySettings,
} from '@/features/chat/types';

const CHAT_PRIVATE_KEY_STORAGE_KEY = 'chat_x25519_private_key';
const CHAT_PUBLIC_KEY_STORAGE_KEY = 'chat_x25519_public_key';

export class ChatService {
  private api: ApiPromise | null = null;
  public myAddress: string;
  private sharedKeys: Map<string, Uint8Array> = new Map();
  private unsubscribe: (() => void) | null = null;
  private myPrivateKey: Uint8Array | null = null;
  private myPublicKey: Uint8Array | null = null;

  constructor(myAddress: string) {
    this.myAddress = myAddress;
  }

  async init(): Promise<void> {
    this.api = await getApi();
    await this.loadOrGenerateKeyPair();
  }

  /**
   * 加载或生成 x25519 密钥对
   */
  private async loadOrGenerateKeyPair(): Promise<void> {
    try {
      const storedPrivateKey = await getSecureValue(CHAT_PRIVATE_KEY_STORAGE_KEY);
      const storedPublicKey = await getSecureValue(CHAT_PUBLIC_KEY_STORAGE_KEY);

      if (storedPrivateKey && storedPublicKey) {
        this.myPrivateKey = hexToPublicKey(storedPrivateKey);
        this.myPublicKey = hexToPublicKey(storedPublicKey);
      } else {
        // 生成新的密钥对
        const { privateKey, publicKey } = generateX25519KeyPair();
        this.myPrivateKey = privateKey;
        this.myPublicKey = publicKey;

        // 安全存储
        await setSecureValue(
          CHAT_PRIVATE_KEY_STORAGE_KEY,
          publicKeyToHex(privateKey)
        );
        await setSecureValue(
          CHAT_PUBLIC_KEY_STORAGE_KEY,
          publicKeyToHex(publicKey)
        );
      }
    } catch (error) {
      console.error('Failed to load/generate key pair:', error);
      throw error;
    }
  }

  /**
   * 获取自己的公钥（用于分享给其他用户）
   */
  getMyPublicKey(): Uint8Array | null {
    return this.myPublicKey;
  }

  /**
   * 获取自己的公钥（十六进制格式）
   */
  getMyPublicKeyHex(): string | null {
    return this.myPublicKey ? publicKeyToHex(this.myPublicKey) : null;
  }

  /**
   * 发送消息
   */
  async sendMessage(
    receiver: string,
    content: string,
    msgType: number = 0,
    sessionId?: string
  ): Promise<{ msgId: number; sessionId: string; cid: string; blockNumber: number }> {
    if (!this.api) throw new Error('API not initialized');

    // 1. 获取或派生共享密钥
    const sharedKey = await this.getSharedKey(receiver);

    // 2. 加密消息
    const encrypted = await encryptMessage(content, sharedKey);

    // 3. 上传到 IPFS
    const cid = await uploadToIpfs(encrypted);

    // 4. 发送链上交易
    const tx = this.api.tx.chat.sendMessage(
      receiver,
      cid,
      msgType,
      sessionId || null
    );

    return new Promise((resolve, reject) => {
      tx.signAndSend(this.myAddress, ({ status, events }) => {
        if (status.isInBlock) {
          const blockNumber = status.asInBlock.toNumber();
          for (const { event } of events) {
            if (this.api!.events.chat.MessageSent.is(event)) {
              const [msgId, sessionIdResult] = event.data;
              resolve({
                msgId: msgId.toNumber(),
                sessionId: sessionIdResult.toHex(),
                cid,
                blockNumber,
              });
              return;
            }
          }
          reject(new Error('MessageSent event not found'));
        }
        if (status.isFinalized) {
          reject(new Error('Transaction finalized without MessageSent event'));
        }
      }).catch(reject);
    });
  }

  /**
   * 获取会话列表
   */
  async getSessions(): Promise<Session[]> {
    if (!this.api) throw new Error('API not initialized');

    const entries = await this.api.query.chat.userSessions.entries(
      this.myAddress
    );
    const sessions: Session[] = [];

    for (const [key] of entries) {
      const sessionId = key.args[1].toHex();
      const session = await this.getSessionDetail(sessionId);
      if (session) sessions.push(session);
    }

    // 按最后活跃时间排序
    return sessions.sort((a, b) => b.lastActive - a.lastActive);
  }

  /**
   * 获取会话详情
   */
  async getSessionDetail(sessionId: string): Promise<Session | null> {
    if (!this.api) return null;

    const sessionData = await this.api.query.chat.sessions(sessionId);
    if (sessionData.isNone) return null;

    const session = sessionData.unwrap();
    const participants = session.participants.map((p: any) => p.toString());
    const peerAddress =
      participants.find((p: string) => p !== this.myAddress) || '';

    // 获取未读数
    const unreadCount = await this.api.query.chat.unreadCount([
      this.myAddress,
      sessionId,
    ]);

    return {
      id: sessionId,
      participants,
      peerAddress,
      lastActive: session.lastActive.toNumber(),
      unreadCount: unreadCount.toNumber(),
      isArchived: session.isArchived.valueOf(),
      createdAt: session.createdAt.toNumber(),
    };
  }

  /**
   * 获取会话消息（分页）
   */
  async getMessages(
    sessionId: string,
    offset: number = 0,
    limit: number = 20
  ): Promise<Message[]> {
    if (!this.api) throw new Error('API not initialized');

    const entries = await this.api.query.chat.sessionMessages.entries(
      sessionId
    );
    const msgIds = entries.map(([key]) => key.args[1].toNumber());

    // 按 ID 倒序排列（最新的在前）
    msgIds.sort((a, b) => b - a);

    const messages: Message[] = [];
    const targetIds = msgIds.slice(offset, offset + limit);

    for (const msgId of targetIds) {
      const msg = await this.getMessageDetail(msgId);
      if (msg) messages.push(msg);
    }

    // 返回时按时间正序（旧的在前）
    return messages.reverse();
  }

  /**
   * 获取消息详情并解密
   */
  async getMessageDetail(msgId: number): Promise<Message | null> {
    if (!this.api) return null;

    const msgData = await this.api.query.chat.messages(msgId);
    if (msgData.isNone) return null;

    const msg = msgData.unwrap();
    const sender = msg.sender.toString();
    const receiver = msg.receiver.toString();
    const isMine = sender === this.myAddress;
    const peerAddress = isMine ? receiver : sender;

    // 检查是否被删除
    if (isMine && msg.isDeletedBySender.valueOf()) return null;
    if (!isMine && msg.isDeletedByReceiver.valueOf()) return null;

    // 解密消息内容
    const cid = msg.contentCid.toUtf8();
    let content = '';

    try {
      const encrypted = await downloadFromIpfs(cid);
      const sharedKey = await this.getSharedKey(peerAddress);
      content = await decryptMessage(encrypted, sharedKey);
    } catch (error) {
      console.error('Failed to decrypt message:', error);
      content = '[无法解密]';
    }

    return {
      id: msgId,
      sessionId: msg.sessionId.toHex(),
      sender,
      receiver,
      senderChatId: msg.senderChatId.isSome
        ? msg.senderChatId.unwrap().toNumber()
        : undefined,
      receiverChatId: msg.receiverChatId.isSome
        ? msg.receiverChatId.unwrap().toNumber()
        : undefined,
      content,
      contentCid: cid,
      msgType: msg.msgType.toNumber() as MessageType,
      sentAt: msg.sentAt.toNumber(),
      isRead: msg.isRead.valueOf(),
      isDeletedBySender: msg.isDeletedBySender.valueOf(),
      isDeletedByReceiver: msg.isDeletedByReceiver.valueOf(),
      isMine,
      status: 'sent',
      replyTo: msg.replyTo.isSome ? msg.replyTo.unwrap().toNumber() : undefined,
    };
  }

  /**
   * 标记消息已读
   */
  async markAsRead(messageIds: number[]): Promise<void> {
    if (!this.api || messageIds.length === 0) return;

    const tx = this.api.tx.chat.markBatchAsRead(messageIds);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 标记整个会话已读
   */
  async markSessionAsRead(sessionId: string): Promise<void> {
    if (!this.api) return;

    const tx = this.api.tx.chat.markSessionAsRead(sessionId);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 删除消息（软删除）
   */
  async deleteMessage(msgId: number): Promise<void> {
    if (!this.api) return;

    const tx = this.api.tx.chat.deleteMessage(msgId);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 归档会话
   */
  async archiveSession(sessionId: string): Promise<void> {
    if (!this.api) return;

    const tx = this.api.tx.chat.archiveSession(sessionId);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 拉黑用户
   */
  async blockUser(address: string): Promise<void> {
    if (!this.api) return;

    const tx = this.api.tx.chat.blockUser(address);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 解除拉黑
   */
  async unblockUser(address: string): Promise<void> {
    if (!this.api) return;

    const tx = this.api.tx.chat.unblockUser(address);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 检查是否被拉黑
   */
  async isBlocked(address: string): Promise<boolean> {
    if (!this.api) return false;

    const result = await this.api.query.chat.blacklist(address, this.myAddress);
    return result.isSome;
  }

  /**
   * 注册聊天用户
   */
  async registerChatUser(nickname?: string): Promise<ChatUserId> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.chat.registerChatUser(nickname || null);

    return new Promise((resolve, reject) => {
      tx.signAndSend(this.myAddress, ({ status, events }) => {
        if (status.isInBlock) {
          for (const { event } of events) {
            if (this.api!.events.chat.ChatUserCreated.is(event)) {
              const [, chatUserId] = event.data;
              resolve(chatUserId.toNumber() as ChatUserId);
              return;
            }
          }
          reject(new Error('ChatUserCreated event not found'));
        }
      }).catch(reject);
    });
  }

  /**
   * 获取用户资料
   */
  async getUserProfile(chatUserId: ChatUserId): Promise<ChatUserProfile | null> {
    if (!this.api) return null;

    const result = await this.api.query.chat.chatUserProfiles(chatUserId);
    if (result.isNone) return null;

    const data = result.unwrap();
    const accountResult = await this.api.query.chat.chatUserIdToAccount(chatUserId);
    const accountId = accountResult.isSome ? accountResult.unwrap().toString() : '';

    return {
      chatUserId,
      accountId,
      nickname: data.nickname.isSome ? data.nickname.unwrap().toUtf8() : undefined,
      avatarCid: data.avatarCid.isSome ? data.avatarCid.unwrap().toUtf8() : undefined,
      signature: data.signature.isSome ? data.signature.unwrap().toUtf8() : undefined,
      status: this.parseUserStatus(data.status.toString()),
      privacySettings: {
        allowStrangerMessages: data.privacySettings.allowStrangerMessages.valueOf(),
        showOnlineStatus: data.privacySettings.showOnlineStatus.valueOf(),
        showLastActive: data.privacySettings.showLastActive.valueOf(),
      },
      createdAt: data.createdAt.toNumber(),
      lastActive: data.lastActive.toNumber(),
    };
  }

  /**
   * 更新用户资料
   */
  async updateProfile(params: {
    nickname?: string;
    avatarCid?: string;
    signature?: string;
  }): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.chat.updateChatProfile(
      params.nickname || null,
      params.avatarCid || null,
      params.signature || null
    );

    await tx.signAndSend(this.myAddress);
  }

  /**
   * 设置用户状态
   */
  async setUserStatus(status: UserStatus): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const statusCode = this.statusToCode(status);
    const tx = this.api.tx.chat.setUserStatus(statusCode);

    await tx.signAndSend(this.myAddress);
  }

  /**
   * 更新隐私设置
   */
  async updatePrivacySettings(settings: Partial<PrivacySettings>): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.chat.updatePrivacySettings(
      settings.allowStrangerMessages ?? null,
      settings.showOnlineStatus ?? null,
      settings.showLastActive ?? null
    );

    await tx.signAndSend(this.myAddress);
  }

  /**
   * 监听新消息事件
   */
  subscribeMessages(callback: (msg: Message) => void): () => void {
    if (!this.api) return () => {};

    const unsub = this.api.query.system.events((events: any[]) => {
      for (const { event } of events) {
        if (this.api!.events.chat.MessageSent.is(event)) {
          const [msgId, , , receiver] = event.data;

          // 只处理发给自己的消息
          if (receiver.toString() === this.myAddress) {
            this.getMessageDetail(msgId.toNumber()).then((msg) => {
              if (msg) callback(msg);
            });
          }
        }
      }
    });

    this.unsubscribe = unsub as any;
    return () => {
      if (this.unsubscribe) {
        this.unsubscribe();
        this.unsubscribe = null;
      }
    };
  }

  /**
   * 获取或派生共享密钥
   */
  private async getSharedKey(peerAddress: string): Promise<Uint8Array> {
    if (!this.sharedKeys.has(peerAddress)) {
      if (!this.myPrivateKey) {
        throw new Error('Private key not initialized');
      }

      // 获取对方的公钥
      const peerPublicKey = await this.getPeerPublicKey(peerAddress);

      // 使用 x25519 ECDH 派生共享密钥
      const key = await deriveSharedKey(this.myPrivateKey, peerPublicKey);
      this.sharedKeys.set(peerAddress, key);
    }
    return this.sharedKeys.get(peerAddress)!;
  }

  /**
   * 获取对方的公钥
   */
  private async getPeerPublicKey(peerAddress: string): Promise<Uint8Array> {
    if (!this.api) throw new Error('API not initialized');

    // 从链上查询对方的 ChatUserId
    const chatUserIdResult = await this.api.query.chat.accountToChatUserId(peerAddress);
    if (chatUserIdResult.isNone) {
      // 对方未注册，使用默认公钥（实际应用中应该要求对方先注册）
      throw new Error('Peer has not registered chat user');
    }

    const chatUserId = chatUserIdResult.unwrap().toNumber();
    const profile = await this.getUserProfile(chatUserId);

    if (!profile?.encryptionPublicKey) {
      throw new Error('Peer encryption public key not found');
    }

    return hexToPublicKey(profile.encryptionPublicKey);
  }

  private parseUserStatus(status: string): UserStatus {
    const statusMap: Record<string, UserStatus> = {
      Online: UserStatus.Online,
      Offline: UserStatus.Offline,
      Busy: UserStatus.Busy,
      Away: UserStatus.Away,
      Invisible: UserStatus.Invisible,
    };
    return statusMap[status] || UserStatus.Offline;
  }

  private statusToCode(status: UserStatus): number {
    const codeMap: Record<UserStatus, number> = {
      [UserStatus.Online]: 0,
      [UserStatus.Offline]: 1,
      [UserStatus.Busy]: 2,
      [UserStatus.Away]: 3,
      [UserStatus.Invisible]: 4,
    };
    return codeMap[status];
  }

  /**
   * 获取当前用户的 ChatUserId
   * @returns ChatUserId 或 null（如果未注册）
   */
  async getMyChatUserId(): Promise<ChatUserId | null> {
    if (!this.api) throw new Error('API not initialized');

    const result = await this.api.query.chat.accountToChatUserId(this.myAddress);
    if (result.isNone) {
      return null;
    }
    return result.unwrap().toNumber();
  }

  /**
   * 获取当前用户的完整资料
   * @returns ChatUserProfile 或 null（如果未注册）
   */
  async getMyProfile(): Promise<ChatUserProfile | null> {
    const chatUserId = await this.getMyChatUserId();
    if (chatUserId === null) {
      return null;
    }
    return this.getUserProfile(chatUserId);
  }

  /**
   * 清理资源
   */
  destroy(): void {
    if (this.unsubscribe) {
      this.unsubscribe();
    }
    this.sharedKeys.clear();
  }
}

// 单例
let chatServiceInstance: ChatService | null = null;

export function getChatService(): ChatService {
  if (!chatServiceInstance) {
    throw new Error('ChatService not initialized');
  }
  return chatServiceInstance;
}

export function initChatService(myAddress: string): ChatService {
  chatServiceInstance = new ChatService(myAddress);
  return chatServiceInstance;
}
