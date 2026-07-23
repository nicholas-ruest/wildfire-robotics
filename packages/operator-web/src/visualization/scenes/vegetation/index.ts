import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const vegetationScene = defineWorkspaceScene({id:"vegetation",metaphor:"Parcel-scale vegetation treatment twin",signature:"parcels-prescriptions-robot-swaths",entityTypes:["parcel","prescription","robot-swath"],signal:"Treatment progress",units:"percent",managementWorkflow:"approve a bounded work unit"});
