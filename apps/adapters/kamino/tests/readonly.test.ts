import { expect } from "chai";
import * as fs from "fs";
import * as path from "path";
import { KaminoClient } from "../src/client";
import { loadConfig } from "../src/config";

describe("Kamino Read-Only Boundary & Invariants", () => {
  it("proves KaminoClient does not contain any transaction construction or broadcast methods", () => {
    const config = loadConfig();
    const client = new KaminoClient(config);

    const forbiddenMethods = [
      "deposit",
      "withdraw",
      "borrow",
      "repay",
      "liquidate",
      "sendTransaction",
      "signTransaction",
      "buildTransaction",
      "createTransaction",
      "execute",
    ];

    for (const method of forbiddenMethods) {
      expect((client as any)[method]).to.be.undefined;
    }
  });

  it("proves source code contains no private key or transaction signing imports", () => {
    const srcDir = path.join(__dirname, "../src");
    const files = fs.readdirSync(srcDir, { recursive: true }) as string[];

    const forbiddenKeywords = [
      "Keypair",
      "Signer",
      "signTransaction",
      "sendAndConfirmTransaction",
      "sendTransaction",
      "depositReserveLiquidity",
      "borrowObligationLiquidity",
      "repayObligationLiquidity",
      "depositObligationCollateral",
      "withdrawObligationCollateral",
    ];

    for (const file of files) {
      const fullPath = path.join(srcDir, file);
      if (fs.statSync(fullPath).isFile() && file.endsWith(".ts")) {
        const content = fs.readFileSync(fullPath, "utf-8");
        for (const kw of forbiddenKeywords) {
          expect(
            content.includes(kw),
            `File ${file} must not contain forbidden write/signer keyword: '${kw}'`
          ).to.be.false;
        }
      }
    }
  });
});

