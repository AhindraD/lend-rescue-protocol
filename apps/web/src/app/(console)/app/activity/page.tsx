import type { Metadata } from "next";
import { ActivityView } from "@/components/console/activity-view";

export const metadata: Metadata = {
  title: "Activity & Observations",
  description: "Pyth Core oracle price observations, market feed status, and position risk timelines",
};

export default function ActivityPage() {
  return <ActivityView />;
}
