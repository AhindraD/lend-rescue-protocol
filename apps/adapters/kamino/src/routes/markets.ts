import { Router, Request, Response, NextFunction } from "express";
import { MarketService } from "../market";
import { ReserveService } from "../reserve";
import { AdapterConfig } from "../config";

export function createMarketsRouter(
  marketService: MarketService,
  reserveService: ReserveService,
  config: AdapterConfig
): Router {
  const router = Router();

  router.get("/", async (_req: Request, res: Response, next: NextFunction) => {
    try {
      const market = await marketService.getMarket(config.defaultMarket);
      res.json(market);
    } catch (err) {
      next(err);
    }
  });

  router.get(
    "/:market_id",
    async (req: Request, res: Response, next: NextFunction) => {
      try {
        const market = await marketService.getMarket(req.params.market_id);
        res.json(market);
      } catch (err) {
        next(err);
      }
    }
  );

  router.get(
    "/:market_id/reserves",
    async (req: Request, res: Response, next: NextFunction) => {
      try {
        const reserves = await reserveService.getReserves(req.params.market_id);
        res.json(reserves);
      } catch (err) {
        next(err);
      }
    }
  );

  return router;
}

