import { createRootRoute, Outlet } from "@tanstack/react-router";
import NavBar from "../components/NavBar";

export const Route = createRootRoute({
  component: () => (
    <>
      <NavBar />
      <div className="px-6">
        <Outlet />
      </div>
    </>
  ),
});
