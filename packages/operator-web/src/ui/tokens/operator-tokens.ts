export const OPERATOR_TOKENS = {
  version: "1.0.0",
  semantic: {
    dataQuality: {current: "quality-current", stale: "quality-stale", unknown: "quality-unknown"},
    health: {nominal: "health-nominal", degraded: "health-degraded"},
    severity: {attention: "severity-attention", critical: "severity-critical"},
    authorization: {permitted: "auth-permitted", denied: "auth-denied"},
    progress: {staged: "progress-staged", executing: "progress-executing"},
  },
  density: {
    comfortable: {space: "12px", target: "44px"},
    compact: {space: "6px", target: "44px"},
  },
  themes: {
    fieldDark: {background: "#080c0b", foreground: "#e7ece9"},
    highContrast: {background: "#000000", foreground: "#ffffff"},
  },
  capabilities: ["wide", "bounded", "narrow"],
} as const;
