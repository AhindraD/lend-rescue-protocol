import { Suspense } from "react";
import type { Metadata } from "next";
import { RecoveryLabView } from "@/components/console/recovery-lab-view";
import { Skeleton } from "@/components/ui/skeleton";

export const metadata: Metadata = {
  title: "Recovery Strategy Lab",
  description: "Deterministic lending recovery candidate optimizer and indicative swap routing",
};

export default function RecoveryPage() {
  return (
    <Suspense
      fallback={
        <div className="p-6 space-y-6">
          <Skeleton className="h-10 w-64" />
          <Skeleton className="h-48 w-full rounded-xl" />
        </div>
      }
    >
      <RecoveryLabView />
    </Suspense>
  );
}
