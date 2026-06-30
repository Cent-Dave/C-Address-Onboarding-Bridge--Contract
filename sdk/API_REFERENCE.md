# API Reference — @stellar/c-address-onboarding-bridge-sdk

This document provides a comprehensive API reference for all exported classes, interfaces, error types, and retry configurations of the C-Address Onboarding Bridge TypeScript SDK.

---

## Table of Contents
1. [Classes](#classes)
   - [OnboardingBridgeSDK](#onboardingbridgesdk)
   - [OffRampIntegration](#offrampintegration)
2. [Interfaces](#interfaces)
   - [BridgeConfig](#bridgeconfig)
   - [FundCOptions](#fundcoptions)
   - [BatchFundCOptions](#batchfundcoptions)
   - [WithdrawFeesOptions](#withdrawfeesoptions)
   - [OffRampConfig](#offrampconfig)
   - [TransactionResult](#transactionresult)
   - [FundCAddressWithSwapOptions](#fundcaddresswithswapoptions)
   - [CrossChainFundOptions](#crosschainfundoptions)
   - [CreateCOptions](#createcoptions)
   - [CreateCAddressResult](#createcaddressresult)
   - [PaginatedResult](#paginatedresult)
   - [PaginationOptions](#paginationoptions)
3. [Error Handling & Validation](#error-handling--validation)
   - [Validation Methods](#validation-methods)
   - [RPC Retry & Transient Errors](#rpc-retry--transient-errors)
   - [Common Error Scenarios & Solutions](#common-error-scenarios--solutions)

---

## Classes

### `OnboardingBridgeSDK`

Main entry point for interacting with the deployed Onboarding Bridge Soroban contract.

#### Constructor

```ts
constructor(config: BridgeConfig)
```
Initializes the SDK instance, instantiates the underlying Soroban RPC server, and wraps it in a transparent retry proxy.

- **Parameters**:
  - `config`: [`BridgeConfig`](#bridgeconfig) — Connection details and retry preferences.

---

#### Methods

##### `fundCAddress`
```ts
async fundCAddress(
  options: FundCOptions,
  sourceKeypair: Keypair
): Promise<TransactionResult>
```
Funds a single C-address from a source account. The source account must have pre-authorized the token transfer to the bridge contract.
- **Parameters**:
  - `options`: [`FundCOptions`](#fundcoptions)
  - `sourceKeypair`: `Keypair` (from `@stellar/stellar-sdk`)
- **Returns**: `Promise<TransactionResult>`

##### `fundCAddressWithSwap`
```ts
async fundCAddressWithSwap(
  options: FundCAddressWithSwapOptions,
  sourceKeypair: any
): Promise<TransactionResult>
```
Funds a C-address by first swapping a source asset to a target asset via DEX pools, deducting the fee, and forwarding the remaining net amount.
- **Parameters**:
  - `options`: [`FundCAddressWithSwapOptions`](#fundcaddresswithswapoptions)
  - `sourceKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `batchFundCAddresses`
```ts
async batchFundCAddresses(
  options: BatchFundCOptions,
  sourceKeypair: Keypair
): Promise<TransactionResult>
```
Funds multiple C-addresses in a single Stellar transaction.
- **Parameters**:
  - `options`: [`BatchFundCOptions`](#batchfundcoptions)
  - `sourceKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `withdrawFees`
```ts
async withdrawFees(
  options: WithdrawFeesOptions,
  feeCollectorKeypair: Keypair
): Promise<TransactionResult>
```
Withdraws accumulated fees from the bridge contract. Accessible only by the designated fee collector address.
- **Parameters**:
  - `options`: [`WithdrawFeesOptions`](#withdrawfeesoptions)
  - `feeCollectorKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `reclaimTokens`
```ts
async reclaimTokens(
  options: ReclaimTokensOptions,
  adminKeypair: Keypair
): Promise<TransactionResult>
```
Reclaims accidentally sent tokens held in the bridge contract. Accessible only by the contract admin.
- **Parameters**:
  - `options`: `ReclaimTokensOptions`
  - `adminKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `getFee`
```ts
async getFee(): Promise<number>
```
Simulates a transaction to retrieve the current protocol fee in basis points (1 bps = 0.01%).
- **Returns**: `Promise<number>`

##### `getFeeCollector`
```ts
async getFeeCollector(): Promise<string>
```
Retrieves the G-address of the designated fee collector.
- **Returns**: `Promise<string>`

##### `getAdmin`
```ts
async getAdmin(): Promise<string>
```
Retrieves the G-address of the contract admin.
- **Returns**: `Promise<string>`

##### `getCAddressBalance`
```ts
async getCAddressBalance(
  cAddress: string,
  asset: string
): Promise<string>
```
Queries the balance of a C-address for a specific whitelisted token.
- **Parameters**:
  - `cAddress`: `string` — Target contract address.
  - `asset`: `string` — Token contract address.
- **Returns**: `Promise<string>` (in the smallest unit of the token)

##### `getFeeBalance`
```ts
async getFeeBalance(asset: string): Promise<string>
```
Queries the accumulated fee balance for a given token contract.
- **Parameters**:
  - `asset`: `string` — Token contract address.
- **Returns**: `Promise<string>`

##### `getAllBalances`
```ts
async getAllBalances(assets: string[]): Promise<Record<string, string>>
```
Queries balances for a set of token contracts in one simulation call.
- **Parameters**:
  - `assets`: `string[]` — Array of token contract addresses.
- **Returns**: `Promise<Record<string, string>>` — Map of `assetAddress -> balance`.

##### `isInitialized`
```ts
async isInitialized(): Promise<boolean>
```
Checks if the contract is initialized.
- **Returns**: `Promise<boolean>`

##### `setFee`
```ts
async setFee(
  newFeeBps: number,
  adminKeypair: Keypair
): Promise<TransactionResult>
```
Updates the protocol fee rate (admin only; capped at 1000 bps or 10%).
- **Parameters**:
  - `newFeeBps`: `number` — New rate in basis points.
  - `adminKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `setFeeCollector`
```ts
async setFeeCollector(
  newFeeCollector: string,
  adminKeypair: Keypair
): Promise<TransactionResult>
```
Rotates the fee collector address (admin only).
- **Parameters**:
  - `newFeeCollector`: `string` — New collector's G-address.
  - `adminKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `setAdmin`
```ts
async setAdmin(
  newAdmin: string,
  adminKeypair: Keypair
): Promise<TransactionResult>
```
Transfers the contract admin role (admin only).
- **Parameters**:
  - `newAdmin`: `string` — New admin's G-address.
  - `adminKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `upgrade`
```ts
async upgrade(
  options: UpgradeOptions,
  adminKeypair: Keypair
): Promise<TransactionResult>
```
Upgrades the contract implementation to a new compiled WASM hash (admin only).
- **Parameters**:
  - `options`: `UpgradeOptions`
  - `adminKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `fundCrosschain`
```ts
async fundCrosschain(
  options: CrossChainFundOptions,
  relayerKeypair: Keypair
): Promise<TransactionResult>
```
Submits a cross-chain funding transaction verified with relayer signatures (relayer only).
- **Parameters**:
  - `options`: [`CrossChainFundOptions`](#crosschainfundoptions)
  - `relayerKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `addRelayer`
```ts
async addRelayer(
  options: RelayerManagementOptions,
  adminKeypair: Keypair
): Promise<TransactionResult>
```
Registers an authorized relayer public key (admin only).
- **Parameters**:
  - `options`: `RelayerManagementOptions`
  - `adminKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `removeRelayer`
```ts
async removeRelayer(
  options: RelayerManagementOptions,
  adminKeypair: Keypair
): Promise<TransactionResult>
```
Removes a registered relayer public key (admin only).
- **Parameters**:
  - `options`: `RelayerManagementOptions`
  - `adminKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `setRelayerThreshold`
```ts
async setRelayerThreshold(
  threshold: number,
  adminKeypair: Keypair
): Promise<TransactionResult>
```
Configures the required threshold of relayer signatures (admin only).
- **Parameters**:
  - `threshold`: `number`
  - `adminKeypair`: `Keypair`
- **Returns**: `Promise<TransactionResult>`

##### `queryRelayerThreshold`
```ts
async queryRelayerThreshold(): Promise<number>
```
Queries the current signature threshold for cross-chain transactions.
- **Returns**: `Promise<number>`

##### `createCAddress`
```ts
async createCAddress(
  options: CreateCOptions
): Promise<CreateCAddressResult>
```
Deploys a new C-address contract deterministically and optionally funds it immediately.
- **Parameters**:
  - `options`: [`CreateCOptions`](#createcoptions)
- **Returns**: `Promise<CreateCAddressResult>`

##### `queryIsRelayer`
```ts
async queryIsRelayer(pubkeyHex: string): Promise<boolean>
```
Checks if a given public key (hex) is a registered relayer.
- **Parameters**:
  - `pubkeyHex`: `string`
- **Returns**: `Promise<boolean>`

##### `getWhitelistedAssets`
```ts
async getWhitelistedAssets(
  cursor?: string,
  limit?: number
): Promise<PaginatedResult<string>>
```
Queries the list of whitelisted tokens with client-side pagination.
- **Parameters**:
  - `cursor`: `string` (optional base64 cursor)
  - `limit`: `number` (optional, default 20)
- **Returns**: `Promise<PaginatedResult<string>>`

##### `getFeeExemptAddresses`
```ts
async getFeeExemptAddresses(
  cursor?: string,
  limit?: number
): Promise<PaginatedResult<string>>
```
Queries fee-exempt addresses with client-side pagination.
- **Parameters**:
  - `cursor`: `string` (optional)
  - `limit`: `number` (optional)
- **Returns**: `Promise<PaginatedResult<string>>`

##### `getBlocklistedAddresses`
```ts
async getBlocklistedAddresses(
  cursor?: string,
  limit?: number
): Promise<PaginatedResult<string>>
```
Queries the blocklist with client-side pagination.
- **Parameters**:
  - `cursor`: `string` (optional)
  - `limit`: `number` (optional)
- **Returns**: `Promise<PaginatedResult<string>>`

##### `getAllowlistedAddresses`
```ts
async getAllowlistedAddresses(
  cursor?: string,
  limit?: number
): Promise<PaginatedResult<string>>
```
Queries the allowlist with client-side pagination.
- **Parameters**:
  - `cursor`: `string` (optional)
  - `limit`: `number` (optional)
- **Returns**: `Promise<PaginatedResult<string>>`

---

### `OffRampIntegration`

A standalone integration class that coordinates off-ramps, on-ramps, and centralized exchange deposit mapping.

#### Constructor

```ts
constructor(config: OffRampConfig)
```
- **Parameters**:
  - `config`: [`OffRampConfig`](#offrampconfig)

---

#### Methods

##### `getOnRampUrl`
```ts
getOnRampUrl(params: OnRampUrlParams): string
```
Generates a direct URL to allow fiat purchases of crypto targetted to a specific C-address.
- **Parameters**:
  - `params`: `OnRampUrlParams`
- **Returns**: `string`

##### `getOffRampUrl`
```ts
getOffRampUrl(params: OffRampUrlParams): string
```
Generates a URL to trigger a crypto off-ramp sell transaction funded from a G-address.
- **Parameters**:
  - `params`: `OffRampUrlParams`
- **Returns**: `string`

##### `getProviderConfig`
```ts
getProviderConfig(provider: OffRampProvider): ProviderConfig
```
Fetches the static configuration and limits for the given off-ramp provider.
- **Parameters**:
  - `provider`: `'moonpay' | 'transak' | 'ramp' | 'banxa'`
- **Returns**: `ProviderConfig`

##### `compareProviders`
```ts
compareProviders(
  amount: string,
  asset: string,
  fiatCurrency?: string
): Partial<Record<OffRampProvider, ProviderComparison>>
```
Compares supported providers for fee levels, net asset yields, and expected settlement times.
- **Parameters**:
  - `amount`: `string` — Total input amount.
  - `asset`: `string` — Target cryptocurrency symbol.
  - `fiatCurrency`: `string` (optional, default `'USD'`)
- **Returns**: `Partial<Record<OffRampProvider, ProviderComparison>>`

##### `generateCEXDepositMemo`
```ts
generateCEXDepositMemo(targetCAddress: string): string
```
Encodes a target C-address into a memo string (`"bridge:<target_c_address>"`) for routing deposits from centralized exchanges.
- **Parameters**:
  - `targetCAddress`: `string` — Destination C-address.
- **Returns**: `string`

##### `decodeCEXDepositMemo`
```ts
decodeCEXDepositMemo(memo: string): string | null
```
Decodes a CEX deposit memo to retrieve the destination C-address, returning `null` if the memo is not formatted correctly.
- **Parameters**:
  - `memo`: `string`
- **Returns**: `string | null`

---

## Interfaces

### `BridgeConfig`
Defines configuration properties to connect the SDK to the network.

| Field | Type | Description |
| :--- | :--- | :--- |
| `contractId` | `string` | Stellar Contract ID (C-address format) of the bridge. |
| `rpcUrl` | `string` | URL of the Soroban RPC server. |
| `networkPassphrase` | `string` | Network identifier (e.g. `Test SDF Network ; September 2015`). |
| `timeout` | `number` | *Optional*. Timeout in seconds for Soroban operations. |
| `retry` | `RpcRetryOptions` | *Optional*. Retries configuration for RPC requests. |

---

### `FundCOptions`
Input parameters for simple single-target funding operations.

| Field | Type | Description |
| :--- | :--- | :--- |
| `source` | `string` | G-address or C-address sending the funds. |
| `target` | `string` | Destination C-address. |
| `asset` | `string` | Token contract address. |
| `amount` | `string` | Amount in the token's smallest unit (e.g., stroops). |

---

### `BatchFundCOptions`
Input parameters for multi-target batch funding operations.

| Field | Type | Description |
| :--- | :--- | :--- |
| `source` | `string` | G-address sending the funds. |
| `targets` | `string[]` | Array of destination C-addresses. |
| `amounts` | `string[]` | Array of amounts corresponding to each target address. |
| `asset` | `string` | Token contract address. |

---

### `WithdrawFeesOptions`
Parameters for fee collection.

| Field | Type | Description |
| :--- | :--- | :--- |
| `asset` | `string` | Token contract address to collect. |
| `amount` | `string` | Amount in smallest unit. |

---

### `OffRampConfig`
Static key definitions for off-ramp integrations.

| Field | Type | Description |
| :--- | :--- | :--- |
| `moonpayApiKey` | `string` | *Optional*. Moonpay API Key. |
| `transakApiKey` | `string` | *Optional*. Transak API Key. |
| `rampApiKey` | `string` | *Optional*. Ramp API Key. |
| `banxaApiKey` | `string` | *Optional*. Banxa API Key. |
| `testMode` | `boolean` | *Optional*. Enables sandbox URLs if `true`. |

---

### `TransactionResult`
The result of an on-chain transaction submission.

| Field | Type | Description |
| :--- | :--- | :--- |
| `hash` | `string` | On-chain transaction hash. |
| `status` | `'success' \| 'pending' \| 'failed'` | Execution status. |
| `error` | `string` | *Optional*. Error description if `status` is `'failed'`. |

---

### `FundCAddressWithSwapOptions`
Input details for swap-and-bridge operations.

| Field | Type | Description |
| :--- | :--- | :--- |
| `source` | `string` | G-address providing the source asset. |
| `target` | `string` | Destination C-address. |
| `sourceAsset` | `string` | Token contract address held by the source. |
| `targetAsset` | `string` | Token contract address the target receives. |
| `sourceAmount` | `string` | Raw input amount of the source asset. |
| `minTargetAmount` | `string` | Minimum acceptable amount of the target asset (slippage limit). |
| `swapRoute` | `string[]` | Ordered array of DEX pool contract addresses. |

---

### `CrossChainFundOptions`
Parameters for bridge relay submissions.

| Field | Type | Description |
| :--- | :--- | :--- |
| `chainId` | `number` | Numeric source chain identifier (e.g. 1 = Ethereum). |
| `txHash` | `string` | Hash of the source transaction. |
| `target` | `string` | Destination C-address. |
| `asset` | `string` | Whitelisted token contract on Soroban. |
| `amount` | `string` | Total gross amount. |
| `sigs` | `RelayerSig[]` | Attestations signed by authorized relayers. |

---

### `CreateCOptions`
Options to deploy a new C-address contract deterministically.

| Field | Type | Description |
| :--- | :--- | :--- |
| `deployerKeypair` | `any` | Deployer's keypair. |
| `salt` | `string` | *Optional*. 32-byte hex salt for address derivation. |
| `initialFunds` | `{ asset: string, amount: string }` | *Optional*. Initial transfer details. |

---

### `CreateCAddressResult`
The return value of C-address deployment.

| Field | Type | Description |
| :--- | :--- | :--- |
| `cAddress` | `string` | Deployed contract C-address. |
| `txHash` | `string` | Transaction hash. |

---

### `PaginatedResult<T>`
Standard return wrapping for paginated queries.

| Field | Type | Description |
| :--- | :--- | :--- |
| `items` | `T[]` | Retrieved items in the page. |
| `cursor` | `string` | *Optional*. Base64 token for next page query. |
| `hasMore` | `boolean` | `true` if more items are available. |

---

### `PaginationOptions`
Parameters to page read queries.

| Field | Type | Description |
| :--- | :--- | :--- |
| `cursor` | `string` | *Optional*. Cursor token to continue listing. |
| `limit` | `number` | *Optional*. Maximum items per page (default: 20). |

---

## Error Handling & Validation

### Validation Methods

The SDK performs frontend validation on Stellar address inputs using the following functions:
- `assertAccountAddress(address: string, field: string)`: Validates that the input is a valid Stellar G-address (ed25519 public key). Throws if validation fails.
- `assertContractAddress(address: string, field: string)`: Validates that the input is a valid Stellar contract C-address. Throws if validation fails.

---

### RPC Retry & Transient Errors

The SDK includes transient error detection and auto-retries via exponential backoff with jitter.

#### Retryable Error Classes
The SDK retries if it catches:
1. **Network Errors**: `ECONNRESET`, `ECONNREFUSED`, `ECONNABORTED`, `ENOTFOUND`, `EAI_AGAIN`, `ETIMEDOUT`, `EPIPE`, `ENETUNREACH`, `EHOSTUNREACH`.
2. **HTTP Transient Statuses**: `429` (Rate Limited), `502` (Bad Gateway), `503` (Service Unavailable), `504` (Gateway Timeout).
3. **Common Message Regex Patterns**: `(timeout|timed out|rate limit|too many requests|network error|socket hang up|fetch failed|service unavailable|temporarily unavailable|connection reset|connection refused)`.

#### Retry Policies
- **Read-Only / Idempotent Calls (`VIEW_RETRY_POLICY`)**: Automatically retries up to 3 times (first backoff at 1s, capped at 30s) using exponential backoff + jitter.
- **State-Changing / Write Calls (`STATE_CHANGING_RETRY_POLICY`)**: Automatically retries **only once** to prevent double-spending, keeping the transaction envelope's sequence identical.

---

### Common Error Scenarios & Solutions

#### 1. `SlippageExceeded` Reverts
- **Scenario**: A swap-and-bridge transaction using `fundCAddressWithSwap` fails on-chain.
- **Cause**: The price of the asset fluctuated on the DEX, causing the output of the swap to drop below `minTargetAmount`.
- **Solution**: Increase the `minTargetAmount` tolerance slightly (higher slippage allowance) or execute the swap in a less volatile window.

#### 2. `Invalid account address` / `Invalid contract address`
- **Scenario**: The SDK throws a validation error before submitting a transaction.
- **Cause**: G-address format was passed where C-address was expected, or vice versa.
- **Solution**: Ensure destination wallets are contract accounts (C-addresses starting with `C...`) and signing accounts are standard Stellar accounts (G-addresses starting with `G...`).

#### 3. `429 Too Many Requests`
- **Scenario**: API or RPC queries are getting rejected.
- **Cause**: The public Soroban RPC node rate limits have been exceeded.
- **Solution**: Use a private RPC node provider or configure custom `retry` settings in [`BridgeConfig`](#bridgeconfig) with a higher `baseDelayMs` to stagger requests.
