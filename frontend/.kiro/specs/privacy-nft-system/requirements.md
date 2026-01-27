# Requirements Document: Privacy Protection NFT System

## Introduction

This document specifies the requirements for a privacy-protecting NFT system within a blockchain-based matchmaking application. The system enables users to tokenize their profile information as NFTs with granular access control, allowing them to control who can view different tiers of personal information, monetize profile access, verify their identity through badge NFTs, and maintain an encrypted privacy vault with full audit capabilities.

## Glossary

- **Profile_NFT**: A non-fungible token representing a user's profile with embedded access control rules
- **Access_Token_NFT**: A transferable or non-transferable NFT granting permission to view specific profile information tiers
- **Contact_Access_NFT**: An NFT granting permission to initiate direct messaging with a profile owner
- **Verification_Badge_NFT**: A non-transferable NFT proving identity, education, income, or photo verification
- **Privacy_Vault**: An encrypted storage system on IPFS containing sensitive user information
- **Information_Tier**: A classification level for profile data (Basic, Detailed, Contact)
- **Access_Level**: The permission scope granted by an Access_Token_NFT
- **Matchmaking_Service**: The existing service managing user profiles and matching logic
- **IPFS_Service**: The existing service handling decentralized file storage
- **Blockchain_Substrate**: The Substrate-based blockchain infrastructure
- **Revenue_Share**: The automatic distribution of access fees between user and platform
- **Audit_Trail**: An immutable record of profile access events
- **Decryption_Key**: A cryptographic key required to access encrypted vault data

## Requirements

### Requirement 1: Profile NFT Creation and Management

**User Story:** As a user, I want my profile to be represented as an NFT with access control, so that I can control who views my information.

#### Acceptance Criteria

1. WHEN a user creates a profile, THE Matchmaking_Service SHALL mint a Profile_NFT containing the user's wallet address and access control metadata
2. WHEN a Profile_NFT is minted, THE System SHALL assign the NFT to the user's wallet address as the owner
3. THE Profile_NFT SHALL contain references to three Information_Tiers: Basic, Detailed, and Contact
4. WHEN a user updates their profile, THE System SHALL update the Profile_NFT metadata without changing the token ID
5. THE Profile_NFT SHALL be non-transferable and bound to the user's wallet address

### Requirement 2: Information Tier Access Control

**User Story:** As a user, I want different levels of my information to require different permissions, so that I can share basic info freely while protecting sensitive details.

#### Acceptance Criteria

1. THE System SHALL classify profile information into three Information_Tiers: Basic (nickname, age range, location city), Detailed (exact age, full location, photos), and Contact (wallet address for messaging)
2. WHEN any user views a profile, THE System SHALL display Basic tier information without requiring an Access_Token_NFT
3. WHEN a user attempts to view Detailed tier information, THE System SHALL verify the viewer possesses a valid Access_Token_NFT for that profile
4. WHEN a user attempts to view Contact tier information, THE System SHALL verify the viewer possesses a valid Contact_Access_NFT for that profile
5. IF a viewer lacks the required Access_Token_NFT, THEN THE System SHALL display a prompt to purchase or request access

### Requirement 3: Access Token NFT Minting and Distribution

**User Story:** As a profile owner, I want to create access tokens for my profile, so that I can grant viewing permissions to specific users or sell access.

#### Acceptance Criteria

1. WHEN a profile owner requests to mint an Access_Token_NFT, THE System SHALL create a token specifying the target Profile_NFT, Access_Level (Detailed or Contact), and expiration timestamp
2. THE System SHALL support four expiration durations: 24 hours, 7 days, 30 days, and permanent
3. WHEN an Access_Token_NFT is minted, THE System SHALL allow the profile owner to transfer it to a specific wallet address or list it for sale
4. THE Access_Token_NFT SHALL contain metadata linking it to the specific Profile_NFT it grants access to
5. WHEN an Access_Token_NFT expires, THE System SHALL automatically revoke the access permissions

### Requirement 4: Access Token Validation and Revocation

**User Story:** As a profile owner, I want to revoke access tokens if I change my mind, so that I can maintain control over my privacy.

#### Acceptance Criteria

1. WHEN a user attempts to access Detailed or Contact tier information, THE System SHALL verify the user possesses a non-expired Access_Token_NFT for that Profile_NFT
2. WHEN a profile owner revokes an Access_Token_NFT, THE System SHALL mark the token as invalid in the blockchain state
3. WHEN an Access_Token_NFT is revoked, THE System SHALL prevent any further access using that token regardless of expiration date
4. THE System SHALL maintain a list of revoked token IDs for each Profile_NFT
5. WHEN validating an Access_Token_NFT, THE System SHALL check both expiration timestamp and revocation status

### Requirement 5: Privacy Marketplace for Access Sales

**User Story:** As a user, I want to sell access to my profile information, so that I can monetize my profile while controlling who views it.

#### Acceptance Criteria

1. WHEN a profile owner lists an Access_Token_NFT for sale, THE System SHALL create a marketplace listing with the specified price and access duration
2. WHEN a buyer purchases an Access_Token_NFT, THE System SHALL transfer the token to the buyer's wallet and transfer payment to the seller
3. THE System SHALL deduct a platform fee from each sale and distribute Revenue_Share according to the configured percentage
4. WHEN a sale completes, THE System SHALL emit a blockchain event recording the transaction details
5. THE System SHALL allow profile owners to set different prices for Detailed tier access and Contact tier access

### Requirement 6: Verification Badge NFT Issuance

**User Story:** As a user, I want to verify my identity and credentials, so that I can increase trust and profile visibility.

#### Acceptance Criteria

1. THE System SHALL support four types of Verification_Badge_NFTs: Identity (real name verified), Education, Income, and Photo (face match)
2. WHEN a user submits verification documents, THE System SHALL validate the documents and mint the appropriate Verification_Badge_NFT upon approval
3. THE Verification_Badge_NFT SHALL be non-transferable and bound to the user's wallet address
4. WHEN a Verification_Badge_NFT is minted, THE System SHALL update the user's Profile_NFT metadata to reference the badge
5. WHEN displaying a profile, THE System SHALL show all Verification_Badge_NFTs associated with that profile

### Requirement 7: Privacy Vault Encryption and Storage

**User Story:** As a user, I want my sensitive information stored securely and encrypted, so that only authorized users can access it.

#### Acceptance Criteria

1. WHEN a user saves Detailed or Contact tier information, THE System SHALL encrypt the data using the user's Decryption_Key
2. THE System SHALL store encrypted data on IPFS using the IPFS_Service
3. THE System SHALL store the IPFS content hash in the Profile_NFT metadata
4. WHEN a user with a valid Access_Token_NFT requests information, THE System SHALL retrieve the encrypted data from IPFS and decrypt it using the appropriate key
5. THE System SHALL never store unencrypted sensitive information on the blockchain

### Requirement 8: Access Audit Trail

**User Story:** As a user, I want to know who accessed my profile and when, so that I can monitor my privacy.

#### Acceptance Criteria

1. WHEN a user accesses Detailed or Contact tier information, THE System SHALL record an audit event containing the viewer's wallet address, accessed Information_Tier, and timestamp
2. THE System SHALL store audit events in the blockchain state associated with the Profile_NFT
3. WHEN a profile owner requests their Audit_Trail, THE System SHALL return all access events sorted by timestamp
4. THE Audit_Trail SHALL be immutable and verifiable on the blockchain
5. THE System SHALL allow profile owners to view statistics including total views, unique viewers, and access patterns

### Requirement 9: Integration with Existing Privacy Modes

**User Story:** As a user, I want the NFT access system to work with existing privacy modes, so that I have layered privacy control.

#### Acceptance Criteria

1. WHEN a profile is set to Public mode, THE System SHALL allow any user to view Basic tier information but still require Access_Token_NFTs for Detailed and Contact tiers
2. WHEN a profile is set to MembersOnly mode, THE System SHALL require membership verification before allowing any profile access
3. WHEN a profile is set to MatchedOnly mode, THE System SHALL only allow matched users to view Basic tier information and require Access_Token_NFTs for higher tiers
4. THE System SHALL enforce privacy mode restrictions before checking Access_Token_NFT permissions
5. WHEN a user changes privacy mode, THE System SHALL update the Profile_NFT metadata to reflect the new mode

### Requirement 10: Access Request and Notification System

**User Story:** As a user, I want to request access to profiles and be notified when someone requests access to mine, so that I can manage access permissions.

#### Acceptance Criteria

1. WHEN a user without an Access_Token_NFT attempts to view restricted information, THE System SHALL provide an option to request access from the profile owner
2. WHEN an access request is submitted, THE System SHALL send a notification to the profile owner containing the requester's wallet address and requested Access_Level
3. WHEN a profile owner approves an access request, THE System SHALL mint an Access_Token_NFT and transfer it to the requester's wallet
4. WHEN a profile owner denies an access request, THE System SHALL notify the requester of the denial
5. THE System SHALL allow profile owners to set automatic approval rules based on criteria such as Verification_Badge_NFTs or membership status

### Requirement 11: Revenue Analytics and Withdrawal

**User Story:** As a user, I want to track earnings from profile access sales and withdraw my funds, so that I can monetize my profile effectively.

#### Acceptance Criteria

1. WHEN a profile owner sells an Access_Token_NFT, THE System SHALL record the sale amount and Revenue_Share distribution
2. THE System SHALL maintain a balance of earned tokens for each user
3. WHEN a user requests revenue analytics, THE System SHALL display total earnings, number of sales, average sale price, and earnings by Information_Tier
4. WHEN a user requests to withdraw earnings, THE System SHALL transfer the available balance to the user's wallet address
5. THE System SHALL deduct any applicable withdrawal fees before transferring funds

### Requirement 12: Batch Access Token Management

**User Story:** As a profile owner, I want to manage multiple access tokens efficiently, so that I can control access at scale.

#### Acceptance Criteria

1. WHEN a profile owner views their access tokens, THE System SHALL display all active, expired, and revoked tokens with their associated wallet addresses and expiration dates
2. THE System SHALL allow profile owners to revoke multiple Access_Token_NFTs in a single transaction
3. WHEN a profile owner creates a batch of Access_Token_NFTs, THE System SHALL mint multiple tokens with the same parameters in a single transaction
4. THE System SHALL provide filtering and sorting options for access token lists by status, expiration date, and Access_Level
5. WHEN a profile owner exports their access token data, THE System SHALL generate a report containing all token details and access statistics
