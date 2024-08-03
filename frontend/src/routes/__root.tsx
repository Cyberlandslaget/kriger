import { createRootRoute, Outlet } from "@tanstack/react-router";
import NavigationBar from "../components/NavigationBar";
import { useWebSocketProvider, useConfigProvider } from "../utils/hooks";
import { CONFIG } from "../utils/constants";

export const RootComponent = () => {
  useWebSocketProvider(CONFIG.webSocketUrl);
  useConfigProvider();

  return (
    <>
      <NavigationBar />
      <div className="px-6">
        <Outlet />
      </div>
    </>
  );
};

export const Route = createRootRoute({
  component: RootComponent,
});
