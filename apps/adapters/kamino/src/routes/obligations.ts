import { Router, Request, Response, NextFunction } from "express";
import { ObligationService } from "../obligation";

export function createObligationsRouter(
  obligationService: ObligationService
): Router {
  const router = Router();

  router.get(
    "/:market_id/by-id/:obligation_id",
    async (req: Request, res: Response, next: NextFunction) => {
      try {
        const { market_id, obligation_id } = req.params;
        const obligation = await obligationService.getObligationById(
          market_id,
          obligation_id
        );
        res.json(obligation);
      } catch (err) {
        next(err);
      }
    }
  );

  router.get(
    "/:market_id/:owner",
    async (req: Request, res: Response, next: NextFunction) => {
      try {
        const { market_id, owner } = req.params;
        const obligations = await obligationService.getUserObligations(
          market_id,
          owner
        );
        res.json({ obligations });
      } catch (err) {
        next(err);
      }
    }
  );

  router.get(
    "/:market_id/:owner/:obligation_id",
    async (req: Request, res: Response, next: NextFunction) => {
      try {
        const { market_id, owner, obligation_id } = req.params;
        const obligation = await obligationService.getObligation(
          market_id,
          owner,
          obligation_id
        );
        res.json(obligation);
      } catch (err) {
        next(err);
      }
    }
  );

  return router;
}
