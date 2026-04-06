import { createBrowserRouter } from "react-router";
import { Layout } from "./components/Layout";
import LandingPage from "./pages/LandingPage";
import Dashboard from "./pages/Dashboard";
import Flights from "./pages/Flights";
import Marketplace from "./pages/Marketplace";
import AssetDetails from "./pages/AssetDetails";
import Transactions from "./pages/Transactions";
import Profile from "./pages/Profile";
import Checkout from "./pages/Checkout";
import Signin from "./pages/Signin";
import Signup from "./pages/Signup";
import Cart from "./pages/Cart";
import OrderTracking from "./pages/OrderTracking";

export const router = createBrowserRouter([
  {
    path: "/",
    Component: Layout,
    children: [
      { index: true, Component: LandingPage },
      { path: "dashboard", Component: Dashboard },
      { path: "flights", Component: Flights },
      { path: "marketplace", Component: Marketplace },
      { path: "marketplace/:id", Component: AssetDetails },
      { path: "transactions", Component: Transactions },
      { path: "orders/:id", Component: OrderTracking },
      { path: "cart", Component: Cart },
      { path: "profile", Component: Profile },
      { path: "checkout", Component: Checkout },
      { path: "signin", Component: Signin },
      { path: "signup", Component: Signup },
    ],
  },
]);
