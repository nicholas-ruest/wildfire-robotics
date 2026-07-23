import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const hazardScene = defineWorkspaceScene({id:"hazard",metaphor:"Spatiotemporal observation confidence field",signature:"sensor-rays-confidence-surface",entityTypes:["observation","sensor","provenance-layer"],signal:"Confidence",units:"probability",managementWorkflow:"publish a bounded hazard picture"});
