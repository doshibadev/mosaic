use regex::Regex;
use std::sync::OnceLock;

/// Validates a package name against strict rules.
///
/// Rules:
/// 1. Lowercase alphanumeric and hyphens only (a-z, 0-9, -)
/// 2. No leading or trailing hyphens
/// 3. Length between 2 and 64 characters
/// 4. Not in the blocklist of offensive/reserved terms
pub fn validate_package_name(name: &str) -> Result<(), String> {
    // 1. Length check
    // 2 chars is minimum because "js" or "go" exists, but 1 char is just lazy.
    // 64 chars is plenty. If you need more, write a book, not a package name.
    if name.len() < 2 {
        return Err("Package name must be at least 2 characters long".to_string());
    }
    if name.len() > 64 {
        return Err("Package name must be at most 64 characters long".to_string());
    }

    // 2. Format check (regex)
    // ^[a-z0-9]        Starts with alphanumeric
    // [a-z0-9-]*       Middle can contain hyphens
    // [a-z0-9]$        Ends with alphanumeric (no trailing hyphen)
    // We use OnceLock because compiling regexes is expensive and I'm cheap.
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$").unwrap());

    if !re.is_match(name) {
        return Err("Package name must be lowercase alphanumeric with hyphens, and cannot start or end with a hyphen".to_string());
    }

    // 3. Double hyphen check
    // "my--package" looks ugly and confuses parsers. Don't do it.
    if name.contains("--") {
        return Err("Package name cannot contain consecutive hyphens".to_string());
    }

    // 4. Blocklist check
    // Because the internet is full of trolls and we can't have nice things without rules.
    if is_blocked(name) {
        return Err("Package name contains reserved or inappropriate words".to_string());
    }

    Ok(())
}

/// Checks if a name contains blocked terms.
fn is_blocked(name: &str) -> bool {
    let blocklist = [
        // System reserved
        // We reserve these so nobody pretends to be us.
        "admin", "root", "system", "mosaic", "registry", "official", "mod", "moderator",
        "polytoria", "staff", "security", "test", "example", "demo", "null", "undefined",
        "api", "dev", "beta", "stable", "latest", "internal",
        
        // Offensive / Inappropriate
        // This list is unfortunately necessary. It's not exhaustive, but it catches the
        // low-effort edgelords.
        "fuck", "shit", "nigger", "faggot", "cunt", "bitch", "whore", "slut", "dick",
        "pussy", "asshole", "bastard", "sex", "porn", "xxx", "kill", "suicide", "death",
        "hate", "nazi", "hitler", "kkk", "terrorist", "bomb", "murder", "rape",
    ];

    for term in blocklist {
        // Exact match is always blocked.
        // "root" is bad, but "beetroot" is a delicious vegetable (usually).
        if name == term {
            return true;
        }
        
        // Substring match for offensive terms.
        // We only check if the term is long enough to avoid the "ass" in "class" problem.
        if term.len() > 3 && name.contains(term) {
            // Check whitelist before flagging.
            // We don't want to ban "analytics" just because it has "anal" in it.
            if !is_whitelisted(name) {
                return true;
            }
        }
    }

    false
}

/// Returns true if the name contains a whitelisted term that might trigger a false positive.
fn is_whitelisted(name: &str) -> bool {
    // The "Scunthorpe problem" whitelist.
    // Words that look bad to a robot but are fine for humans.
    let whitelist = [
        "analytics", "analysis", "assassin", "assembly", "assets", "assistant",
        "association", "assume", "class", "classic", "classify", "pass", "password",
        "shell", "shithzu", "button", "push", "pull", "hello", "scraper", "grass",
    ];

    for safe in whitelist {
        if name.contains(safe) {
            return true;
        }
    }
    false
}