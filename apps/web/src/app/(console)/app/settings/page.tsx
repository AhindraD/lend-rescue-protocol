import type { Metadata } from "next";
import { SettingsView } from "@/components/console/settings-view";

export const metadata: Metadata = {
  title: "Settings & Diagnostics",
  description: "API environment, backend readiness status, and runtime security boundaries",
};

export default function SettingsPage() {
  return <SettingsView />;
}
