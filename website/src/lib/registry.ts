export interface RegistryPackage {
  name: string;
  description: string;
  author: string;
  version: string;
  repository?: string;
  downloads?: number;
  updated_at?: number;
  readme?: string;
}

const REGISTRY_URL = process.env.NEXT_PUBLIC_REGISTRY_URL || "https://api.getmosaic.run";

export async function searchPackages(query: string = ""): Promise<RegistryPackage[]> {
  try {
    const res = await fetch(`${REGISTRY_URL}/packages/search?q=${encodeURIComponent(query)}`, {
      next: { revalidate: 60 },
      signal: AbortSignal.timeout(3000), // Fail fast if API is slow/down
    });

    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    const data = await res.json();
    if (Array.isArray(data)) return data;
  } catch (err) {
    console.error("Registry API error:", err);
  }

  return [];
}

export async function getPackage(name: string): Promise<RegistryPackage | null> {
  try {
    const res = await fetch(`${REGISTRY_URL}/packages/${encodeURIComponent(name)}`, {
      next: { revalidate: 3600 },
      signal: AbortSignal.timeout(3000), // Fail fast if API is slow/down
    });

    if (!res.ok) return null;

    return await res.json();
  } catch (err) {
    console.error("Registry API error:", err);
    return null;
  }
}
