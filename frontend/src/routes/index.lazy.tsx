// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import { createLazyFileRoute } from "@tanstack/react-router";
import SimpleDisplay from "../components/SimpleDisplay";
import { StatsCards } from "../components/StatsCards";

function DashboardPage() {
  return (
    <main className="relative grid grid-rows-[auto_1fr] gap-3 min-h-0 h-[calc(100vh-6rem)]">
      <StatsCards />
      <SimpleDisplay />
    </main>
  );
}

export const Route = createLazyFileRoute("/")({
  component: DashboardPage,
});
