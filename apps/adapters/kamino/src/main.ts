import express, { Request, Response, NextFunction } from "express";
import { loadConfig } from "./config";
import { KaminoClient } from "./client";
import { MarketService } from "./market";
import { ReserveService } from "./reserve";
import { ObligationService } from "./obligation";
import { createHealthRouter } from "./routes/health";
import { createMarketsRouter } from "./routes/markets";
import { createObligationsRouter } from "./routes/obligations";
import { AdapterError } from "./errors";
import { KaminoErrorResponse } from "./contracts";

export function createApp() {
  const config = loadConfig();
  const client = new KaminoClient(config);
  const marketService = new MarketService(client, config);
  const reserveService = new ReserveService(client);
  const obligationService = new ObligationService(client, config);

  const app = express();
  app.use(express.json({ limit: "1mb" }));

  // Request logger
  app.use((req: Request, _res: Response, next: NextFunction) => {
    const start = Date.now();
    _res.on("finish", () => {
      const duration = Date.now() - start;
      console.log(
        `[kamino-adapter] ${req.method} ${req.originalUrl} ${_res.statusCode} (${duration}ms)`
      );
    });
    next();
  });

  // Mount routes
  app.use("/health", createHealthRouter(config));
  app.use("/markets", createMarketsRouter(marketService, reserveService, config));
  app.get(
    "/reserves/:market_id",
    async (req: Request, res: Response, next: NextFunction) => {
      try {
        const reserves = await reserveService.getReserves(req.params.market_id);
        res.json(reserves);
      } catch (err) {
        next(err);
      }
    }
  );
  app.use("/obligations", createObligationsRouter(obligationService));

  // Central error handler
  app.use(
    (
      err: any,
      _req: Request,
      res: Response<KaminoErrorResponse>,
      _next: NextFunction
    ) => {
      if (err instanceof AdapterError) {
        res.status(err.statusCode).json({
          error: err.message,
          code: err.code,
          details: err.details,
        });
        return;
      }

      console.error("[kamino-adapter] Unhandled error:", err);
      res.status(500).json({
        error: err.message || "Internal server error",
        code: "INTERNAL_ERROR",
      });
    }
  );

  return { app, config, client, marketService, reserveService, obligationService };
}

if (require.main === module) {
  const { app, config } = createApp();
  app.listen(config.bindPort, config.bindHost, () => {
    console.log(
      `[kamino-adapter] Started on http://${config.bindHost}:${config.bindPort} (network: ${config.solanaNetwork})`
    );
  });
}

