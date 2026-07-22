import type {AerialSafetyView,AerialActionOutcome,AerialAuthorityLane} from "@wildfire-robotics/api-client/dist/aerial.generated.js";

const escape=(value:string)=>value.replace(/[&<>"']/g,c=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]??c));
export interface AerialDecisionRow {authority:AerialAuthorityLane;outcome:AerialActionOutcome;updatedAt:string}

/** Renders server-authoritative facts; it deliberately contains no release or authorization rule. */
export function renderAerialSafetyView(root:HTMLElement,view:AerialSafetyView,decisions:readonly AerialDecisionRow[]):void {
  root.innerHTML=`<section aria-labelledby="aerial-heading"><h2 id="aerial-heading">${escape(view.kind.replaceAll("_"," "))}</h2><p role="status">Authoritative view version ${view.aggregate_version}</p><dl><div><dt>Source time</dt><dd><time datetime="${escape(view.provenance.source_time)}">${escape(view.provenance.source_time)}</time></dd></div><div><dt>Fresh until</dt><dd><time datetime="${escape(view.provenance.expires_at)}">${escape(view.provenance.expires_at)}</time></dd></div><div><dt>Uncertainty</dt><dd>${view.provenance.uncertainty_bps} basis points</dd></div><div><dt>Configuration evidence</dt><dd><code>${escape(view.provenance.configuration_digest)}</code></dd></div><div><dt>Safety evidence</dt><dd><code>${escape(view.provenance.evidence_digest)}</code></dd></div><div><dt>Authority</dt><dd>${escape(view.provenance.authority)}</dd></div><div><dt>Map CRS</dt><dd>${escape(String((view.value as {crs?:unknown})?.crs??"not applicable"))}</dd></div></dl><h3>Independent decisions</h3><ul>${decisions.map(d=>`<li><strong>${escape(d.authority)}</strong>: ${escape(d.outcome)} <time datetime="${escape(d.updatedAt)}">${escape(d.updatedAt)}</time></li>`).join("")||"<li>No decision reported</li>"}</ul></section>`;
}

export function renderIrreversibleConfirmation(action:string,digest:string):string {
  return `<fieldset><legend>Confirm irreversible action</legend><p>${escape(action)}</p><label><input required type="checkbox" name="irreversible-confirmation" value="${escape(digest)}"> I understand this action cannot be reversed</label><label>Type <strong>CONFIRM IRREVERSIBLE ACTION</strong><input required name="confirmation-statement" autocomplete="off"></label></fieldset>`;
}
