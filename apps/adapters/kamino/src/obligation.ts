import { KaminoObligation } from "@kamino-finance/klend-sdk";
import { KaminoClient } from "./client";
import { KaminoNormalizer } from "./normalizer";
import { KaminoObligationResponse } from "./contracts";
import { AdapterConfig } from "./config";
import { NotFoundError } from "./errors";

export class ObligationService {
  constructor(
    private readonly client: KaminoClient,
    private readonly config: AdapterConfig
  ) {}

  public async getUserObligations(
    marketId: string,
    owner: string
  ): Promise<KaminoObligationResponse[]> {
    const { obligations, slot } = await this.client.getUserObligations(
      marketId,
      owner
    );

    return obligations.map((obl) =>
      KaminoNormalizer.normalizeObligation(
        obl,
        this.config.solanaNetwork,
        slot
      )
    );
  }

  public async getObligation(
    marketId: string,
    owner: string,
    obligationId: string
  ): Promise<KaminoObligationResponse> {
    const { obligation, slot } = await this.client.getObligation(
      marketId,
      owner,
      obligationId
    );

    return KaminoNormalizer.normalizeObligation(
      obligation,
      this.config.solanaNetwork,
      slot
    );
  }

  public async getObligationById(
    marketId: string,
    obligationId: string
  ): Promise<KaminoObligationResponse> {
    const { market, slot } = await this.client.loadMarket(marketId);
    const obligationAddr = this.client.parseAddress(obligationId, "obligationId");

    const obligation = await KaminoObligation.load(market, obligationAddr);
    if (!obligation) {
      throw new NotFoundError(
        `Obligation '${obligationId}' not found in market '${marketId}'`
      );
    }

    return KaminoNormalizer.normalizeObligation(
      obligation,
      this.config.solanaNetwork,
      slot
    );
  }
}
