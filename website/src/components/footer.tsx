import Link from "next/link";
import { Github, Twitter, MessageSquare } from "lucide-react";

export function Footer() {
  return (
    <footer className="border-t border-border/50 mt-auto">
      <div className="mx-auto max-w-7xl px-6 py-10 flex flex-col md:flex-row items-center justify-between gap-6">
        <div className="flex flex-col items-center md:items-start gap-2">
          <p className="text-sm text-muted-foreground/60">
            © {new Date().getFullYear()} Mosaic · MIT License
          </p>
          <div className="flex gap-4 text-xs text-muted-foreground/40">
            <Link href="/docs/terms" className="hover:text-foreground transition-colors">Terms</Link>
            <Link href="/docs/privacy" className="hover:text-foreground transition-colors">Privacy</Link>
          </div>
        </div>
        
        <div className="flex items-center gap-6 text-muted-foreground/60">
          <Link href="/docs" className="hover:text-foreground transition-colors text-sm font-medium">Docs</Link>
          <Link href="/packages" className="hover:text-foreground transition-colors text-sm font-medium">Packages</Link>
          <div className="h-4 w-[1px] bg-border" />
          <Link href="https://discord.gg/QeJVQ3emNp" target="_blank" className="hover:text-foreground transition-colors" aria-label="Discord">
            <MessageSquare className="h-5 w-5" />
          </Link>
          <Link href="https://x.com/getmosaic" target="_blank" className="hover:text-foreground transition-colors" aria-label="Twitter">
            <Twitter className="h-5 w-5" />
          </Link>
          <Link href="https://github.com/doshibadev/mosaic" target="_blank" className="hover:text-foreground transition-colors" aria-label="GitHub">
            <Github className="h-5 w-5" />
          </Link>
        </div>
      </div>
    </footer>
  );
}
