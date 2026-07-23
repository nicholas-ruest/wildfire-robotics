import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const missionScene = defineWorkspaceScene({id:"mission",metaphor:"Spatial authorized mission graph",signature:"objectives-leases-geofences",entityTypes:["objective","lease","vehicle"],signal:"Lease remaining",units:"min",managementWorkflow:"confirm and dispatch a mission"});
