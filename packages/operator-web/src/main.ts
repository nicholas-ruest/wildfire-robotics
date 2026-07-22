/// <reference types="vite/client" />
import "./styles.css";import {OperatorApiAdapter} from "./client";import {renderOperatorShell} from "./shell";
const root=document.querySelector<HTMLElement>("#app");if(!root)throw new Error("Missing application root");
root.innerHTML='<main id="workspace" tabindex="-1"><h1>Wildfire operations</h1><p role="status">Loading authorized operational views…</p></main>';
const client=new OperatorApiAdapter({baseUrl:import.meta.env.VITE_API_BASE_URL??location.origin,accessToken:()=>{const token=sessionStorage.getItem("operator_access_token");if(!token)throw new Error("Operator authentication is required");return token},tenant:document.documentElement.dataset.tenant??"unknown",region:document.documentElement.dataset.region??"unknown"});
try{renderOperatorShell(root,await client.snapshot())}catch(error){const message=error instanceof Error?error.message:"Unknown gateway failure";root.innerHTML=`<main id="workspace" tabindex="-1"><h1>Operational views unavailable</h1><p role="alert"></p><p>No stale or cross-tenant data was substituted.</p></main>`;root.querySelector("[role=alert]")!.textContent=message}
