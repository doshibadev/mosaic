"use client";

import { useEffect, useState, Suspense } from "react";
import { Search, Loader2, ArrowRight } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { searchPackages, type RegistryPackage } from "@/lib/registry";

function PackagesContent() {
  const searchParams = useSearchParams();
  const initialQuery = searchParams?.get("q") || "";
  const [query, setQuery] = useState(initialQuery);
  const [packages, setPackages] = useState<RegistryPackage[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchPackages = async () => {
      setIsLoading(true);
      const data = await searchPackages(query);
      setPackages(data);
      setIsLoading(false);
    };

    const timer = setTimeout(fetchPackages, 300);
    return () => clearTimeout(timer);
  }, [query]);

  return (
    <div className="flex flex-col min-h-screen">
      <div className="mx-auto max-w-5xl w-full px-6 py-12 flex-1">
        <h1 className="text-3xl font-semibold text-foreground mb-8">Packages</h1>

        {/* Search */}
        <div className="relative mb-10">
          {isLoading ? (
            <Loader2 className="absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 animate-spin text-primary" />
          ) : (
            <Search className="absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-muted-foreground/50" />
          )}
          <input
            type="text"
            placeholder="Search packages..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="h-14 w-full rounded-lg border border-border bg-card pl-12 pr-4 text-lg text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary/50 transition-all"
          />
        </div>

        {/* Results */}
        {isLoading ? (
          <div className="space-y-3">
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="h-20 w-full animate-pulse rounded-lg bg-card" />
            ))}
          </div>
        ) : packages.length > 0 ? (
          <div className="space-y-3">
            {packages.map((pkg) => (
              <Link
                key={pkg.name}
                href={`/packages/${pkg.name}`}
                className="group flex items-center justify-between rounded-lg border border-border bg-card hover:border-primary/30 p-6 transition-all"
              >
                <div className="min-w-0">
                  <div className="flex items-baseline gap-3 mb-1">
                    <span className="font-mono font-medium text-lg text-foreground group-hover:text-primary transition-colors">
                      {pkg.name}
                    </span>
                    <span className="text-base text-muted-foreground/60 font-mono">
                      {pkg.version}
                    </span>
                  </div>
                  <p className="text-base text-muted-foreground">
                    {pkg.description || "No description"}
                  </p>
                </div>
                <ArrowRight className="h-5 w-5 shrink-0 ml-4 text-muted-foreground/20 group-hover:text-primary transition-colors" />
              </Link>
            ))}
          </div>
        ) : (
          <div className="text-center py-24">
            <p className="text-xl text-muted-foreground mb-2">
              {query ? `Nothing found for "${query}"` : "No packages yet"}
            </p>
            <p className="text-base text-muted-foreground/60">
              {query ? "Try a different search." : "Be the first to publish something."}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

export default function PackagesPage() {
  return (
    <Suspense>
      <PackagesContent />
    </Suspense>
  );
}
