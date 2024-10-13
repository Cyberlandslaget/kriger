// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import { createRootRoute, Outlet } from "@tanstack/react-router";
import NavigationBar from "../components/NavigationBar";
import {
  useWebSocketProvider,
  useConfigProvider,
  useCompetition,
  useExploits,
} from "../utils/hooks";
import { CONFIG } from "../utils/constants";
import { Toaster } from "sonner";

export const RootComponent = () => {
  useWebSocketProvider(CONFIG.webSocketUrl);
  useConfigProvider();
  useCompetition();
  useExploits();

  return (
    <div className="flex flex-col">
      <NavigationBar />
      <div className="px-6 flex-1 min-h-0">
        <Outlet />
      </div>
      <Toaster expand={true} richColors toastOptions={{
        classNames: {
          toast: '!bg-slate-950 border-2',
          error: '!text-red-500 !border-red-500/50',
          success: '!text-green-400 !border-green-500',
          warning: '!text-amber-200 !border-amber-200/40',
          info: '!text-blue-400 !border-blue-400/50 !bg-slate-950',
          loading: '!text-white'
        },
      }} />
    </div>
  );
};

export const Route = createRootRoute({
  component: RootComponent,
});
