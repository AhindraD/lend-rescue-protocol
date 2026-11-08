import type { Metadata } from "next";
import { PositionsView } from "@/components/console/positions-view";

export const metadata: Metadata = {
  title: "Positions",
  description: "Monitored lending obligations, collateral assets, and risk status",
};

export default function PositionsPage() {
  return <PositionsView />;
}
