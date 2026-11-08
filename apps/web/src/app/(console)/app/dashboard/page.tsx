import type { Metadata } from "next";
import { DashboardView } from "@/components/console/dashboard-view";

export const metadata: Metadata = {
  title: "Dashboard",
  description: "Lending position overview, real-time portfolio metrics, and risk distribution",
};

export default function DashboardPage() {
  return <DashboardView />;
}
