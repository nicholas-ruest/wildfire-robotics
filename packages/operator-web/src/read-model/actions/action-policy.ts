export type ConnectivityPolicy = "online-required" | "stage-offline" | "edge-authority-required";
interface Connectivity {readonly online: boolean; readonly edgeAuthority: boolean}

export class ActionPolicyEngine {
  evaluate(policy: ConnectivityPolicy, connectivity: Connectivity) {
    if (policy === "online-required") {
      return {allowed: connectivity.online, submit: connectivity.online, revalidate: false};
    }
    if (policy === "stage-offline") {
      return {allowed: true, submit: connectivity.online, revalidate: !connectivity.online};
    }
    return {
      allowed: connectivity.edgeAuthority,
      submit: connectivity.edgeAuthority,
      revalidate: !connectivity.edgeAuthority,
    };
  }
  reconnect(): {autoSubmitted: 0} {
    return {autoSubmitted: 0};
  }
}
