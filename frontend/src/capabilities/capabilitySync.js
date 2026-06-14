export function isCapabilitySyncBlocked(capabilityStatus, capabilitySource) {
  return capabilityStatus === "loading" || capabilitySource === "safe_fallback";
}
