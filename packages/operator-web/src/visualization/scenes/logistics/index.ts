import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const logisticsScene = defineWorkspaceScene({id:"logistics",metaphor:"Custody-aware supply flow landscape",signature:"custody-nodes-carriers-routes",entityTypes:["supply","custody-node","carrier"],signal:"Reserved supply",units:"kg",managementWorkflow:"stage and confirm a delivery reroute"});
