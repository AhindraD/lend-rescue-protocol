import { KaminoClient } from "./client";
import { KaminoNormalizer } from "./normalizer";
import { KaminoReserveResponse } from "./contracts";

export class ReserveService {
  constructor(private readonly client: KaminoClient) {}

  public async getReserves(marketId: string): Promise<KaminoReserveResponse[]> {
    const { reserves } = await this.client.loadReserves(marketId);
    return reserves.map((r) => KaminoNormalizer.normalizeReserve(r));
  }
}

