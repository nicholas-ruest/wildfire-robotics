import {defineWorkspaceScene} from "../base/owned-workspace-scene";
export const commercialScene = defineWorkspaceScene({id:"commercial",metaphor:"Tenant entitlement and value-flow model",signature:"usage-ledger-slo-roi",entityTypes:["tenant","rated-event","ledger-entry"],signal:"Budget burn",units:"percent",managementWorkflow:"confirm a billing-period close"});
