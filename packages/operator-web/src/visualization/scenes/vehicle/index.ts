import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const vehicleScene = defineWorkspaceScene({id:"vehicle",metaphor:"Exploded vehicle subsystem twin",signature:"subsystems-command-pulses-faults",entityTypes:["subsystem","adapter","gateway-session"],signal:"Gateway latency",units:"ms",managementWorkflow:"request a safe-state transition"});
