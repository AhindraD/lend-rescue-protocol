import type { Metadata } from "next";
import { ConsoleHeader } from "@/components/console/header";
import { EmptyState } from "@/components/ui/empty-state";

export const metadata: Metadata = {
  title: "Recovery Strategy Lab",
  description: "Model and compare recovery plans for at-risk lending positions",
};

export default function RecoveryPage() {
  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Recovery Strategy Lab"
        subtitle="Ranked candidate plans and indicative swap modeling"
      />
      <div className="flex-1 p-6">
        <EmptyState
          title="Select a position to model recovery"
          description="Phase 6D will implement the full Recovery Strategy Lab with candidate comparison and indicative quotes."
        />
      </div>
    </div>
  );
}
