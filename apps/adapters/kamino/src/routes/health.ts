import { Router, Request, Response } from "express";
import { AdapterConfig } from "../config";

export function createHealthRouter(config: AdapterConfig): Router {
  const router = Router();

  router.get("/", (_req: Request, res: Response) => {
    res.json({
      status: "ok",
      version: "0.1.0",
      sdkVersion: "13.0.6",
      network: config.solanaNetwork,
      readOnly: true,
      timestamp: new Date().toISOString(),
    });
  });

  return router;
}
