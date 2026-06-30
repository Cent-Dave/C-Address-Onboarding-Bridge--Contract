import {
  OffRampConfig,
  OffRampProvider,
  OnRampUrlParams,
  OffRampUrlParams,
  ProviderConfig,
  ProviderComparison,
} from './types';

export class OffRampIntegration {
  private config: OffRampConfig;

  constructor(config: OffRampConfig) {
    this.config = config;
  }

  /**
   * Get unified on-ramp URL builder for multiple providers.
   * Supports funding a C-address with crypto via credit card or bank transfer.
   */
  getOnRampUrl(params: OnRampUrlParams): string {
    switch (params.provider) {
      case 'moonpay':
        return this.getMoonpayOnRampUrl(params);
      case 'transak':
        return this.getTransakOnRampUrl(params);
      case 'ramp':
        return this.getRampOnRampUrl(params);
      case 'banxa':
        return this.getBanxaOnRampUrl(params);
      default:
        throw new Error(`Unsupported provider: ${params.provider}`);
    }
  }

  /**
   * Get unified off-ramp URL builder for multiple providers.
   * Supports selling crypto for fiat, funded from a G-address.
   */
  getOffRampUrl(params: OffRampUrlParams): string {
    switch (params.provider) {
      case 'moonpay':
        return this.getMoonpayOffRampUrl(params);
      case 'transak':
        return this.getTransakOffRampUrl(params);
      case 'ramp':
        return this.getRampOffRampUrl(params);
      case 'banxa':
        return this.getBanxaOffRampUrl(params);
      default:
        throw new Error(`Unsupported provider: ${params.provider}`);
    }
  }

  /**
   * Get configuration and capabilities for a provider.
   * Returns supported assets, countries, limits, and fees.
   */
  getProviderConfig(provider: OffRampProvider): ProviderConfig {
    const configs: Record<OffRampProvider, ProviderConfig> = {
      moonpay: {
        provider: 'moonpay',
        supportedAssets: ['XLM', 'USDC', 'ETH', 'BTC'],
        supportedFiatCurrencies: [
          'USD', 'EUR', 'GBP', 'AUD', 'CAD', 'CHF', 'SGD', 'HKD', 'JPY',
        ],
        supportedCountries: [
          'US', 'GB', 'DE', 'FR', 'IT', 'ES', 'NL', 'BE', 'AT', 'CH',
          'SE', 'NO', 'DK', 'FI', 'PL', 'AU', 'NZ', 'CA', 'SG', 'HK',
          'JP', 'KR', 'BR', 'MX',
        ],
        minAmount: '20',
        maxAmount: '50000',
        feePercentage: '4.5',
        testModeAvailable: true,
      },
      transak: {
        provider: 'transak',
        supportedAssets: ['XLM', 'USDC', 'ETH', 'BTC', 'MATIC', 'SOL'],
        supportedFiatCurrencies: [
          'USD', 'EUR', 'GBP', 'AUD', 'CAD', 'INR', 'MXN', 'BRL',
        ],
        supportedCountries: [
          'US', 'GB', 'DE', 'FR', 'IT', 'ES', 'NL', 'AT', 'CH', 'SE',
          'AU', 'CA', 'IN', 'MX', 'BR', 'SG', 'HK', 'AE', 'SA',
        ],
        minAmount: '25',
        maxAmount: '100000',
        feePercentage: '3.9',
        testModeAvailable: true,
      },
      ramp: {
        provider: 'ramp',
        supportedAssets: ['XLM', 'USDC', 'ETH', 'BTC', 'DAI', 'USDT'],
        supportedFiatCurrencies: [
          'USD', 'EUR', 'GBP', 'SEK', 'NOK', 'DKK', 'CHF', 'PLN', 'CZK',
        ],
        supportedCountries: [
          'US', 'GB', 'DE', 'FR', 'IT', 'ES', 'NL', 'BE', 'AT', 'CH',
          'SE', 'NO', 'DK', 'FI', 'PL', 'CZ', 'SK', 'HU', 'RO',
        ],
        minAmount: '15',
        maxAmount: '20000',
        feePercentage: '2.9',
        testModeAvailable: false,
      },
      banxa: {
        provider: 'banxa',
        supportedAssets: ['XLM', 'USDC', 'ETH', 'BTC', 'ADA', 'DOGE'],
        supportedFiatCurrencies: [
          'USD', 'EUR', 'GBP', 'AUD', 'CAD', 'NZD', 'SGD', 'ZAR',
        ],
        supportedCountries: [
          'US', 'GB', 'DE', 'FR', 'AU', 'NZ', 'CA', 'SG', 'ZA', 'NL',
          'BE', 'AT', 'IT', 'ES', 'IE',
        ],
        minAmount: '10',
        maxAmount: '75000',
        feePercentage: '3.5',
        testModeAvailable: false,
      },
    };

    return configs[provider];
  }

  /**
   * Compare providers for a given transaction.
   * Returns fee amounts and settlement times across providers.
   * Only includes providers that support the specified asset and fiat currency.
   */
  compareProviders(
    amount: string,
    asset: string,
    fiatCurrency: string = 'USD',
  ): Partial<Record<OffRampProvider, ProviderComparison>> {
    const amountNum = parseFloat(amount);
    const result: Partial<Record<OffRampProvider, ProviderComparison>> = {};

    const providers: OffRampProvider[] = ['moonpay', 'transak', 'ramp', 'banxa'];

    for (const provider of providers) {
      const config = this.getProviderConfig(provider);

      // Check if provider supports this asset
      if (!config.supportedAssets.includes(asset)) {
        continue;
      }

      // Check if provider supports this fiat currency
      if (!config.supportedFiatCurrencies.includes(fiatCurrency)) {
        continue;
      }

      // Calculate fees
      const feePercentage = parseFloat(config.feePercentage);
      const feeAmount = (amountNum * feePercentage) / 100;
      const netAmount = amountNum - feeAmount;

      // Settlement times vary by provider and method
      const settlementTimes: Record<OffRampProvider, number> = {
        moonpay: 2,
        transak: 3,
        ramp: 1,
        banxa: 4,
      };

      result[provider] = {
        feeAmount: feeAmount.toFixed(2),
        feePercentage: config.feePercentage,
        netAmount: netAmount.toFixed(2),
        settlementTime: settlementTimes[provider],
      };
    }

    return result;
  }

  private getMoonpayOnRampUrl(params: OnRampUrlParams): string {
    const baseUrl = this.config.testMode
      ? 'https://buy-staging.moonpay.com'
      : 'https://buy.moonpay.com';

    const url = new URL(baseUrl);
    url.searchParams.set('apiKey', this.config.moonpayApiKey || '');
    url.searchParams.set('currencyCode', params.asset);
    url.searchParams.set('baseCurrencyAmount', params.amount);
    url.searchParams.set('baseCurrencyCode', params.fiatCurrency);
    url.searchParams.set('walletAddress', params.cAddress);
    url.searchParams.set('showWalletAddressForm', 'false');

    return url.toString();
  }

  private getMoonpayOffRampUrl(params: OffRampUrlParams): string {
    const baseUrl = this.config.testMode
      ? 'https://sell-staging.moonpay.com'
      : 'https://sell.moonpay.com';

    const url = new URL(baseUrl);
    url.searchParams.set('apiKey', this.config.moonpayApiKey || '');
    url.searchParams.set('cryptoCurrencyCode', params.asset);
    url.searchParams.set('baseCurrencyCode', params.fiatCurrency);
    url.searchParams.set('walletAddress', params.gAddress);
    url.searchParams.set('refundWalletAddress', params.gAddress);

    return url.toString();
  }

  /**
   * @deprecated Use getOnRampUrl() instead
   * Generate a Moonpay purchase URL to fund a C-address via credit card.
   */
  getMoonpayUrl(params: {
    targetCAddress: string;
    amount: string;
    currency: string;
    assetCode?: string;
  }): string {
    return this.getMoonpayOnRampUrl({
      provider: 'moonpay',
      amount: params.amount,
      asset: params.assetCode || 'XLM',
      fiatCurrency: params.currency,
      cAddress: params.targetCAddress,
    });
  }

  private getTransakOnRampUrl(params: OnRampUrlParams): string {
    const baseUrl = this.config.testMode
      ? 'https://global-staging.transak.com'
      : 'https://global.transak.com';

    const url = new URL(baseUrl);
    url.searchParams.set('apiKey', this.config.transakApiKey || '');
    url.searchParams.set('defaultCryptoCurrency', params.asset);
    url.searchParams.set('walletAddress', params.cAddress);
    url.searchParams.set('defaultFiatAmount', params.amount);
    url.searchParams.set('fiatCurrency', params.fiatCurrency);
    url.searchParams.set('network', 'stellar');

    return url.toString();
  }

  private getTransakOffRampUrl(params: OffRampUrlParams): string {
    const baseUrl = this.config.testMode
      ? 'https://global-staging.transak.com'
      : 'https://global.transak.com';

    const url = new URL(baseUrl);
    url.searchParams.set('apiKey', this.config.transakApiKey || '');
    url.searchParams.set('defaultCryptoCurrency', params.asset);
    url.searchParams.set('walletAddress', params.gAddress);
    url.searchParams.set('fiatCurrency', params.fiatCurrency);
    url.searchParams.set('network', 'stellar');
    url.searchParams.set('isSellMode', 'true');

    return url.toString();
  }

  /**
   * @deprecated Use getOnRampUrl() instead
   * Generate a Transak purchase URL to fund a C-address via credit card.
   */
  getTransakUrl(params: {
    targetCAddress: string;
    amount: string;
    currency: string;
    fiatCurrency?: string;
  }): string {
    return this.getTransakOnRampUrl({
      provider: 'transak',
      amount: params.amount,
      asset: params.currency || 'XLM',
      fiatCurrency: params.fiatCurrency || 'USD',
      cAddress: params.targetCAddress,
    });
  }

  private getRampOnRampUrl(params: OnRampUrlParams): string {
    const baseUrl = 'https://buy.ramp.network';

    const url = new URL(baseUrl);
    url.searchParams.set('userAddress', params.cAddress);
    url.searchParams.set('assetCode', `${params.asset}_stellar`);
    url.searchParams.set('fiatCurrency', params.fiatCurrency);
    url.searchParams.set('fiatAmount', params.amount);
    if (this.config.rampApiKey) {
      url.searchParams.set('hostApiKey', this.config.rampApiKey);
    }

    return url.toString();
  }

  private getRampOffRampUrl(params: OffRampUrlParams): string {
    const baseUrl = 'https://sell.ramp.network';

    const url = new URL(baseUrl);
    url.searchParams.set('userAddress', params.gAddress);
    url.searchParams.set('assetCode', `${params.asset}_stellar`);
    url.searchParams.set('fiatCurrency', params.fiatCurrency);
    if (this.config.rampApiKey) {
      url.searchParams.set('hostApiKey', this.config.rampApiKey);
    }

    return url.toString();
  }

  private getBanxaOnRampUrl(params: OnRampUrlParams): string {
    const baseUrl = 'https://app.banxa.com';

    const url = new URL(baseUrl);
    url.searchParams.set('walletAddress', params.cAddress);
    url.searchParams.set('blockchain', 'stellar');
    url.searchParams.set('cryptoCurrency', params.asset);
    url.searchParams.set('fiatType', params.fiatCurrency);
    url.searchParams.set('fiatAmount', params.amount);
    if (this.config.banxaApiKey) {
      url.searchParams.set('apiKey', this.config.banxaApiKey);
    }

    return url.toString();
  }

  private getBanxaOffRampUrl(params: OffRampUrlParams): string {
    const baseUrl = 'https://app.banxa.com';

    const url = new URL(baseUrl);
    url.searchParams.set('walletAddress', params.gAddress);
    url.searchParams.set('blockchain', 'stellar');
    url.searchParams.set('cryptoCurrency', params.asset);
    url.searchParams.set('fiatType', params.fiatCurrency);
    url.searchParams.set('isSellMode', 'true');
    if (this.config.banxaApiKey) {
      url.searchParams.set('apiKey', this.config.banxaApiKey);
    }

    return url.toString();
  }

  /**
   * Generate a CEX (Centralized Exchange) deposit memo that encodes
   * the target C-address so the bridge contract can route the funds.
   *
   * The memo format is: "bridge:<target_c_address>"
   */
  generateCEXDepositMemo(targetCAddress: string): string {
    return `bridge:${targetCAddress}`;
  }

  /**
   * Decode a CEX deposit memo to extract the target C-address.
   */
  decodeCEXDepositMemo(memo: string): string | null {
    if (!memo.startsWith('bridge:')) {
      return null;
    }
    return memo.slice('bridge:'.length);
  }
}
