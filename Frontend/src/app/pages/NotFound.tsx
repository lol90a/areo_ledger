import { Link } from "react-router";
import { Compass, ArrowRight } from "lucide-react";

export default function NotFound() {
  return (
    <div className="flex min-h-[70vh] flex-col items-center justify-center px-6 text-center">
      <div className="flex h-16 w-16 items-center justify-center rounded-2xl border border-amber-500/30 bg-amber-500/10">
        <Compass className="h-7 w-7 text-amber-500" />
      </div>
      <p className="mt-8 text-sm font-semibold uppercase tracking-[0.32em] text-amber-500">Error 404</p>
      <h1 className="mt-4 text-4xl md:text-5xl font-bold text-white">Page not found</h1>
      <p className="mt-4 max-w-md text-slate-400">
        This route has drifted off the flight plan. The page you are looking for does not exist or has been moved.
      </p>
      <Link
        to="/"
        className="mt-8 inline-flex items-center gap-2 rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black transition-all hover:bg-amber-400"
      >
        Back to Home <ArrowRight className="h-4 w-4" />
      </Link>
    </div>
  );
}
