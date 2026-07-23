import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const incidentScene = defineWorkspaceScene({id:"incident",metaphor:"Layered terrain command table",signature:"terrain-perimeter-divisions",entityTypes:["perimeter","division","resource"],signal:"Perimeter revision",units:"ha",managementWorkflow:"stage and publish an operational objective"});
