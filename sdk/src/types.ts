import type { RpcRetryOptions } from './retry';

export interface BridgeConfig {
  /** Contract ID of the deployed OnboardingBridge Soroban contract */
  contractId: string;
  /** Soroban RPC URL (e.g. https://soroban-testnet.stellar.org) */
  rpcUrl: string;
  /** Network passphrase */
  networkPassphrase: string;
  /** Optional timeout in seconds for Soroban operations */
  timeout?: number;
  /**
   * Automatic retry behaviour for RPC calls (exponential backoff + jitter).
   * Omit to use the defaults (3 retries for reads, 1 for writes). Set
   * `maxRetries: 0` to disable retries entirely.
   */
  retry?: RpcRetryOptions;
}

export interface FundCOptions {
  /** Source account (G-address or C-address) sending the funds */
  source: string;
  /** Target C-address to receive funds */
  target: string;
  /** Asset contract address to use for the transfer */
  asset: string;
  /** Amount in smallest unit (stroops for XLM, or asset's native unit) */
  amount: string;
}

export interface BatchFundCOptions {
  /** Source account sending the funds */
  source: string;
  /** Target C-addresses to receive funds */
  targets: string[];
  /** Corresponding amounts for each target */
  amounts: string[];
  /** Asset contract address */
  asset: string;
}

export interface WithdrawFeesOptions {
  /** Asset contract address to withdraw fees from */
  asset: string;
  /** Amount to withdraw */
  amount: string;
}

export interface UpgradeOptions {
  /** New wasm hash (32-byte hex string) to upgrade the contract to */
  newWasmHash: string;
}

export interface ReclaimTokensOptions {
  /** Asset contract address */
  asset: string;
  /** Amount to reclaim */
  amount: string;
  /** Destination address to receive the reclaimed tokens */
  to: string;
}

export interface OffRampConfig {
  /** Your Moonpay API key */
  moonpayApiKey?: string;
  /** Your Transak API key */
  transakApiKey?: string;
  /** Your Ramp API key */
  rampApiKey?: string;
  /** Your Banxa API key */
  banxaApiKey?: string;
  /** Whether to use sandbox/test environment */
  testMode?: boolean;
}

export type OffRampProvider = 'moonpay' | 'transak' | 'ramp' | 'banxa';

export interface ProviderConfig {
  /** Provider name */
  provider: OffRampProvider;
  /** List of supported crypto assets */
  supportedAssets: string[];
  /** List of supported fiat currencies */
  supportedFiatCurrencies: string[];
  /** List of supported countries (ISO 3166-1 alpha-2 codes) */
  supportedCountries: string[];
  /** Minimum transaction amount in fiat */
  minAmount: string;
  /** Maximum transaction amount in fiat */
  maxAmount: string;
  /** Fee percentage (e.g., "2.5" for 2.5%) */
  feePercentage: string;
  /** Whether provider is available in test mode */
  testModeAvailable: boolean;
}

export interface ProviderComparison {
  /** Total fee amount for the transaction */
  feeAmount: string;
  /** Fee as a percentage */
  feePercentage: string;
  /** Net amount after fees */
  netAmount: string;
  /** Time to receive funds (in hours, approximate) */
  settlementTime: number;
}

export interface OnRampUrlParams {
  /** Provider to use */
  provider: OffRampProvider;
  /** Amount in fiat currency */
  amount: string;
  /** Fiat currency code (e.g., 'USD', 'EUR') */
  fiatCurrency: string;
  /** Crypto asset (e.g., 'XLM', 'USDC') */
  asset: string;
  /** Target C-address to receive funds */
  cAddress: string;
}

export interface OffRampUrlParams {
  /** Provider to use */
  provider: OffRampProvider;
  /** Amount in crypto to sell */
  amount: string;
  /** Crypto asset (e.g., 'XLM', 'USDC') */
  asset: string;
  /** Fiat currency to receive (e.g., 'USD', 'EUR') */
  fiatCurrency: string;
  /** Source G-address to send crypto from */
  gAddress: string;
}

export interface TransactionResult {
  /** Transaction hash */
  hash: string;
  /** Status of the transaction */
  status: 'success' | 'pending' | 'failed';
  /** Error message if failed */
  error?: string;
}

// --- Cross-chain types ---

/** A single relayer attestation: ed25519 pubkey (hex) + signature (hex) over the payload hash */
export interface RelayerSig {
  /** 32-byte Ed25519 public key as hex string */
  pubkey: string;
  /** 64-byte Ed25519 signature as hex string */
  signature: string;
}

/** Options for funding a C-address from a cross-chain event */
export interface CrossChainFundOptions {
  /** Numeric source chain id (1 = Ethereum, 101 = Solana, etc.) */
  chainId: number;
  /** 32-byte source-chain transaction hash as hex string */
  txHash: string;
  /** Destination Soroban C-address */
  target: string;
  /** Whitelisted token contract address on Stellar */
  asset: string;
  /** Gross amount (fee deducted before crediting target) */
  amount: string;
  /** At least `threshold` relayer signatures over the canonical payload hash */
  sigs: RelayerSig[];
}

/** Options for adding/removing a relayer */
export interface RelayerManagementOptions {
  /** 32-byte Ed25519 public key as hex string */
  pubkey: string;
}

/** Options for creating a new C-address (smart contract account) */
export interface CreateCOptions {
  /** Keypair used to deploy the C-address contract */
  deployerKeypair: any;
  /** Optional salt for deterministic address derivation */
  salt?: string;
  /** Optional initial funds to transfer to the new C-address after creation */
  initialFunds?: {
    /** Asset contract address */
    asset: string;
    /** Amount in smallest unit */
    amount: string;
  };
}

/** Result of creating a C-address */
export interface CreateCAddressResult {
  /** The newly created C-address */
  cAddress: string;
  /** Transaction hash of the creation */
  txHash: string;
}

// ---------------------------------------------------------------------------
// Swap-and-bridge (#100)
// ---------------------------------------------------------------------------

/**
 * Options for `fundCAddressWithSwap`.
 *
 * The bridge contract will:
 * 1. Pull `sourceAmount` of `sourceAsset` from `source`.
 * 2. Route it through `swapRoute` (ordered DEX pool addresses).
 * 3. Deduct the protocol fee in `targetAsset`.
 * 4. Credit the net amount to `target`.
 */
export interface FundCAddressWithSwapOptions {
  /** Source account providing `sourceAsset` (G-address). Must sign. */
  source: string;
  /** Destination C-address to receive `targetAsset`. */
  target: string;
  /** Token contract the source holds (e.g. USDC contract address). */
  sourceAsset: string;
  /** Token contract the target should receive (e.g. XLM wrapper). */
  targetAsset: string;
  /** Gross amount of `sourceAsset` in its smallest unit. */
  sourceAmount: string;
  /**
   * Minimum acceptable `targetAsset` amount after the swap.
   * Reverts with `SlippageExceeded` if the DEX returns less.
   */
  minTargetAmount: string;
  /**
   * Ordered list of DEX pool contract addresses.
   * Each pool must implement `swap(min_amount_out: i128, to: Address) -> i128`.
   */
  swapRoute: string[];
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

/**
 * A page of results from a list-returning contract query.
 *
 * Because Soroban contracts return full vectors the pagination is handled
 * client-side: cursor is an opaque base64-encoded offset token so callers
 * don't need to track numeric indices.
 */
export interface PaginatedResult<T> {
  /** Items in this page */
  items: T[];
  /**
   * Opaque token to pass as `cursor` in the next call.
   * `undefined` when there are no more pages.
   */
  cursor?: string;
  /** Convenience flag — true when a next page exists */
  hasMore: boolean;
}

export interface PaginationOptions {
  /** Opaque cursor returned by a previous call */
  cursor?: string;
  /** Maximum items to return per page (default: 20) */
  limit?: number;
}

// ---------------------------------------------------------------------------
// Issue #58: Transaction cost estimation
// ---------------------------------------------------------------------------

/**
 * Estimated cost for a `fundCAddress` transaction.
 * All fee values are in stroops (1 XLM = 10_000_000 stroops).
 */
export interface CostEstimate {
  /**
   * Inclusion fee (base network fee) in stroops.
   * This is the minimum fee required for the transaction to be included.
   */
  fee: string;
  /**
   * Minimum account balance required to submit the transaction, in stroops.
   * Accounts below this balance cannot submit transactions.
   */
  minBalance: string;
  /**
   * Soroban resource fee in stroops (covers CPU, memory, storage, I/O).
   * This is on top of the inclusion fee.
   */
  resourceFee: string;
  /**
   * Wall-clock time the simulation took, in milliseconds.
   * Useful for UI feedback — if this is high, the RPC may be under load.
   */
  executionTimeMs: number;
}
