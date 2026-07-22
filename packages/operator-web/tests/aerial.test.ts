import {describe,it,expect} from "vitest";
import axe from "axe-core";
import {renderAerialSafetyView,renderIrreversibleConfirmation} from "../src/aerial";

describe("aerial operator surface",()=>{
  it("renders all safety provenance and independent authority lanes accessibly",async()=>{document.documentElement.lang="en";document.title="Aerial operations";document.body.innerHTML='<main id="root"></main>';const root=document.querySelector<HTMLElement>("#root")!;const digest=`sha256:${"a".repeat(64)}`;renderAerialSafetyView(root,{api_version:"v1",scope:{tenant:"t",region:"r",incident:"i"},resource_id:"m",kind:"corridor_map",aggregate_version:3,provenance:{source_time:"2026-01-01T00:00:00Z",expires_at:"2026-01-01T00:01:00Z",uncertainty_bps:10,configuration_digest:digest,evidence_digest:digest,authority:"incident-command"},value:{crs:"urn:ogc:def:crs:OGC::CRS84"}},[{authority:"aircraft",outcome:"accepted",updatedAt:"2026-01-01T00:00:01Z"},{authority:"incident",outcome:"observed",updatedAt:"2026-01-01T00:00:02Z"}]);expect(root.textContent).toContain("CRS84");expect((await axe.run(root,{runOnly:{type:"tag",values:["wcag2a","wcag2aa"]},rules:{"color-contrast":{enabled:false}}})).violations).toEqual([])});
  it("requires a checkbox and typed statement for irreversible actions",()=>{const html=renderIrreversibleConfirmation("Release",`sha256:${"b".repeat(64)}`);expect(html).toContain("required type=\"checkbox\"");expect(html).toContain("confirmation-statement")});
});
