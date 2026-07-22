// Generated from the aerial deployment v1 contract. Do not add authorization or safety rules here.
export type AerialViewKind = "qualification_matrix"|"assembly_manifest"|"load_approval"|"corridor_map"|"exclusion_map"|"dispersion_map"|"release_checklist"|"dual_decisions"|"deployment_phase"|"cohort_stability"|"tether_tension"|"panel_health"|"footprint"|"degraded_zones"|"unaccounted_components"|"disposition";
export interface AerialSafetyProvenance {source_time:string;expires_at:string;uncertainty_bps:number;configuration_digest:string;evidence_digest:string;authority:string}
export interface AerialSafetyView<T=unknown> {api_version:"v1";scope:{tenant:string;region:string;incident:string};resource_id:string;kind:AerialViewKind;aggregate_version:number;provenance:AerialSafetyProvenance;value:T}
export type AerialAuthorityLane="aircraft"|"incident";
export type AerialActionOutcome="requested"|"accepted"|"executed"|"observed"|"failed";
export interface AerialOperatorCommand {id:string;scope:{tenant:string;region:string;incident:string};resource_id:string;action:string;authority_lane:AerialAuthorityLane;expected_version:number;irreversible:boolean;confirmation?:{command_digest:string;statement:string}}

export interface AerialTransport {request<T>(path:string,init:RequestInit):Promise<T>}
export class GeneratedAerialClient {
  constructor(private readonly transport:AerialTransport){}
  view<T>(incident:string,kind:AerialViewKind,resourceId:string,signal?:AbortSignal):Promise<AerialSafetyView<T>> {
    return this.transport.request(`/v1/aerial/incidents/${encodeURIComponent(incident)}/views/${kind}/${encodeURIComponent(resourceId)}`,{method:"GET",...(signal?{signal}:{})});
  }
  command(value:AerialOperatorCommand,idempotencyKey:string,signal?:AbortSignal):Promise<{command_id:string;outcome:AerialActionOutcome}> {
    return this.transport.request("/v1/aerial/commands",{method:"POST",headers:{"Content-Type":"application/json","Idempotency-Key":idempotencyKey},body:JSON.stringify(value),...(signal?{signal}:{})});
  }
}
