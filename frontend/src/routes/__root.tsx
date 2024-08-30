import { createRootRoute, Outlet } from "@tanstack/react-router";
import NavigationBar from "../components/NavigationBar";
import {
  useWebSocketProvider,
  useConfigProvider,
  useCompetition,
} from "../utils/hooks";
import { CONFIG } from "../utils/constants";

export const RootComponent = () => {
  useWebSocketProvider(CONFIG.webSocketUrl);
  useConfigProvider();
  useCompetition();

  return (
    <div className="flex flex-col">
      <NavigationBar />
      <div className="px-6 flex-1 min-h-0">
        <Outlet />
      </div>
    </div>
  );
};

export const Route = createRootRoute({
  component: RootComponent,
});
