import { Outlet, Link, useLocation, useNavigate } from "react-router";
import { Plane, ShoppingBag, Settings, Compass, Menu, X, ArrowRightLeft, Shield, ShoppingCart, Route, LogOut } from "lucide-react";
import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "motion/react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

import { getCartItems } from "../lib/commerce";
import { authEventName, clearAuthSession, getAuthSession, type AuthSession } from "../lib/auth";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function Layout() {
  const navigate = useNavigate();
  const [isScrolled, setIsScrolled] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [cartCount, setCartCount] = useState(0);
  const [authSession, setAuthSession] = useState<AuthSession | null>(null);
  const location = useLocation();

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 20);
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  useEffect(() => {
    setMobileMenuOpen(false);
  }, [location.pathname]);

  useEffect(() => {
    const syncCart = () => setCartCount(getCartItems().length);
    syncCart();
    window.addEventListener("aeroledger-cart-updated", syncCart);
    return () => window.removeEventListener("aeroledger-cart-updated", syncCart);
  }, []);

  useEffect(() => {
    const syncAuth = () => setAuthSession(getAuthSession());
    syncAuth();
    window.addEventListener(authEventName(), syncAuth);
    return () => window.removeEventListener(authEventName(), syncAuth);
  }, []);

  const handleSignOut = () => {
    clearAuthSession();
    navigate("/signin");
  };

  const navLinks = [
    { name: "Flights", path: "/flights", icon: Compass },
    { name: "Marketplace", path: "/marketplace", icon: ShoppingBag },
    { name: "Cart", path: "/cart", icon: ShoppingCart },
    { name: "Transactions", path: "/transactions", icon: ArrowRightLeft },
    { name: "Tracking", path: "/orders/live", icon: Route },
  ];

  return (
    <div className="min-h-screen bg-[#05050A] text-slate-200 font-sans flex flex-col selection:bg-amber-500/30">
      <header
        className={cn(
          "fixed top-0 w-full z-50 transition-all duration-300 border-b border-transparent",
          isScrolled
            ? "bg-[#0A0A0E]/80 backdrop-blur-xl border-white/5 py-3 shadow-2xl shadow-black/50"
            : "bg-gradient-to-b from-black/80 to-transparent py-5"
        )}
      >
        <div className="max-w-7xl mx-auto px-6 flex items-center justify-between">
          <Link to="/" className="flex items-center gap-3 group">
            <div className="w-8 h-8 rounded-full bg-gradient-to-tr from-amber-600 to-amber-400 flex items-center justify-center shadow-lg shadow-amber-500/20 group-hover:shadow-amber-500/40 transition-all">
              <Plane className="w-4 h-4 text-black fill-black" />
            </div>
            <span className="text-xl font-bold tracking-wider text-white">AERO<span className="text-amber-500 font-light">LEDGER</span></span>
          </Link>

          <nav className="hidden md:flex items-center gap-8">
            {navLinks.map((link) => {
              const isActive = link.name === "Tracking" ? location.pathname.startsWith("/orders") : location.pathname.startsWith(link.path);
              return (
                <Link
                  key={link.path}
                  to={link.path}
                  className={cn(
                    "text-sm font-medium tracking-wide transition-colors hover:text-amber-400 flex items-center gap-2",
                    isActive ? "text-amber-500" : "text-slate-400"
                  )}
                >
                  {link.name === "Cart" && cartCount > 0 ? (
                    <span className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-amber-500/15 px-1.5 text-[10px] text-amber-400">
                      {cartCount}
                    </span>
                  ) : (
                    isActive && <motion.div layoutId="nav-indicator" className="w-1 h-1 rounded-full bg-amber-500" />
                  )}
                  {link.name}
                </Link>
              );
            })}
          </nav>

          <div className="hidden md:flex items-center gap-4">
            {authSession ? (
              <>
                <Link to="/transactions" className="px-4 py-2.5 text-sm font-medium text-slate-300 transition-colors hover:text-white">
                  {authSession.name}
                </Link>
                <button onClick={handleSignOut} className="flex h-10 w-10 items-center justify-center rounded-full bg-white/5 border border-white/10 hover:bg-white/10 transition-colors">
                  <LogOut className="w-4 h-4 text-slate-300" />
                </button>
              </>
            ) : (
              <Link to="/signin" className="px-4 py-2.5 text-sm font-medium text-slate-300 transition-colors hover:text-white">
                Sign In
              </Link>
            )}
            <Link to="/cart" className="relative flex h-10 w-10 items-center justify-center rounded-full bg-white/5 border border-white/10 hover:bg-white/10 transition-colors">
              <ShoppingCart className="w-4 h-4 text-slate-300" />
              {cartCount > 0 && <span className="absolute -right-1 -top-1 flex h-5 min-w-5 items-center justify-center rounded-full bg-amber-500 px-1 text-[10px] font-bold text-black">{cartCount}</span>}
            </Link>
            <Link to="/profile" className="w-10 h-10 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:bg-white/10 transition-colors">
              <Settings className="w-4 h-4 text-slate-300" />
            </Link>
            <Link to="/flights" className="px-5 py-2.5 bg-amber-500 hover:bg-amber-400 text-black text-sm font-semibold rounded-full shadow-[0_0_15px_rgba(245,158,11,0.3)] hover:shadow-[0_0_25px_rgba(245,158,11,0.5)] transition-all">
              Book Flight
            </Link>
          </div>

          <button className="md:hidden text-slate-300 p-2" onClick={() => setMobileMenuOpen(!mobileMenuOpen)}>
            {mobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
          </button>
        </div>
      </header>

      <AnimatePresence>
        {mobileMenuOpen && (
          <motion.div
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            className="fixed inset-0 z-40 bg-[#05050A]/95 backdrop-blur-2xl pt-24 px-6 md:hidden"
          >
            <div className="flex flex-col gap-6">
              {navLinks.map((link) => (
                <Link
                  key={link.path}
                  to={link.path}
                  className="text-xl font-medium text-slate-200 flex items-center gap-4 py-2 border-b border-white/5"
                >
                  <link.icon className="w-6 h-6 text-amber-500" />
                  {link.name}
                  {link.name === "Cart" && cartCount > 0 && (
                    <span className="ml-auto inline-flex h-6 min-w-6 items-center justify-center rounded-full bg-amber-500 text-xs font-bold text-black">
                      {cartCount}
                    </span>
                  )}
                </Link>
              ))}
              <div className="pt-6 flex flex-col gap-4">
                <Link to="/profile" className="flex items-center gap-4 text-lg text-slate-300">
                  <Settings className="w-5 h-5" /> Account Settings
                </Link>
                {authSession ? (
                  <button onClick={handleSignOut} className="flex items-center gap-4 text-lg text-slate-300 text-left">
                    <LogOut className="w-5 h-5" /> Sign Out
                  </button>
                ) : (
                  <Link to="/signin" className="flex items-center gap-4 text-lg text-slate-300">
                    <Settings className="w-5 h-5" /> Sign In
                  </Link>
                )}
                <Link to="/flights" className="w-full text-center mt-4 px-6 py-4 bg-amber-500 text-black font-semibold rounded-xl">
                  Book a Flight
                </Link>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <main className="flex-1 pt-24 pb-12">
        <Outlet />
      </main>

      <footer className="border-t border-white/5 bg-[#0A0A0E] py-12 mt-auto">
        <div className="max-w-7xl mx-auto px-6 flex flex-col md:flex-row justify-between items-center gap-6">
          <div className="flex items-center gap-3">
            <div className="w-6 h-6 rounded-full bg-gradient-to-tr from-slate-600 to-slate-400 flex items-center justify-center">
              <Plane className="w-3 h-3 text-black fill-black" />
            </div>
            <span className="text-lg font-bold tracking-wider text-white">AERO<span className="text-slate-500 font-light">LEDGER</span></span>
          </div>
          <div className="flex gap-8 text-sm text-slate-500">
            <a href="#" className="hover:text-amber-500 transition-colors">Privacy</a>
            <a href="#" className="hover:text-amber-500 transition-colors">Terms</a>
            <a href="#" className="hover:text-amber-500 transition-colors">Support</a>
          </div>
          <div className="flex items-center gap-2 text-xs text-slate-600">
            <Shield className="w-4 h-4" />
            <span>Institutional Grade Security</span>
          </div>
        </div>
      </footer>
    </div>
  );
}

