import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const identityScene = defineWorkspaceScene({id:"identity",metaphor:"Zero-trust authority relationship graph",signature:"principals-grants-signatures",entityTypes:["principal","device","scoped-grant"],signal:"Grant lifetime",units:"min",managementWorkflow:"confirm a scoped grant revocation"});
