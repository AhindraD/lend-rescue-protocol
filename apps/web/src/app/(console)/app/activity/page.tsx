import type { Metadata } from "next";
import { ConsoleHeader } from "@/components/console/header";
import { EmptyState } from "@/components/ui/empty-state";

export const metadata: Metadata = {
  title: "Activity",
  description: "Recent recovery recommendation history and oracle observations",
};

export default function ActivityPage() {
  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Activity"
        subtitle="Historical recovery recommendations and oracle feed updates"
      />
      <div className="flex-1 p-6">
        <EmptyState
          title="No recent activity logged"
          description="Historical recovery recommendations and oracle price feed events will be recorded here in Phase 6E."
        />
      </div>
    </div>
  );
}

