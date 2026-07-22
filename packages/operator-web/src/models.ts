export const contexts=["incident","mission","fleet","station","logistics","hazard","safety","recovery"] as const;
export type Context=typeof contexts[number];
export type DataState="current"|"stale"|"gap"|"degraded"|"unknown";
export interface Provenance {source:string; observedAt:string; receivedAt:string; lineage:string;}
export interface ReadModel {context:Context; title:string; summary:string; state:DataState; uncertainty:string; provenance:Provenance; limitation?:string;}
export type CommandStage="accepted"|"acknowledged"|"executed"|"outcome-confirmed"|"outcome-unknown";
export interface CommandStatus {id:string; stage:CommandStage; detail:string; updatedAt:string;}
export interface OperatorSnapshot {tenant:string;region:string;generatedAt:string;models:ReadModel[];commands:CommandStatus[];}
