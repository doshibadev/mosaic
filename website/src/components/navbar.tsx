"use client";

import Link from "next/link";
import Image from "next/image";
import { Github } from "lucide-react";

export function Navbar() {
  return (
    <header className="w-full border-b border-border/50">
      <div className="mx-auto flex h-20 max-w-7xl items-center justify-between px-8">
        <Link href="/" className="flex items-center gap-3">
          <Image src="/logo.png" alt="Mosaic" width={40} height={40} priority />
          <span className="text-2xl font-semibold text-foreground">mosaic</span>
        </Link>

        <nav className="flex items-center gap-8 text-lg">
          <Link href="/packages" className="text-muted-foreground hover:text-foreground transition-colors">
            Packages
          </Link>
          <Link href="/docs/getting-started" className="text-muted-foreground hover:text-foreground transition-colors">
            Docs
          </Link>
          <Link
            href="https://github.com/doshibadev/mosaic"
            target="_blank"
            rel="noreferrer"
            className="text-muted-foreground hover:text-foreground transition-colors"
          >
            <Github className="h-6 w-6" />
          </Link>
        </nav>
      </div>
    </header>
  );
}
