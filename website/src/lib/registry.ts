export interface RegistryPackage {
  name: string;
  description: string;
  author: string;
  version: string;
  repository?: string;
  download_count: number;
  updated_at?: number;
  readme?: string;
  license?: string;
  deprecated?: boolean;
  deprecation_reason?: string;
}

export interface RegistryVersion {
  version: string;
  created_at: number;
  dependencies: Record<string, string>;
  download_count?: number; // Not per version in DB yet, but maybe future proof
}

const REGISTRY_URL = process.env.NEXT_PUBLIC_REGISTRY_URL || "https://api.getmosaic.run";

interface SearchOptions {
  sort?: "updated" | "downloads" | "newest";
  limit?: number;
}

/// Searches the registry for packages.
/// 
/// Falls back to empty array if the API is down or times out (graceful degradation).
/// Uses Next.js ISR (Incremental Static Regeneration) with 60-second revalidation—
/// results are cached on the CDN and refreshed every minute.
export async function searchPackages(query: string = "", options: SearchOptions = {}): Promise<RegistryPackage[]> {
  try {
    const params = new URLSearchParams();
    if (query) params.set("q", query);
    if (options.sort) params.set("sort", options.sort);
    if (options.limit) params.set("limit", options.limit.toString());

    const res = await fetch(`${REGISTRY_URL}/packages/search?${params.toString()}`, {
      next: { revalidate: 60 }, // Cache results for 60 seconds on Vercel
      signal: AbortSignal.timeout(3000), // Timeout after 3s—fail fast instead of hanging
    });

    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    const data = await res.json();
    if (Array.isArray(data)) return data;
  } catch (err) {
    // API is down, slow, or returned garbage. Just return empty list.
    // The UI should handle this gracefully (show "no results" or something).
    console.error("Registry API error:", err);
  }

  return [];
}

/// Fetches a single package by name.
/// 
/// Uses ISR with 1-hour revalidation since individual package data changes less frequently.
/// Returns null if package not found or API is down.
export async function getPackage(name: string): Promise<RegistryPackage | null> {
  try {
    const res = await fetch(`${REGISTRY_URL}/packages/${encodeURIComponent(name)}`, {
      next: { revalidate: 3600 }, // Cache for 1 hour before revalidating
      signal: AbortSignal.timeout(3000), // Fail fast if API is slow
    });

    if (!res.ok) return null;

    return await res.json();
  } catch (err) {
    console.error("Registry API error:", err);
    return null;
  }
}

/// Fetches the version history for a package.
export async function getVersions(name: string): Promise<RegistryVersion[]> {
  try {
    const res = await fetch(`${REGISTRY_URL}/packages/${encodeURIComponent(name)}/versions`, {
      next: { revalidate: 3600 },
      signal: AbortSignal.timeout(3000),
    });

    if (!res.ok) return [];

    return await res.json();
  } catch (err) {
    console.error("Registry API error:", err);
    return [];
  }
}