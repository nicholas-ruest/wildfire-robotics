import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const suppressionScene = defineWorkspaceScene({id:"suppression",metaphor:"Hydraulic target-envelope instrument",signature:"pumps-relays-dose-field",entityTypes:["pump","nozzle","target-envelope"],signal:"Measured flow",units:"L/min",managementWorkflow:"confirm an armed flow envelope"});
