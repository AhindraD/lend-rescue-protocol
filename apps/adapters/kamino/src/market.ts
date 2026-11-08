import { KaminoClient } from "./client";
import { KaminoNormalizer } from "./normalizer";
import { KaminoMarketResponse } from "./contracts";
import { AdapterConfig } from "./config";

export class MarketService {
  constructor(
    private readonly client: KaminoClient,
    private readonly config: AdapterConfig
  ) {}

  public async getMarket(marketId: string): Promise<KaminoMarketResponse> {
    const { market, slot } = await this.client.loadMarket(marketId);
    return KaminoNormalizer.normalizeMarket(
      market,
      this.config.solanaNetwork,
      slot
    );
  }
}
