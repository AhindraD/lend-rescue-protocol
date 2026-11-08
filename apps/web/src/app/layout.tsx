import type { Metadata, Viewport } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    template: "%s | LendRescue",
    default: "LendRescue — Lending Risk Intelligence",
  },
  description:
    "Understand lending risk. Compare recovery options. Make a better-informed decision before liquidation risk becomes critical.",
  keywords: ["solana", "lending", "defi", "risk", "recovery", "kamino"],
};

export const viewport: Viewport = {
  themeColor: "#07090d",
  colorScheme: "dark",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-[var(--background)] text-[var(--text-primary)] antialiased font-sans">
        {children}
      </body>
    </html>
  );
}
