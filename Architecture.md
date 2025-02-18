# SolRefer Protocol Architecture

## Core Functions Specification

### 1. Referral Program Management Functions
```rust
// Create a new referral program with specified parameters and fixed reward amount
create_referral_program(program_params: ProgramParams, fixed_reward_amount: u64) -> ProgramId

// Set eligibility criteria for program participation
set_eligibility_criteria(criteria_params: CriteriaParams) -> Result<(), ProgramError>

// Update existing program settings
update_program_settings(program_id: ProgramId, new_settings: ProgramSettings) -> Result<(), ProgramError>
```

### 2. Referral Link Management Functions
```rust
// Generate a unique referral code for a user
generate_referral_code(program_id: ProgramId, user: Pubkey) -> ReferralCode

// Create referral relationship account
create_referral_relationship(referrer: Pubkey, referee: Pubkey) -> ReferralAccount

// Validate referral code
validate_referral_code(code: ReferralCode) -> Result<bool, ProgramError>
```

### 3. Referral Tracking Functions
```rust
// Record successful conversion for a referral
record_conversion(referral_code: ReferralCode) -> Result<(), ProgramError>

// Update status of a referral
update_referral_status(referral_id: ReferralId, status: ReferralStatus) -> Result<(), ProgramError>

// Get statistics for a specific referral
get_referral_stats(referral_id: ReferralId) -> ReferralStats
```

### 4. Reward Distribution Functions
```rust
// Calculate rewards based on successful referral count
calculate_fixed_rewards(referral_count: u64) -> u64

// Distribute rewards to user wallet
distribute_rewards(user_wallet: Pubkey, amount: u64) -> Result<(), ProgramError>

// Check if lock period has elapsed for rewards
check_lock_period(referral_id: ReferralId) -> bool

// Process early redemption with fee
process_early_redemption(referral_id: ReferralId) -> Result<u64, ProgramError>

// Update program reward balance
update_reward_balance(program_id: ProgramId) -> Result<(), ProgramError>
```

### 5. Token Management Functions
```rust
// Deposit tokens for rewards
deposit_reward_tokens(program_id: ProgramId, amount: u64) -> Result<(), ProgramError>

// Manage protocol fee vault
manage_fee_vault(action: VaultAction, amount: u64) -> Result<(), ProgramError>
```

### 6. User Interaction Functions
```rust
// Join referral program using referral code
join_referral_program(program_id: ProgramId, referral_code: ReferralCode) -> Result<(), ProgramError>

// Claim earned rewards
claim_rewards(referral_id: ReferralId) -> Result<u64, ProgramError>

// Get user performance metrics
get_performance_metrics(user_id: Pubkey) -> PerformanceMetrics

// Get dashboard data for program
get_dashboard_data(program_id: ProgramId) -> DashboardData
```

### 7. Account Management Functions
```rust
// Create new program account
create_program_account(owner: Pubkey) -> ProgramAccount

// Create referral tracking account
create_referral_account(program_id: ProgramId) -> ReferralAccount

// Create reward distribution account
create_reward_account(program_id: ProgramId) -> RewardAccount

// Update account state
update_account_state(account_id: Pubkey, new_state: AccountState) -> Result<(), ProgramError>
```
## Referrer Tracking

### Key Components
- **Participant Account**: Stores referrer information for each user
  - `referrer: Option<Pubkey>`: Stores the referrer's public key
  - `total_referrals: u64`: Tracks number of successful referrals
  - `total_rewards: u64`: Tracks total rewards earned

### Tracking Flow
1. User joins through referral link
2. System verifies referrer exists and is valid
3. Participant account created with referrer field set
4. Referrer's stats updated:
   - `total_referrals` incremented
   - Rewards calculated based on program settings

### Validation Rules
- Referrer must be an existing participant in the same program
- Referrer cannot be self
- Referral program must be active

## Program Architecture

### State Accounts
1. **ReferralProgram**: Stores program parameters and settings
2. **ReferralCode**: PDA storing referral code data and relationships
3. **ReferralAccount**: Tracks referral relationships and conversions
4. **RewardAccount**: Manages reward distribution and claims

### Key Features
- Simple referral code generation and validation
- On-chain tracking of referral relationships
- Flexible reward distribution system
- Support for both SOL and custom tokens
- Configurable lock periods and early redemption fees

### Security Considerations
1. **Referral Code Validation**
   - Ensure uniqueness of referral codes
   - Prevent code spoofing and replay attacks
   - Validate referrer eligibility

2. **Reward Distribution**
   - Secure token vaults
   - Atomic reward distribution
   - Prevention of double claims

3. **Program Access Control**
   - Authority validation
   - Program state mutations
   - Account ownership checks

### Integration Points

1. **Program Creation**
   - Program initialization
   - Token vault setup (if using SPL tokens)
   - Eligibility criteria configuration

2. **User Onboarding**
   - Referral code generation
   - Account creation
   - Program participation

3. **Reward Distribution**
   - Token transfers
   - Reward calculations
   - Lock period management

4. **Data Access**
   - Statistics and metrics
   - User dashboard
   - Program analytics

## Future Features

### 1. Token-Gated Referral Programs
   - Require users to hold a minimum amount of specific tokens to participate
   - Support for both SPL tokens and NFTs as eligibility criteria
   - Configurable token thresholds and holding periods
   - Dynamic eligibility updates based on token balance changes

### 2. NFT-Based Referral System
The protocol can be enhanced with NFT functionality to provide additional features and benefits:

1. **NFT Referral Links**
   - Mint unique NFTs representing referral rights
   - Trade or transfer referral rights through NFT marketplaces
   - Collect different tiers of referral NFTs

2. **Benefits of NFT Implementation**
   - **Ownership & Transferability**
     * Trade referral rights as digital assets
     * Secondary market for successful referral programs
     * Transfer rights between wallets
   
   - **Programmable Rewards**
     * Embed reward tiers in NFT metadata
     * Smart contract-based reward distribution
     * Automatic reward calculations
   
   - **Verifiable On-chain**
     * Transparent referral relationships
     * Immutable record of referral history
     * Proof of referral ownership
   
   - **Gamification & Collection**
     * Different rarity levels for referrers
     * Special edition referral NFTs
     * Collectible referral achievements
   
   - **NFT Ecosystem Integration**
     * List on NFT marketplaces
     * Integration with NFT platforms
     * Cross-platform referral rights

3. **Technical Components**
```rust
// NFT Minting Functions
mint_referral_nft(program_id: ProgramId) -> NftId
generate_referral_link(nft_id: NftId) -> ReferralLink
store_nft_metadata(metadata: Metadata) -> MetadataId

// NFT Management Functions
stake_referral_nft(nft_id: NftId, program_id: ProgramId) -> Result<(), ProgramError>
update_nft_metadata(nft_id: NftId, new_metadata: Metadata) -> Result<(), ProgramError>
verify_nft_ownership(nft_id: NftId, owner: Pubkey) -> bool
```

4. **Additional State Accounts**
   - **NFTMint**: Manages NFT minting and metadata
   - **NFTStaking**: Handles NFT staking for referral programs
   - **NFTMetadata**: Stores referral-specific NFT attributes

5. **Enhanced Security Features**
   - NFT ownership verification
   - Metadata tampering prevention
   - Stake period validation
   - Transfer restrictions

6. **Integration Requirements**
   - NFT marketplace compatibility
   - Metadata standard compliance
   - Cross-program invocation handling
   - Token metadata program integration

This NFT-based implementation would be suitable for:
- High-value referral programs
- Long-term referral relationships
- Programs requiring transferable referral rights
- Gamified referral systems
- Community-driven referral networks
