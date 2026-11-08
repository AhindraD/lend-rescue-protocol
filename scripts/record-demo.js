const { chromium } = require("playwright");
const path = require("path");
const fs = require("fs");
const { execSync } = require("child_process");

const ffmpegStatic = require("ffmpeg-static");

const TEMP_VIDEO_DIR = path.join("/tmp", "lendrescue-video-" + Date.now());
const OUTPUT_MP4_PATH = path.join(__dirname, "..", "artifacts", "demo", "lendrescue-demo.mp4");

async function recordDemo() {
  console.log("=== Starting LendRescue Product Demo Recording ===");
  fs.mkdirSync(TEMP_VIDEO_DIR, { recursive: true });
  fs.mkdirSync(path.dirname(OUTPUT_MP4_PATH), { recursive: true });

  const browser = await chromium.launch({
    headless: true,
    args: [
      "--no-sandbox",
      "--disable-setuid-sandbox",
      "--disable-dev-shm-usage",
      "--enable-webgl",
      "--use-gl=angle",
      "--use-angle=swiftshader",
    ],
  });

  const context = await browser.newContext({
    viewport: { width: 1600, height: 900 },
    recordVideo: {
      dir: TEMP_VIDEO_DIR,
      size: { width: 1600, height: 900 },
    },
  });

  const page = await context.newPage();
  page.setDefaultTimeout(15000);

  // Helper pause
  const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  try {
    // ─── Scene 1: Landing Page (0–5s) ─────────────────────────────────────────
    console.log("Scene 1: Loading Landing Page (http://localhost:3000/)...");
    await page.goto("http://localhost:3000/", { waitUntil: "networkidle" });
    await wait(5000);

    // ─── Scene 2: Dashboard (5–14s) ──────────────────────────────────────────
    console.log("Scene 2: Navigating to Dashboard...");
    // Click 'Launch Console' or use navbar
    const launchConsoleLink = page.locator('a[href="/app/dashboard"]').first();
    if (await launchConsoleLink.isVisible()) {
      await launchConsoleLink.click();
    } else {
      await page.goto("http://localhost:3000/app/dashboard");
    }
    await page.waitForSelector("h1:has-text('Dashboard')", { timeout: 10000 });
    // Let dashboard metrics and positions table load and stay readable
    await wait(8000);

    // ─── Scene 3: Positions List (14–20s) ────────────────────────────────────
    console.log("Scene 3: Navigating to Positions list...");
    await page.click('aside a[href="/app/positions"]');
    await page.waitForSelector("h1:has-text('Positions')", { timeout: 10000 });
    await wait(5000);

    // ─── Scene 4: Position Detail (20–27s) ───────────────────────────────────
    console.log("Scene 4: Inspecting Position Detail...");
    // Click on the first position or direct navigate to demo-pos-critical
    await page.goto("http://localhost:3000/app/positions/demo-pos-critical", {
      waitUntil: "networkidle",
    });
    await page.waitForSelector("h1:has-text('Position Inspection')", { timeout: 10000 });
    await wait(6000);

    // ─── Scene 5: Recovery Strategy Lab (27–42s) ─────────────────────────────
    console.log("Scene 5: Opening Recovery Strategy Lab...");
    // Click Open Recovery Lab link
    const recLabLink = page.locator('a[href*="/app/recovery"]').first();
    if (await recLabLink.isVisible()) {
      await recLabLink.click();
    } else {
      await page.goto("http://localhost:3000/app/recovery?positionId=demo-pos-critical");
    }
    await page.waitForSelector("h1:has-text('Recovery Strategy Lab')", { timeout: 10000 });
    await wait(2000);

    // Select 1.30 preset
    const preset130 = page.locator('button:has-text("1.30")');
    if (await preset130.isVisible()) {
      await preset130.click();
      await wait(1000);
    }

    // Click 'Run Optimizer'
    console.log("Scene 5: Running recovery optimizer...");
    const runBtn = page.locator('button:has-text("Run Optimizer")');
    if (await runBtn.isVisible()) {
      await runBtn.click();
      // Wait for proposal card to appear
      await page.waitForSelector("h3:has-text('Ranked Recovery Proposals')", { timeout: 10000 });
      await wait(3000);

      // Scroll slightly down to showcase the proposal breakdown & Jupiter quote
      await page.evaluate(() => window.scrollBy({ top: 380, behavior: "smooth" }));
      await wait(6000);
    } else {
      await wait(5000);
    }

    // ─── Scene 6: Activity & Observations (42–48s) ───────────────────────────
    console.log("Scene 6: Navigating to Activity...");
    await page.click('aside a[href="/app/activity"]');
    await page.waitForSelector("h1:has-text('Activity')", { timeout: 10000 });
    await wait(5000);

    // ─── Scene 7: Settings & Final Screen (48–53s) ───────────────────────────
    console.log("Scene 7: Navigating to Settings...");
    await page.click('aside a[href="/app/settings"]');
    await page.waitForSelector("h1:has-text('Settings')", { timeout: 10000 });
    await wait(3500);

    // Finish on Dashboard
    console.log("Scene 7: Finishing on Dashboard...");
    await page.click('aside a[href="/app/dashboard"]');
    await page.waitForSelector("h1:has-text('Dashboard')", { timeout: 10000 });
    await wait(2500);

    console.log("Recording flow completed successfully.");
  } finally {
    // Close page and context to ensure video file is finalized on disk
    await page.close();
    await context.close();
    await browser.close();
  }

  // Find the generated WebM file
  const videoFiles = fs.readdirSync(TEMP_VIDEO_DIR).filter((f) => f.endsWith(".webm"));
  if (videoFiles.length === 0) {
    throw new Error("No WebM video was produced by Playwright");
  }

  const rawWebmPath = path.join(TEMP_VIDEO_DIR, videoFiles[0]);
  console.log(`Raw WebM video captured: ${rawWebmPath} (${(fs.statSync(rawWebmPath).size / 1024 / 1024).toFixed(2)} MB)`);

  // Convert WebM to MP4 using static ffmpeg
  console.log(`Converting to H.264 MP4: ${OUTPUT_MP4_PATH}...`);
  const ffmpegCmd = `"${ffmpegStatic}" -y -i "${rawWebmPath}" -c:v libx264 -pix_fmt yuv420p -r 30 -an "${OUTPUT_MP4_PATH}"`;
  execSync(ffmpegCmd, { stdio: "inherit" });

  const finalStats = fs.statSync(OUTPUT_MP4_PATH);
  console.log(`Final MP4 video created successfully: ${OUTPUT_MP4_PATH} (${(finalStats.size / 1024 / 1024).toFixed(2)} MB)`);

  // Clean up temp directory
  fs.rmSync(TEMP_VIDEO_DIR, { recursive: true, force: true });
}

recordDemo().catch((err) => {
  console.error("Recording failed:", err);
  process.exit(1);
});

