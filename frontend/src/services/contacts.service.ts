/**
 * 通讯录服务
 * 与后端 pallet-contacts 交互
 */

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/api';
import type {
  Contact,
  ContactGroup,
  BlockedUser,
  FriendRequest,
  FriendStatus,
  ContactsStats,
  FRIEND_REQUEST_EXPIRY_BLOCKS,
} from '@/features/contacts/types';

/** Codec 类型接口 */
interface CodecValue {
  toNumber(): number;
  toString(): string;
  toHuman(): unknown;
  toUtf8?(): string;
  isSome?: boolean;
  isNone?: boolean;
  unwrap?(): CodecValue;
}

/** 联系人数据接口 */
interface ContactData {
  alias: { isSome: boolean; unwrap(): CodecValue };
  groups: CodecValue[];
  friendStatus: CodecValue;
  addedAt: CodecValue;
  updatedAt: CodecValue;
}

/** 分组数据接口 */
interface GroupData {
  memberCount: CodecValue;
  createdAt: CodecValue;
}

/** 黑名单数据接口 */
interface BlockedData {
  reason: { isSome: boolean; unwrap(): CodecValue };
  blockedAt: CodecValue;
}

/** 好友申请数据接口 */
interface FriendRequestData {
  message: { isSome: boolean; unwrap(): CodecValue };
  requestedAt: CodecValue;
}

export class ContactsService {
  private api: ApiPromise | null = null;
  private myAddress: string;

  constructor(myAddress: string) {
    this.myAddress = myAddress;
  }

  async init(): Promise<void> {
    this.api = await getApi();
  }

  // ========== 统计信息 ==========

  /**
   * 获取通讯录统计信息
   */
  async getStats(): Promise<ContactsStats> {
    if (!this.api) throw new Error('API not initialized');

    const [contactCount, groupCount, blacklistCount, pendingRequestCount] =
      await Promise.all([
        this.api.query.contacts.contactCount(this.myAddress),
        this.api.query.contacts.groupCount(this.myAddress),
        this.api.query.contacts.blacklistCount(this.myAddress),
        this.api.query.contacts.pendingRequestCount(this.myAddress),
      ]);

    return {
      contactCount: (contactCount as CodecValue).toNumber(),
      groupCount: (groupCount as CodecValue).toNumber(),
      blacklistCount: (blacklistCount as CodecValue).toNumber(),
      pendingRequestCount: (pendingRequestCount as CodecValue).toNumber(),
    };
  }

  // ========== 联系人管理 ==========

  /**
   * 添加联系人
   */
  async addContact(
    contact: string,
    alias?: string,
    groups?: string[]
  ): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.addContact(
      contact,
      alias || null,
      groups || []
    );
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 删除联系人
   */
  async removeContact(contact: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.removeContact(contact);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 更新联系人信息
   */
  async updateContact(
    contact: string,
    alias?: string,
    groups?: string[]
  ): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.updateContact(
      contact,
      alias || null,
      groups || []
    );
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 获取所有联系人
   */
  async getAllContacts(): Promise<Contact[]> {
    if (!this.api) return [];

    const entries = await this.api.query.contacts.contacts.entries(
      this.myAddress
    );
    const contacts: Contact[] = [];

    for (const [key, value] of entries) {
      const contactAddr = key.args[1].toString();
      const data = (value as { unwrap(): ContactData }).unwrap();

      // 解析好友状态
      const friendStatusStr = data.friendStatus.toString();
      let friendStatus: FriendStatus;
      switch (friendStatusStr) {
        case 'Mutual':
          friendStatus = FriendStatus.Mutual;
          break;
        case 'Pending':
          friendStatus = FriendStatus.Pending;
          break;
        default:
          friendStatus = FriendStatus.OneWay;
      }

      contacts.push({
        address: contactAddr,
        alias: data.alias.isSome ? data.alias.unwrap().toUtf8?.() : undefined,
        groups: data.groups.map((g) => g.toUtf8?.() ?? g.toString()),
        friendStatus,
        addedAt: data.addedAt.toNumber(),
        updatedAt: data.updatedAt.toNumber(),
      });
    }

    return contacts;
  }

  /**
   * 获取单个联系人
   */
  async getContact(contactAddress: string): Promise<Contact | null> {
    if (!this.api) return null;

    const result = await this.api.query.contacts.contacts(
      this.myAddress,
      contactAddress
    );
    if ((result as { isNone?: boolean }).isNone) return null;

    const data = (result as { unwrap(): ContactData }).unwrap();
    const friendStatusStr = data.friendStatus.toString();
    let friendStatus: FriendStatus;
    switch (friendStatusStr) {
      case 'Mutual':
        friendStatus = FriendStatus.Mutual;
        break;
      case 'Pending':
        friendStatus = FriendStatus.Pending;
        break;
      default:
        friendStatus = FriendStatus.OneWay;
    }

    return {
      address: contactAddress,
      alias: data.alias.isSome ? data.alias.unwrap().toUtf8?.() : undefined,
      groups: data.groups.map((g) => g.toUtf8?.() ?? g.toString()),
      friendStatus,
      addedAt: data.addedAt.toNumber(),
      updatedAt: data.updatedAt.toNumber(),
    };
  }

  /**
   * 检查是否为双向好友
   */
  async areMutualFriends(contactAddress: string): Promise<boolean> {
    const contact = await this.getContact(contactAddress);
    return contact?.friendStatus === FriendStatus.Mutual;
  }

  /**
   * 获取双向好友列表
   */
  async getMutualFriends(): Promise<Contact[]> {
    const contacts = await this.getAllContacts();
    return contacts.filter((c) => c.friendStatus === FriendStatus.Mutual);
  }

  // ========== 分组管理 ==========

  /**
   * 创建分组
   */
  async createGroup(name: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.createGroup(name);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 删除分组
   */
  async deleteGroup(name: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.deleteGroup(name);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 重命名分组
   */
  async renameGroup(oldName: string, newName: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.renameGroup(oldName, newName);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 获取所有分组
   */
  async getAllGroups(): Promise<ContactGroup[]> {
    if (!this.api) return [];

    const entries = await this.api.query.contacts.groups.entries(
      this.myAddress
    );
    const groups: ContactGroup[] = [];

    for (const [key, value] of entries) {
      const name = key.args[1].toHuman() as string;
      const data = (value as { unwrap(): GroupData }).unwrap();

      groups.push({
        name,
        memberCount: data.memberCount.toNumber(),
        createdAt: data.createdAt.toNumber(),
      });
    }

    return groups;
  }

  /**
   * 获取分组成员
   */
  async getGroupMembers(groupName: string): Promise<string[]> {
    if (!this.api) return [];

    const result = await this.api.query.contacts.groupMembers(
      this.myAddress,
      groupName
    );
    if ((result as { isNone?: boolean }).isNone) return [];

    return (result as { unwrap(): CodecValue[] }).unwrap().map((addr) => addr.toString());
  }

  // ========== 黑名单管理 ==========

  /**
   * 添加到黑名单
   */
  async blockAccount(account: string, reason?: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.blockAccount(account, reason || null);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 从黑名单移除
   */
  async unblockAccount(account: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.unblockAccount(account);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 获取黑名单列表
   */
  async getBlacklist(): Promise<BlockedUser[]> {
    if (!this.api) return [];

    const entries = await this.api.query.contacts.blacklist.entries(
      this.myAddress
    );
    const blockedUsers: BlockedUser[] = [];

    for (const [key, value] of entries) {
      const blockedAddr = key.args[1].toString();
      const data = (value as { unwrap(): BlockedData }).unwrap();

      blockedUsers.push({
        address: blockedAddr,
        reason: data.reason.isSome ? data.reason.unwrap().toUtf8?.() : undefined,
        blockedAt: data.blockedAt.toNumber(),
      });
    }

    return blockedUsers;
  }

  /**
   * 检查是否在黑名单中
   */
  async isBlocked(account: string): Promise<boolean> {
    if (!this.api) return false;

    const result = await this.api.query.contacts.blacklist(
      this.myAddress,
      account
    );
    return (result as { isSome?: boolean }).isSome ?? false;
  }

  // ========== 好友申请 ==========

  /**
   * 发送好友申请
   */
  async sendFriendRequest(target: string, message?: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.sendFriendRequest(target, message || null);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 接受好友申请
   */
  async acceptFriendRequest(requester: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.acceptFriendRequest(requester);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 拒绝好友申请
   */
  async rejectFriendRequest(requester: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.contacts.rejectFriendRequest(requester);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 获取收到的好友申请
   */
  async getReceivedFriendRequests(
    includeExpired = false
  ): Promise<FriendRequest[]> {
    if (!this.api) return [];

    const entries = await this.api.query.contacts.friendRequests.entries(
      this.myAddress
    );
    const requests: FriendRequest[] = [];
    const currentBlock = (
      await this.api.query.system.number()
    ).toNumber();
    const expiryBlocks = 100800; // 约 7 天

    for (const [key, value] of entries) {
      const requesterAddr = key.args[1].toString();
      const data = (value as { unwrap(): FriendRequestData }).unwrap();
      const requestedAt = data.requestedAt.toNumber();
      const expiresAt = requestedAt + expiryBlocks;
      const isExpired = currentBlock > expiresAt;

      if (!includeExpired && isExpired) continue;

      requests.push({
        requester: requesterAddr,
        message: data.message.isSome
          ? data.message.unwrap().toUtf8?.()
          : undefined,
        requestedAt,
        expiresAt,
        isExpired,
      });
    }

    // 按申请时间倒序排列
    return requests.sort((a, b) => b.requestedAt - a.requestedAt);
  }

  /**
   * 获取待处理好友申请数量
   */
  async getPendingRequestCount(): Promise<number> {
    if (!this.api) return 0;

    const count = await this.api.query.contacts.pendingRequestCount(
      this.myAddress
    );
    return (count as CodecValue).toNumber();
  }
}

// 单例
let contactsServiceInstance: ContactsService | null = null;

export function getContactsService(): ContactsService {
  if (!contactsServiceInstance) {
    throw new Error('ContactsService not initialized');
  }
  return contactsServiceInstance;
}

export function initContactsService(myAddress: string): ContactsService {
  contactsServiceInstance = new ContactsService(myAddress);
  return contactsServiceInstance;
}
