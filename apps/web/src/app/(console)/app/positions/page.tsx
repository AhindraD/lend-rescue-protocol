import type { Metadata } from "next";
import { ConsoleHeader } from "@/components/console/header";
import { EmptyState } from "@/components/ui/empty-state";

export const metadata: Metadata = {
  title: "Positions",
  description: "Monitored Kamino lending positions and risk assessments",
};

export default function PositionsPage() {
  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Positions"
        subtitle="Monitored lending obligations on Kamino"
      />
      <div className="flex-1 p-6">
        <EmptyState
          title="Positions will appear here"
          description="Phase 6C will implement the live position explorer and real-time risk status table."
        />
      </div>
    </div>
  );
}

