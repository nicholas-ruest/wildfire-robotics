import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const aerialScene = defineWorkspaceScene({id:"aerial",metaphor:"Airspace-to-ground deployment sequence",signature:"aircraft-corridor-panels-tethers",entityTypes:["aircraft","payload-panel","anchor"],signal:"Maximum tether tension",units:"kN",managementWorkflow:"approve a deployment phase advance"});
