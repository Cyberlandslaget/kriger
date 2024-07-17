import { createRootRoute, Outlet } from "@tanstack/react-router";
import NavigationBar from "../components/NavigationBar";

export const Route = createRootRoute({
  component: () => (
    <>
      <NavigationBar />
      <div className="px-6">
        <Outlet />
      </div>
    </>
  ),
});
