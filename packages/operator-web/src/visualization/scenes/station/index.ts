import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const stationScene = defineWorkspaceScene({id:"station",metaphor:"Microgrid and habitat cutaway",signature:"bays-energy-particles-reserve",entityTypes:["energy-source","charge-bay","edge-workload"],signal:"Protected reserve",units:"kWh",managementWorkflow:"confirm a load-priority change"});
