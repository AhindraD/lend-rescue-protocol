import { expect } from "chai";
import { KaminoClient } from "../src/client";
import { loadConfig } from "../src/config";
import { ValidationError } from "../src/errors";

describe("KaminoClient Address Validation & Config", () => {
  const config = loadConfig();
  const client = new KaminoClient(config);

  it("accepts valid Solana base58 public keys", () => {
    const valid = "7u3HeHxatcAACvdFtHGryaZWwZxKhqWisdHJaJBYbugj";
    const parsed = client.parseAddress(valid);
    expect(parsed).to.equal(valid);
  });

  it("rejects invalid Solana addresses with ValidationError", () => {
    expect(() => {
      client.parseAddress("invalid-address", "wallet");
    }).to.throw(ValidationError, /must be a valid base58 Solana address/);

    expect(() => {
      client.parseAddress("", "owner");
    }).to.throw(ValidationError, /must be a valid base58 Solana address/);
  });

  it("throws ValidationError for invalid programId in constructor", () => {
    const badConfig = { ...config, programId: "not-a-valid-pubkey" };
    expect(() => new KaminoClient(badConfig)).to.throw(
      ValidationError,
      /Invalid program ID/
    );
  });
});

