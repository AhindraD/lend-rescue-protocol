import { expect } from "chai";
import http from "http";
import { createApp } from "../src/main";

describe("Kamino Adapter HTTP Routes", () => {
  let server: http.Server;
  let baseUrl: string;

  before((done) => {
    const { app } = createApp();
    server = app.listen(0, "127.0.0.1", () => {
      const address = server.address() as any;
      baseUrl = `http://127.0.0.1:${address.port}`;
      done();
    });
  });

  after((done) => {
    server.close(done);
  });

  it("GET /health returns 200 with readOnly status and SDK version", async () => {
    const res = await fetch(`${baseUrl}/health`);
    expect(res.status).to.equal(200);
    const body = (await res.json()) as any;
    expect(body.status).to.equal("ok");
    expect(body.sdkVersion).to.equal("13.0.6");
    expect(body.readOnly).to.be.true;
    expect(body.version).to.equal("0.1.0");
  });

  it("GET /markets/invalid-market returns 400 validation error", async () => {
    const res = await fetch(`${baseUrl}/markets/invalid-market`);
    expect(res.status).to.equal(400);
    const body = (await res.json()) as any;
    expect(body.code).to.equal("VALIDATION_ERROR");
  });

  it("GET /obligations/invalid-market/invalid-owner returns 400 validation error", async () => {
    const res = await fetch(`${baseUrl}/obligations/invalid-market/invalid-owner`);
    expect(res.status).to.equal(400);
    const body = (await res.json()) as any;
    expect(body.code).to.equal("VALIDATION_ERROR");
  });
});
