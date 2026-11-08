import type { Metadata } from "next";
import { PositionDetailView } from "@/components/console/position-detail-view";

interface PageProps {
  params: Promise<{ id: string }>;
}

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  const { id } = await params;
  return {
    title: `Position: ${id.slice(0, 8)}...`,
    description: "Detailed lending obligation inspection and risk assessment",
  };
}

export default async function PositionDetailPage({ params }: PageProps) {
  const { id } = await params;
  return <PositionDetailView positionId={id} />;
}

