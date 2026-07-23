import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const safetyScene = defineWorkspaceScene({id:"safety",metaphor:"Evidence-linked assurance case graph",signature:"hazards-claims-evidence-gates",entityTypes:["hazard","claim","evidence"],signal:"Evidence freshness",units:"hours",managementWorkflow:"approve or block a promotion gate"});
