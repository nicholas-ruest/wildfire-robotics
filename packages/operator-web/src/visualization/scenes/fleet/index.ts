import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const fleetScene = defineWorkspaceScene({id:"fleet",metaphor:"Hierarchical fleet-cell constellation",signature:"cells-epochs-readiness-orbits",entityTypes:["fleet-cell","vehicle","partition"],signal:"Eligible capacity",units:"vehicles",managementWorkflow:"stage a cohort rebalance"});
