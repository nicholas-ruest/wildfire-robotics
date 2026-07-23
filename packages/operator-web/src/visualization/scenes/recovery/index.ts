import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const recoveryScene = defineWorkspaceScene({id:"recovery",metaphor:"Robot recovery and hospital scene",signature:"medic-route-quarantine-repair",entityTypes:["recovery-case","medic-pod","repair-bay"],signal:"Case elapsed time",units:"min",managementWorkflow:"approve return to service"});
