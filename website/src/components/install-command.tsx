"use client";

import { useState, useEffect } from "react";
import { Check, Copy, Terminal } from "lucide-react";

export function InstallCommand() {
  const [os, setOs] = useState<"unix" | "windows">("unix");
  const [copied, setCopied] = useState(false);

  // Derive the command from the OS state
  const command = os === "windows" 
    ? "irm https://getmosaic.run/install.ps1 | iex" 
    : "curl -fsSL https://getmosaic.run/install.sh | sh";

  // Detect the user's OS on mount.
  useEffect(() => {
    const isWin = window.navigator.userAgent.toLowerCase().includes("win");
    if (isWin) {
      // Use setTimeout to avoid synchronous state update warning during hydration
      setTimeout(() => setOs("windows"), 0);
    }
  }, []);

  // Copy to clipboard and show a brief success indicator.
  // The icon changes to a checkmark for 2 seconds, then reverts.
  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  return (
    <div className="w-full max-w-lg mx-auto mt-8">
      {/* Gradient border effect using pseudo-elements. The blur creates that glowing look. */}
      <div className="relative group">
        <div className="absolute -inset-0.5 bg-gradient-to-r from-primary to-secondary rounded-lg blur opacity-30 group-hover:opacity-50 transition duration-200"></div>
        <div className="relative flex items-center bg-card border border-border rounded-lg p-1 pr-2 shadow-xl">
          <div className="flex-shrink-0 pl-4 pr-3 text-muted-foreground">
            <Terminal className="w-5 h-5" />
          </div>
          <code className="flex-1 font-mono text-sm text-foreground overflow-x-auto whitespace-nowrap py-3 scrollbar-hide">
            {command}
          </code>
          <button
            onClick={copyToClipboard}
            className="flex-shrink-0 p-2 hover:bg-muted rounded-md transition-colors text-muted-foreground hover:text-foreground"
            aria-label="Copy install command"
          >
            {copied ? (
              <Check className="w-5 h-5 text-green-500" />
            ) : (
              <Copy className="w-5 h-5" />
            )}
          </button>
        </div>
      </div>
      <p className="text-center text-sm text-muted-foreground mt-3">
        Detected {os === "windows" ? "Windows" : "macOS/Linux"}.{" "}
        <button
          onClick={() => {
            // Let users manually switch OS if auto-detection got it wrong.
            setOs(os === "windows" ? "unix" : "windows");
          }}
          className="underline hover:text-foreground transition-colors"
        >
          Switch to {os === "windows" ? "macOS/Linux" : "Windows"}
        </button>
      </p>
    </div>
  );
}