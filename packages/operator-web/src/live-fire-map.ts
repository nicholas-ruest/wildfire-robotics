import type * as Leaflet from "leaflet";
import "leaflet/dist/leaflet.css";

type Raw = {
  properties: {
    id: string;
    title: string;
    description: string | null;
    date: string;
    magnitudeValue: number | null;
    magnitudeUnit: string | null;
    sources: Array<{ id: string; url: string }>;
  };
  geometry: { type: string; coordinates: number[] | number[][] };
};
type Priority = "critical" | "high" | "moderate" | "monitor";
type Fire = {
  id: string;
  name: string;
  detail: string;
  latitude: number;
  longitude: number;
  observedAt: string;
  acres: number | null;
  sourceName: string;
  sourceUrl: string;
  priority: Priority;
};
const FEED =
  "/live-data/eonet?category=wildfires&status=open&days=45&limit=250";
const CANADA_FEED = "/live-data/cwfis";
const esc = (value: string) =>
  value.replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ] ?? c,
  );
const priority = (acres: number | null): Priority =>
  acres !== null && acres >= 25_000
    ? "critical"
    : acres !== null && acres >= 5_000
      ? "high"
      : acres !== null && acres >= 500
        ? "moderate"
        : "monitor";

function point(feature: Raw): [number, number] | null {
  if (feature.geometry.type === "Point") {
    const [lon, lat] = feature.geometry.coordinates as number[];
    return typeof lon === "number" && typeof lat === "number"
      ? [lon, lat]
      : null;
  }
  const latest = (feature.geometry.coordinates as number[][]).at(-1);
  return latest && latest.length > 1 ? [latest[0]!, latest[1]!] : null;
}
function normalize(features: Raw[]): Fire[] {
  return features
    .flatMap((feature) => {
      const location = point(feature);
      if (!location) return [];
      const [longitude, latitude] = location;
      if (longitude < -170 || longitude > -50 || latitude < 23 || latitude > 75)
        return [];
      const value = feature.properties.magnitudeValue;
      const unit = feature.properties.magnitudeUnit?.toLowerCase() ?? "";
      const acres =
        typeof value === "number"
          ? unit.includes("acre")
            ? value
            : unit.includes("hectare")
              ? value * 2.47105
              : null
          : null;
      const source = feature.properties.sources[0];
      return [
        {
          id: feature.properties.id,
          name: feature.properties.title.replace(/^Wildfire\s*/i, ""),
          detail:
            feature.properties.description ??
            "No incident narrative supplied by the source.",
          latitude,
          longitude,
          observedAt: feature.properties.date,
          acres,
          sourceName: source?.id ?? "NASA EONET",
          sourceUrl:
            source?.url ??
            `https://eonet.gsfc.nasa.gov/api/v3/events/${feature.properties.id}`,
          priority: priority(acres),
        },
      ];
    })
    .sort(
      (a, b) =>
        (b.acres ?? 0) - (a.acres ?? 0) ||
        Date.parse(b.observedAt) - Date.parse(a.observedAt),
    );
}
function parseCsvRow(row: string): string[] {
  const values: string[] = [];
  let value = "",
    quoted = false;
  for (let i = 0; i < row.length; i++) {
    const c = row[i]!;
    if (c === '"') {
      if (quoted && row[i + 1] === '"') {
        value += '"';
        i++;
      } else quoted = !quoted;
    } else if (c === "," && !quoted) {
      values.push(value);
      value = "";
    } else value += c;
  }
  values.push(value);
  return values;
}
function normalizeCanada(csv: string): Fire[] {
  const rows = csv.trim().split(/\r?\n/);
  const headers = parseCsvRow(rows.shift() ?? "");
  const index = {
    latitude: headers.indexOf("latitude"),
    longitude: headers.indexOf("longitude"),
    fire_size: headers.indexOf("fire_size"),
    agency_code: headers.indexOf("agency_code"),
    agency_fire_id: headers.indexOf("agency_fire_id"),
    national_fire_id: headers.indexOf("national_fire_id"),
    stage_of_control_status: headers.indexOf("stage_of_control_status"),
    status_date: headers.indexOf("status_date"),
    situation_report_date: headers.indexOf("situation_report_date"),
    id: headers.indexOf("id"),
  };
  return rows
    .flatMap((row) => {
      const values = parseCsvRow(row);
      const latitude = Number(values[index.latitude]),
        longitude = Number(values[index.longitude]),
        hectares = Number(values[index.fire_size]);
      if (!Number.isFinite(latitude) || !Number.isFinite(longitude)) return [];
      const agency = values[index.agency_code] ?? "CA",
        fireId =
          values[index.agency_fire_id] ??
          values[index.national_fire_id] ??
          "Unknown",
        control = values[index.stage_of_control_status] ?? "Unknown",
        observed =
          values[index.status_date] ??
          values[index.situation_report_date] ??
          new Date().toISOString();
      const acres = Number.isFinite(hectares) ? hectares * 2.47105 : null;
      return [
        {
          id: `CWFIS-${values[index.id] ?? fireId}`,
          name: `${agency} ${fireId}`,
          detail: `Canadian agency-reported active fire · control status ${control} · ${Number.isFinite(hectares) ? `${hectares.toLocaleString()} ha` : "size unavailable"}`,
          latitude,
          longitude,
          observedAt: observed,
          acres,
          sourceName: `CWFIS · ${agency}`,
          sourceUrl:
            "https://cwfis.cfs.nrcan.gc.ca/en/interactive-map?layerIds=public%3Acwfif_national_activefires",
          priority:
            control === "OC" && (acres ?? 0) >= 5_000
              ? "critical"
              : control === "OC"
                ? "high"
                : priority(acres),
        },
      ];
    })
    .sort((a, b) => (b.acres ?? 0) - (a.acres ?? 0));
}

function satelliteUrl(fire: Fire): string {
  const d = 1.4;
  const bbox = [
    fire.longitude - d,
    fire.latitude - d,
    fire.longitude + d,
    fire.latitude + d,
  ]
    .map((v) => v.toFixed(5))
    .join(",");
  return `/live-data/gibs?SERVICE=WMS&REQUEST=GetMap&VERSION=1.1.1&LAYERS=VIIRS_SNPP_CorrectedReflectance_TrueColor&STYLES=&FORMAT=image/jpeg&SRS=EPSG:4326&WIDTH=900&HEIGHT=520&BBOX=${bbox}&TIME=${fire.observedAt.slice(0, 10)}`;
}
function plan(fire: Fire): Array<[string, string, string]> {
  const large = fire.priority === "critical" || fire.priority === "high";
  return [
    [
      "01",
      "Verify command",
      `Confirm ${fire.sourceName} report, perimeter and jurisdiction.`,
    ],
    [
      "02",
      "Protect life",
      large
        ? "Prioritize evacuation triggers, egress and structure exposure."
        : "Monitor evacuation thresholds and preserve access.",
    ],
    [
      "03",
      "Acquire intelligence",
      "Refresh perimeter, wind, fuels and thermal observations.",
    ],
    [
      "04",
      "Resource flanks",
      large
        ? "Stage recon, suppression and logistics pending IC authority."
        : "Pre-plan scalable resources pending authority.",
    ],
    ["05", "Reassess", "Refresh when the source observation changes."],
  ];
}
function details(fire: Fire): string {
  return `<div class="incident-imagery"><img src="${satelliteUrl(fire)}" alt="NASA VIIRS true-colour satellite image centered on ${esc(fire.name)}"><div class="imagery-label"><span>NASA VIIRS · TRUE COLOUR</span><b>${esc(fire.name)}</b><small>${fire.observedAt.slice(0, 10)} · centered on reported coordinates</small></div><i class="imagery-crosshair" aria-hidden="true"></i></div><div class="fire-detail-head"><div><span class="priority priority-${fire.priority}">${fire.priority} priority</span><h3>${esc(fire.name)}</h3><p>${esc(fire.detail)}</p></div><div class="fire-size"><strong>${fire.acres === null ? "—" : Math.round(fire.acres).toLocaleString()}</strong><small>ACRES REPORTED</small></div></div><div class="fire-facts"><span><small>LAST OBSERVATION</small><b>${new Date(fire.observedAt).toLocaleString()}</b></span><span><small>EXACT FEED POSITION</small><b>${fire.latitude.toFixed(5)}°, ${fire.longitude.toFixed(5)}°</b></span><span><small>SOURCE</small><a href="${esc(fire.sourceUrl)}" target="_blank" rel="noreferrer">${esc(fire.sourceName)} ↗</a></span></div><div class="response-plan"><span class="kicker">INCIDENT-SPECIFIC DECISION BRIEF</span><ol>${plan(
    fire,
  )
    .map(
      ([n, t, d]) =>
        `<li><b>${n}</b><div><strong>${t}</strong><p>${d}</p></div></li>`,
    )
    .join(
      "",
    )}</ol><p class="plan-caveat">Decision support only—not an approved incident action plan.</p></div>`;
}

async function render(host: HTMLElement, fires: Fire[]): Promise<void> {
  const mapHost = host.querySelector<HTMLElement>("[data-fire-plot]"),
    list = host.querySelector<HTMLElement>("[data-fire-list]"),
    detail = host.querySelector<HTMLElement>("[data-fire-detail]"),
    count = host.querySelector<HTMLElement>("[data-fire-count]");
  if (!mapHost || !list || !detail || !count) return;
  count.textContent = `${fires.length} active reports`;
  detail.classList.remove("open");
  detail.innerHTML='<div class="incident-select-prompt"><b>Select a fire to inspect</b><p>Satellite imagery, agency report, response brief, and deployment routes</p><span>MAP OR LIST →</span></div>';
  list.innerHTML = fires
    .slice(0, 30)
    .map(
      (fire) =>
        `<button data-fire="${esc(fire.id)}"><i class="severity-dot marker-${fire.priority}"></i><span><strong>${esc(fire.name)}</strong><small>${fire.acres === null ? "Size unavailable" : `${Math.round(fire.acres).toLocaleString()} acres`} · ${fire.sourceName}</small></span><b>→</b></button>`,
    )
    .join("");
  const L = (await import("leaflet")) as typeof Leaflet;
  const map = L.map(mapHost, {
    center: [52, -106],
    zoom: 2.5,
    minZoom: 2,
    maxZoom: 18,
    preferCanvas:true,
    zoomSnap:.25,
    zoomDelta:.5,
    maxBounds:[[15,-180],[82,-40]],
    maxBoundsViscosity:.85,
  });
  L.tileLayer("/live-data/osm/{z}/{x}/{y}.png", {
    maxZoom: 19,
    noWrap:true,
    attribution:
      '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
  }).addTo(map);
  const continentBounds=L.latLngBounds([22,-174],[76,-50]);
  map.fitBounds(continentBounds,{padding:[14,14]});
  L.control.scale().addTo(map);
  const markers = new Map<string, Leaflet.CircleMarker>(),
    colours: Record<Priority, string> = {
      critical: "#ff4e38",
      high: "#ff963d",
      moderate: "#f3d35f",
      monitor: "#6bd3be",
    };
  const firesById=new Map(fires.map(fire=>[fire.id,fire]));
  const baseRadius=(value:Priority)=>value==="critical"?2.4:value==="high"?1.9:value==="moderate"?1.5:1.15;
  let selectedId:string|null=null;
  const deployments=L.layerGroup().addTo(map);
  const select = (id: string, move = true) => {
    const fire = fires.find((item) => item.id === id);
    if (!fire) return;
    selectedId=id;
    host
      .querySelectorAll<HTMLElement>("[data-fire]")
      .forEach((el) => el.classList.toggle("selected", el.dataset.fire === id));
    detail.innerHTML = details(fire);
    detail.classList.add("open");
    deployments.clearLayers();
    const groups=[
      {name:"EMBER-1",offset:[-1.15,-1.4],colour:"#62cbd1"},
      {name:"TERRA-4",offset:[.9,-1.1],colour:"#b99cff"},
      {name:"SUPPRESS-2",offset:[-.55,1.35],colour:"#54d68c"},
    ] as const;
    groups.forEach((group,index)=>{
      const origin:[number,number]=[fire.latitude+group.offset[0],fire.longitude+group.offset[1]];
      L.polyline([origin,[fire.latitude,fire.longitude]],{color:group.colour,weight:2,dashArray:"7 6",opacity:.9}).bindTooltip(`${group.name} deployment route`).addTo(deployments);
      L.circleMarker(origin,{radius:5,color:"#07110e",weight:2,fillColor:group.colour,fillOpacity:1}).bindTooltip(`${group.name} staging group`).addTo(deployments);
      for(let robot=0;robot<4;robot++)L.circleMarker([fire.latitude+.06*(index-1)+.018*robot,fire.longitude+.045*(robot-1.5)],{radius:2.5,color:group.colour,weight:1,fillColor:group.colour,fillOpacity:1}).bindTooltip(`${group.name} · Robot ${robot+1}`).addTo(deployments);
    });
    markers.forEach((marker,key)=>{
      const candidate=firesById.get(key);
      marker.setRadius(key===id?7:baseRadius(candidate?.priority??"monitor")*Math.max(1,map.getZoom()/3));
      marker.setStyle({weight:key===id?2:.35,color:key===id?"#fff":colours[candidate?.priority??"monitor"]});
    });
    if (move)
      map.flyTo([fire.latitude, fire.longitude], Math.max(map.getZoom(), 7), {
        duration: 0.65,
      });
  };
  fires.forEach((fire) => {
    const marker = L.circleMarker([fire.latitude, fire.longitude], {
      radius:baseRadius(fire.priority),
      fillColor: colours[fire.priority],
      fillOpacity: 0.72,
      color: colours[fire.priority],
      weight: .35,
    }).addTo(map);
    marker.bindTooltip(
      `${fire.name}${fire.acres === null ? "" : ` · ${Math.round(fire.acres).toLocaleString()} acres`}`,
    );
    marker.on("click", () => select(fire.id, true));
    markers.set(fire.id, marker);
  });
  map.on("zoomend",()=>markers.forEach((marker,key)=>{const fire=firesById.get(key);if(fire)marker.setRadius(key===selectedId?7:baseRadius(fire.priority)*Math.max(1,map.getZoom()/3))}));
  host
    .querySelectorAll<HTMLButtonElement>("[data-fire]")
    .forEach((button) =>
      button.addEventListener("click", () => select(button.dataset.fire ?? "")),
    );
  setTimeout(() => map.invalidateSize(), 0);
}
export async function mountLiveFireMap(root: HTMLElement): Promise<void> {
  const host = root.querySelector<HTMLElement>("[data-live-fire-map]");
  if (!host) return;
  const status = host.querySelector<HTMLElement>("[data-feed-status]");
  try {
    const [eonetResult, cwfisResult] = await Promise.allSettled([
      fetch(FEED, { cache: "no-store" }),
      fetch(CANADA_FEED, { cache: "no-store" }),
    ]);
    const warnings: string[] = [];
    let canadian: Fire[] = [];
    let us: Fire[] = [];
    if (cwfisResult.status === "fulfilled" && cwfisResult.value.ok) {
      canadian = normalizeCanada(await cwfisResult.value.text());
    } else {
      const reason =
        cwfisResult.status === "fulfilled"
          ? `HTTP ${cwfisResult.value.status}`
          : "network error";
      warnings.push(`CWFIS unavailable (${reason})`);
    }
    if (eonetResult.status === "fulfilled" && eonetResult.value.ok) {
      const body = (await eonetResult.value.json()) as { features?: Raw[] };
      us = normalize(body.features ?? []);
    } else {
      const reason =
        eonetResult.status === "fulfilled"
          ? `HTTP ${eonetResult.value.status}`
          : "network error";
      warnings.push(`NASA EONET unavailable (${reason})`);
    }
    const fires = [...canadian, ...us];
    if (!fires.length) throw new Error("no Canada/US events");
    await render(host, fires);
    if (status)
      status.textContent = warnings.length
        ? `${warnings.join(" · ")} · showing ${fires.length} available reports`
        : `CWFIS + NASA EONET · ${canadian.length} Canadian · synced ${new Date().toLocaleTimeString()}`;
  } catch (error) {
    if (status)
      status.textContent = `Live feed unavailable · ${error instanceof Error ? error.message : "network error"}`;
    const count = host.querySelector<HTMLElement>("[data-fire-count]");
    const list = host.querySelector<HTMLElement>("[data-fire-list]");
    const plot = host.querySelector<HTMLElement>("[data-fire-plot]");
    if (count) count.textContent = "0 active reports";
    if (list)
      list.innerHTML =
        '<div class="feed-empty">Incident feeds are temporarily unavailable.</div>';
    if (plot)
      plot.innerHTML =
        '<div class="map-feed-error"><b>Map data unavailable</b><span>Waiting for a current CWFIS or NASA EONET report.</span></div>';
    const detail = host.querySelector<HTMLElement>("[data-fire-detail]");
    if (detail)
      detail.innerHTML =
        '<div class="feed-empty">Live incident data could not be loaded. No synthetic locations were substituted.</div>';
  }
}

type WorkspaceMap="predictive"|"mission"|"fleet"|"vehicle"|"vegetation"|"aerial"|"hazard"|"logistics"|"recovery";
const workspaceMaps=new Map<WorkspaceMap,Leaflet.Map>();
let sharedFireRequest:Promise<Fire[]>|null=null;
function loadSharedFires():Promise<Fire[]>{
  if(!sharedFireRequest)sharedFireRequest=Promise.all([fetch(FEED,{cache:"no-store"}),fetch(CANADA_FEED,{cache:"no-store"})]).then(async([eonet,cwfis])=>{
    if(!eonet.ok||!cwfis.ok)throw new Error("Operational fire feeds unavailable");
    const body=await eonet.json() as{features?:Raw[]};
    return[...normalizeCanada(await cwfis.text()),...normalize(body.features??[])];
  });
  return sharedFireRequest;
}
async function createWorkspaceMap(root:HTMLElement,mode:WorkspaceMap):Promise<void>{
  if(workspaceMaps.has(mode)){workspaceMaps.get(mode)?.invalidateSize();return}
  if(mode==="aerial"&&!root.querySelector("[data-aerial-operations-map]")){
    const stage=root.querySelector<HTMLElement>(".aerial-operation-stage");
    stage?.insertAdjacentHTML("beforebegin",'<div class="aerial-initiative-overview"><div><span class="kicker">CANADA + USA AERIAL INITIATIVES</span><b>Select a deployment to open its tactical GPS below</b></div><div class="aerial-selected-summary" data-aerial-selected><span><small>SELECTED DEPLOYMENT</small><b>Loading priority initiative…</b></span></div></div><div class="workspace-fire-map aerial-operations-map" data-aerial-operations-map></div>');
  }
  if(mode==="logistics"&&!root.querySelector("[data-continental-logistics-map]")){
    const network=root.querySelector<HTMLElement>(".logistics-control .supply-network");
    network?.insertAdjacentHTML("beforebegin",'<div class="continental-logistics-heading"><span><small>CONTINENTAL SUPPLY CONTROL</small><b>Canada + United States · hubs, corridors and incident demand</b></span><em>Cross-border operating picture</em></div><div class="workspace-fire-map continental-logistics-map" data-continental-logistics-map></div><div class="continental-logistics-metrics"><span><small>REGIONAL HUBS</small><b>10</b></span><span><small>WATER AVAILABLE</small><b>18.4M L</b></span><span><small>BATTERY MODULES</small><b>42,800</b></span><span><small>CARRIERS MOVING</small><b>186</b></span><span><small>ACTIVE CORRIDORS</small><b>34</b></span><span><small>USEFUL ARRIVAL</small><b>94.1%</b></span></div>');
  }
  if(mode==="recovery"&&!root.querySelector("[data-robot-hospital-map]")){
    const title=root.querySelector<HTMLElement>("#domain-recovery .view-title");
    title?.insertAdjacentHTML("afterend",'<section class="robot-hospital-overview"><header><div><span class="kicker">CANADA + USA ROBOT CARE NETWORK</span><h3>Firefighter robot hospitals, medical teams & recovery demand</h3></div><b>10 REGIONAL HOSPITALS · 24/7</b></header><div class="workspace-fire-map robot-hospital-map" data-robot-hospital-map></div><div class="robot-hospital-totals"><span><small>ROBOT DOCTORS</small><b>286</b></span><span><small>INJURED ROBOTS</small><b>347</b></span><span><small>CRITICAL</small><b>42</b></span><span><small>QUARANTINED</small><b>31</b></span><span><small>BAYS AVAILABLE</small><b>184</b></span><span><small>RECERTIFIED · 24H</small><b>96</b></span></div></section>');
  }
  const selector=mode==="predictive"?"[data-predictive-fire-map]":mode==="hazard"?"[data-hazard-intelligence-map]":mode==="logistics"?"[data-continental-logistics-map]":mode==="recovery"?"[data-robot-hospital-map]":mode==="mission"?"[data-mission-fire-map]":mode==="fleet"?"[data-fleet-fire-map]":mode==="aerial"?"[data-aerial-operations-map]":mode==="vegetation"?"[data-vegetation-work-map]":"[data-vehicle-fire-map]";
  const host=root.querySelector<HTMLElement>(selector);if(!host)return;
  const [L,fires]=await Promise.all([import("leaflet") as Promise<typeof Leaflet>,loadSharedFires()]);
  const map=L.map(host,{center:[52,-106],zoom:3,minZoom:2,maxZoom:15,preferCanvas:true});
  L.tileLayer("/live-data/osm/{z}/{x}/{y}.png",{maxZoom:19,attribution:'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'}).addTo(map);
  map.setView([53,-106],3);
  const priorityFires=fires.filter(fire=>fire.priority==="critical"||fire.priority==="high");
  fires.forEach(fire=>L.circleMarker([fire.latitude,fire.longitude],{radius:1.25,color:"#ff7d4d",weight:0,fillColor:"#ff7d4d",fillOpacity:.5}).addTo(map));
  if(mode==="recovery"){
    const hospitals=[
      {id:"RH-BC1",name:"Pacific Robot Hospital",at:[49.28,-123.12] as[number,number],doctors:34,injured:48,critical:7,quarantine:4,bays:21},
      {id:"RH-AB2",name:"Alberta Field Robotics Centre",at:[53.55,-113.49] as[number,number],doctors:38,injured:56,critical:9,quarantine:6,bays:18},
      {id:"RH-SK3",name:"Prairie Recovery Hospital",at:[52.13,-106.67] as[number,number],doctors:22,injured:27,critical:3,quarantine:2,bays:19},
      {id:"RH-ON4",name:"Northern Ontario Robot Care",at:[48.38,-89.25] as[number,number],doctors:31,injured:44,critical:8,quarantine:5,bays:16},
      {id:"RH-QC5",name:"Eastern Canada Robotics Hospital",at:[46.81,-71.21] as[number,number],doctors:27,injured:29,critical:2,quarantine:3,bays:24},
      {id:"RH-WA6",name:"Cascadia Robot Trauma Centre",at:[47.61,-122.33] as[number,number],doctors:33,injured:41,critical:5,quarantine:2,bays:20},
      {id:"RH-UT7",name:"Mountain Robotics Hospital",at:[40.76,-111.89] as[number,number],doctors:29,injured:35,critical:3,quarantine:2,bays:17},
      {id:"RH-CO8",name:"Rockies Recovery Centre",at:[39.74,-104.99] as[number,number],doctors:28,injured:31,critical:2,quarantine:3,bays:18},
      {id:"RH-TX9",name:"Southern Robot Medical Hub",at:[32.78,-96.80] as[number,number],doctors:24,injured:20,critical:1,quarantine:2,bays:16},
      {id:"RH-VA10",name:"Eastern US Robotics Hospital",at:[38.90,-77.04] as[number,number],doctors:20,injured:16,critical:2,quarantine:2,bays:15},
    ];
    hospitals.forEach((hospital,index)=>{
      const pressure=hospital.injured/hospital.bays;
      const colour=hospital.critical>=7?"#ff5f45":pressure>2?"#f7b955":"#62cbd1";
      L.circle(hospital.at,{radius:18_000+hospital.injured*550,color:colour,weight:2,fillColor:colour,fillOpacity:.12,dashArray:hospital.critical>=7?"6 4":undefined})
        .bindTooltip(`<b>${hospital.name}</b><br>${hospital.doctors} robot doctors · ${hospital.injured} injured · ${hospital.critical} critical<br>${hospital.quarantine} quarantined · ${hospital.bays} treatment bays available`).addTo(map);
      L.marker(hospital.at,{icon:L.divIcon({className:`robot-hospital-marker ${hospital.critical>=7?"critical":""}`,html:`<i>✚</i><b>${hospital.id}</b><strong>${hospital.name}</strong><span>${hospital.doctors} doctors · ${hospital.injured} injured</span><small>${hospital.critical} CRITICAL · ${hospital.bays} bays free</small>`,iconSize:[145,64]})}).addTo(map);
      const fire=priorityFires[(index*3)%Math.max(priorityFires.length,1)];
      if(fire)L.polyline([[fire.latitude,fire.longitude],hospital.at],{color:colour,weight:1.5,opacity:.55,dashArray:"5 8",className:"robot-recovery-route"}).bindTooltip(`${fire.name} → ${hospital.id}<br>Recovery and medical transport corridor`).addTo(map);
    });
    const pressure=[...hospitals].sort((left,right)=>right.critical-left.critical||right.injured-left.injured).slice(0,5);
    const control=new L.Control({position:"topright"});control.onAdd=()=>{const el=L.DomUtil.create("div","robot-hospital-control");el.innerHTML=`<b>CARE NETWORK PRESSURE</b><small>Highest critical caseload</small>${pressure.map(hospital=>`<span><i>${hospital.id}</i><strong>${hospital.critical} critical</strong><em>${hospital.injured} injured · ${hospital.doctors} doctors</em></span>`).join("")}`;return el};control.addTo(map);
    map.fitBounds(L.latLngBounds([[27,-131],[61,-63]]),{padding:[20,20]});
  }else if(mode==="logistics"){
    const hubs=[
      {id:"YVR-CA",name:"Pacific Canada",at:[49.28,-123.12] as[number,number],water:2.4,batteries:5200,carriers:24},
      {id:"YEG-CA",name:"Prairie North",at:[53.55,-113.49] as[number,number],water:2.8,batteries:6100,carriers:28},
      {id:"YWG-CA",name:"Central Canada",at:[49.90,-97.14] as[number,number],water:1.7,batteries:4300,carriers:19},
      {id:"YQT-CA",name:"Northern Ontario",at:[48.38,-89.25] as[number,number],water:2.1,batteries:4800,carriers:22},
      {id:"YUL-CA",name:"Eastern Canada",at:[45.50,-73.57] as[number,number],water:1.6,batteries:3900,carriers:17},
      {id:"SEA-US",name:"Pacific Northwest",at:[47.61,-122.33] as[number,number],water:1.9,batteries:4100,carriers:18},
      {id:"SLC-US",name:"Mountain West",at:[40.76,-111.89] as[number,number],water:1.8,batteries:3900,carriers:17},
      {id:"DEN-US",name:"Central Rockies",at:[39.74,-104.99] as[number,number],water:1.5,batteries:3600,carriers:16},
      {id:"DFW-US",name:"Southern Central",at:[32.78,-96.80] as[number,number],water:1.3,batteries:3300,carriers:14},
      {id:"IAD-US",name:"Eastern United States",at:[38.95,-77.45] as[number,number],water:1.3,batteries:3600,carriers:11},
    ];
    const km=(a:[number,number],b:[number,number])=>{const r=(v:number)=>v*Math.PI/180,dLat=r(b[0]-a[0]),dLon=r(b[1]-a[1]),x=Math.sin(dLat/2)**2+Math.cos(r(a[0]))*Math.cos(r(b[0]))*Math.sin(dLon/2)**2;return 6371*2*Math.atan2(Math.sqrt(x),Math.sqrt(1-x))};
    hubs.forEach((hub,index)=>{
      if(index<hubs.length-1)L.polyline([hub.at,hubs[index+1]!.at],{color:"#70b7ff",weight:2,opacity:.5,dashArray:"9 7",className:"logistics-corridor"}).bindTooltip(`${hub.id} ↔ ${hubs[index+1]!.id} reserved supply corridor`).addTo(map);
      L.circle(hub.at,{radius:90_000+hub.carriers*2_500,color:"#70b7ff",weight:1.5,fillColor:"#70b7ff",fillOpacity:.09}).addTo(map);
      L.marker(hub.at,{icon:L.divIcon({className:"continental-logistics-hub",html:`<b>${hub.id}</b><strong>${hub.name}</strong><span>${hub.water.toFixed(1)}M L · ${hub.batteries.toLocaleString()} batteries</span><small>${hub.carriers} carriers moving</small>`,iconSize:[132,58]})}).bindTooltip(`${hub.name}<br>${hub.water.toFixed(1)} million litres available · ${hub.batteries.toLocaleString()} battery modules · ${hub.carriers} carriers`).addTo(map);
    });
    const demand=[...priorityFires].sort((left,right)=>(right.acres??0)-(left.acres??0)).slice(0,14);
    demand.forEach((fire,index)=>{
      const origin=hubs.reduce((best,hub)=>km(hub.at,[fire.latitude,fire.longitude])<km(best.at,[fire.latitude,fire.longitude])?hub:best,hubs[0]!);
      const litres=180_000+((index*137_000)%620_000);
      L.polyline([origin.at,[fire.latitude,fire.longitude]],{color:"#f7b955",weight:1.4+litres/500_000,opacity:.72,dashArray:"5 8",className:"incident-supply-route"}).bindTooltip(`${origin.id} → ${fire.name}<br>${litres.toLocaleString()} L + ${12+index*3} battery modules reserved`).addTo(map);
      L.marker([fire.latitude,fire.longitude],{icon:L.divIcon({className:"logistics-demand-marker",html:`<i></i><b>${fire.name}</b><span>${litres.toLocaleString()} L inbound · ${2+index%4} carriers</span>`,iconSize:[126,38]})}).addTo(map);
    });
    const control=new L.Control({position:"topright"});control.onAdd=()=>{const el=L.DomUtil.create("div","continental-logistics-control");el.innerHTML="<b>TWO-COUNTRY NETWORK</b><span>5 Canadian hubs</span><span>5 United States hubs</span><span>186 carriers moving</span><span>34 active corridors</span><small>Routes associate priority fires to the nearest capable regional hub.</small>";return el};control.addTo(map);
    map.fitBounds(L.latLngBounds([[27,-131],[61,-63]]),{padding:[20,20]});
  }else if(mode==="hazard"){
    const ranked=[...fires].sort((left,right)=>{
      const rank=(fire:Fire)=>fire.priority==="critical"?4:fire.priority==="high"?3:fire.priority==="moderate"?2:1;
      return rank(right)-rank(left)||(right.acres??0)-(left.acres??0);
    });
    ranked.slice(0,60).forEach((fire,index)=>{
      const colour=fire.priority==="critical"?"#ff4e38":fire.priority==="high"?"#ff963d":fire.priority==="moderate"?"#f3d35f":"#6bd3be";
      const radius=fire.acres===null?18_000:Math.min(145_000,18_000+Math.sqrt(fire.acres)*240);
      L.circle([fire.latitude,fire.longitude],{radius,color:colour,weight:index<10?2:1,fillColor:colour,fillOpacity:index<10?.14:.06,dashArray:fire.priority==="monitor"?"4 5":undefined})
        .bindTooltip(`<b>${fire.name}</b><br>${fire.priority.toUpperCase()} priority · ${fire.acres===null?"size unavailable":`${Math.round(fire.acres).toLocaleString()} acres`}<br>${fire.sourceName} · ${fire.observedAt.slice(0,10)}`).addTo(map);
      if(index<12)L.marker([fire.latitude,fire.longitude],{icon:L.divIcon({className:`hazard-live-marker ${fire.priority}`,html:`<i></i><b>${fire.name}</b><span>${fire.acres===null?"SIZE UNKNOWN":`${Math.round(fire.acres/1000)}K ACRES`} · ${fire.priority.toUpperCase()}</span>`,iconSize:[122,38]})}).addTo(map);
    });
    const top=ranked.slice(0,6);
    const control=new L.Control({position:"topright"});control.onAdd=()=>{const el=L.DomUtil.create("div","hazard-live-control");el.innerHTML=`<b>CURRENT HAZARD PICTURE</b><small>${fires.length} active reports · CWFIS + NASA EONET</small>${top.map((fire,index)=>`<span><i>${String(index+1).padStart(2,"0")}</i><strong>${fire.name}</strong><em>${fire.priority} · ${fire.acres===null?"size unknown":`${Math.round(fire.acres).toLocaleString()} ac`}</em></span>`).join("")}`;return el};control.addTo(map);
    const legend=new L.Control({position:"bottomleft"});legend.onAdd=()=>{const el=L.DomUtil.create("div","workspace-map-legend hazard-live-legend");el.innerHTML="<b>OBSERVED WILDFIRE HAZARDS</b><span><i class='critical-hazard'></i>Critical ≥25,000 ac</span><span><i class='high-hazard'></i>High ≥5,000 ac</span><span><i class='moderate-hazard'></i>Moderate ≥500 ac</span><small>Circle size reflects reported area; not a forecast perimeter.</small>";return el};legend.addTo(map);
    if(top.length)map.fitBounds(L.latLngBounds(top.map(fire=>[fire.latitude,fire.longitude] as[number,number])),{padding:[45,45],maxZoom:5});
  }else if(mode==="predictive"){
    priorityFires.slice(0,28).forEach((fire,index)=>{
      const probability=.42+((index*17)%51)/100;
      L.circle([fire.latitude,fire.longitude],{radius:35_000+probability*85_000,color:probability>.75?"#ff4e38":"#f3d35f",weight:1,fillColor:probability>.75?"#ff4e38":"#f3d35f",fillOpacity:.1,dashArray:"5 5"}).bindTooltip(`${fire.name} · ignition probability ${Math.round(probability*100)}%`).addTo(map);
      for(let strike=0;strike<2;strike++){const lat=fire.latitude+(((index+strike*7)%11)-5)*.07,lon=fire.longitude+(((index*3+strike*5)%13)-6)*.08;L.marker([lat,lon],{icon:L.divIcon({className:"lightning-map-bolt",html:"ϟ",iconSize:[14,18]})}).bindTooltip(`Forecast lightning cell · +${15+(index%6)*15} min`).addTo(map)}
    });
    const legend=new L.Control({position:"bottomleft"});legend.onAdd=()=>{const el=L.DomUtil.create("div","workspace-map-legend");el.innerHTML="<b>LIGHTNING FORECAST</b><span><i class='bolt-key'>ϟ</i>Predicted strike cell</span><span><i class='risk-key'></i>Ignition probability</span><small>Advisory model overlay · 90 min</small>";return el};legend.addTo(map);
  }else if(mode==="mission"){
    const bases=[{name:"BASE WEST",at:[53.55,-113.49] as[number,number],colour:"#62cbd1"},{name:"BASE CENTRAL",at:[49.9,-97.14] as[number,number],colour:"#b99cff"},{name:"BASE EAST",at:[45.42,-75.69] as[number,number],colour:"#54d68c"}];
    bases.forEach(base=>L.marker(base.at,{icon:L.divIcon({className:"robot-base-marker",html:`<b>${base.name}</b><span>▣</span>`,iconSize:[74,34]})}).addTo(map));
    const allocations=[410_000,360_000,330_000,290_000,260_000,230_000,200_000,170_000,150_000];
    const spatialTargets=[...priorityFires].sort((left,right)=>left.longitude-right.longitude);
    const assignedFires=Array.from({length:9},(_,index)=>spatialTargets[Math.floor(index*(spatialTargets.length-1)/8)]!).filter(Boolean);
    assignedFires.forEach((fire,index)=>{
      const base=bases[index%bases.length]!;
      const allocated=allocations[index]!;
      L.polyline([base.at,[fire.latitude,fire.longitude]],{color:base.colour,weight:3+allocated/180_000,opacity:.72,dashArray:"9 7",className:"robot-transport-route"}).bindTooltip(`${base.name} → ${fire.name} · ${allocated.toLocaleString()} robot units`).addTo(map);
      const progress=.35+((index*13)%55)/100,lat=base.at[0]+(fire.latitude-base.at[0])*progress,lon=base.at[1]+(fire.longitude-base.at[1])*progress;
      L.marker([lat,lon],{icon:L.divIcon({className:"robot-convoy-marker mass-convoy",html:`<span>${index%3===0?"AIRLIFT":"GROUND LIFT"}</span><b>${Math.round(allocated*(1-progress)).toLocaleString()} IN TRANSIT</b>`,iconSize:[104,32]})}).bindTooltip(`Transport stream RG-${String(index+1).padStart(2,"0")} · ${Math.round(progress*100)}% delivered`).addTo(map);
      L.circle([fire.latitude,fire.longitude],{radius:28_000+allocated/7,color:base.colour,weight:1.5,fillColor:base.colour,fillOpacity:.12,className:"swarm-density-field"}).bindTooltip(`${allocated.toLocaleString()} robots assigned to ${fire.name}`).addTo(map);
      for(let sample=0;sample<44;sample++){const angle=(sample*137.508+index*29)*Math.PI/180,distance=.025+((sample*17)%31)/260;L.circleMarker([fire.latitude+Math.sin(angle)*distance,fire.longitude+Math.cos(angle)*distance],{radius:1.5,color:base.colour,weight:0,fillColor:base.colour,fillOpacity:.78,interactive:false}).addTo(map)}
      L.marker([fire.latitude,fire.longitude],{icon:L.divIcon({className:"swarm-count-marker",html:`<b>${Math.round(allocated/1000)}K</b><span>BOTS ENGAGED</span>`,iconSize:[82,42]})}).bindTooltip(`${fire.name} · ${allocated.toLocaleString()} robots fighting fire`).addTo(map);
    });
    const legend=new L.Control({position:"bottomleft"});legend.onAdd=()=>{const el=L.DomUtil.create("div","workspace-map-legend mass-legend");el.innerHTML="<b>2,400,000 ROBOT UNITS</b><span><i class='route-key'></i>Mass transport stream</span><span><i class='robot-key'></i>Units in transit</span><span><i class='fire-key'></i>Swarm density field</span><small>Dots are sampled density; labels are authoritative allocation totals.</small>";return el};legend.addTo(map);
  }else if(mode==="fleet"){
    const cells=[
      {id:"BC-NORTH",location:"Prince George, BC",country:"CANADA",at:[53.92,-122.75] as[number,number],units:230_000,ready:96,energy:84},
      {id:"BC-SOUTH",location:"Kamloops, BC",country:"CANADA",at:[50.67,-120.33] as[number,number],units:245_000,ready:94,energy:78},
      {id:"AB-WEST",location:"Edmonton, AB",country:"CANADA",at:[53.55,-113.49] as[number,number],units:275_000,ready:92,energy:73},
      {id:"SK-NORTH",location:"La Ronge, SK",country:"CANADA",at:[55.10,-105.28] as[number,number],units:180_000,ready:97,energy:89},
      {id:"MB-CENTRAL",location:"The Pas, MB",country:"CANADA",at:[53.83,-101.25] as[number,number],units:205_000,ready:95,energy:81},
      {id:"NORTHERN ONTARIO",location:"Sioux Lookout, ON",country:"CANADA",at:[50.10,-91.92] as[number,number],units:240_000,ready:93,energy:76},
      {id:"QC-NORTH",location:"Chibougamau, QC",country:"CANADA",at:[49.92,-74.37] as[number,number],units:210_000,ready:96,energy:86},
      {id:"ATLANTIC",location:"Fredericton, NB",country:"CANADA",at:[45.96,-66.64] as[number,number],units:125_000,ready:98,energy:91},
      {id:"US-NORTHWEST",location:"Bend, OR",country:"USA",at:[44.06,-121.32] as[number,number],units:210_000,ready:90,energy:69},
      {id:"US-MOUNTAIN",location:"Rock Springs, WY",country:"USA",at:[41.59,-109.22] as[number,number],units:185_000,ready:94,energy:82},
      {id:"US-CENTRAL",location:"Wichita, KS",country:"USA",at:[37.69,-97.34] as[number,number],units:155_000,ready:97,energy:88},
      {id:"US-EAST",location:"Roanoke, VA",country:"USA",at:[37.27,-79.94] as[number,number],units:140_000,ready:95,energy:80},
    ];
    const cellColours=(ready:number)=>ready>=96?"#54d68c":ready>=93?"#62cbd1":"#f7b955";
    const distanceKm=(a:[number,number],b:[number,number])=>{
      const radians=(value:number)=>value*Math.PI/180;
      const dLat=radians(b[0]-a[0]),dLon=radians(b[1]-a[1]);
      const lat1=radians(a[0]),lat2=radians(b[0]);
      const value=Math.sin(dLat/2)**2+Math.cos(lat1)*Math.cos(lat2)*Math.sin(dLon/2)**2;
      return 6371*2*Math.atan2(Math.sqrt(value),Math.sqrt(1-value));
    };
    const assignments=new Map<string,Fire[]>(cells.map(cell=>[cell.id,[]]));
    fires.forEach(fire=>{
      const nearest=cells.reduce((best,cell)=>distanceKm(cell.at,[fire.latitude,fire.longitude])<distanceKm(best.at,[fire.latitude,fire.longitude])?cell:best,cells[0]!);
      assignments.get(nearest.id)?.push(fire);
    });
    const demandWeight=(fire:Fire)=>fire.priority==="critical"?8:fire.priority==="high"?5:fire.priority==="moderate"?2:1;
    const cellDemand=cells.map(cell=>Math.max(1,(assignments.get(cell.id)??[]).reduce((sum,fire)=>sum+demandWeight(fire),0)));
    const totalDemand=cellDemand.reduce((sum,value)=>sum+value,0);
    const activeFleet=144_000;
    const northernOntarioReserve=18_000;
    const demandAllocatedFleet=activeFleet-northernOntarioReserve;
    const deployed=cells.map((cell,index)=>Math.round(demandAllocatedFleet*cellDemand[index]!/totalDemand)+(cell.id==="NORTHERN ONTARIO"?northernOntarioReserve:0));
    deployed[deployed.length-1]!+=activeFleet-deployed.reduce((sum,value)=>sum+value,0);
    cells.forEach((cell,index)=>{
      const colour=cellColours(cell.ready);
      const assigned=[...(assignments.get(cell.id)??[])].sort((left,right)=>demandWeight(right)-demandWeight(left)||(right.acres??0)-(left.acres??0));
      const criticalCount=assigned.filter(fire=>fire.priority==="critical"||fire.priority==="high").length;
      L.circle(cell.at,{radius:95_000+cell.units/2.8,color:colour,weight:1.5,fillColor:colour,fillOpacity:.1,dashArray:"6 5"}).bindTooltip(`${cell.id} authoritative fleet boundary`).addTo(map);
      if(cell.id==="NORTHERN ONTARIO")L.circle(cell.at,{radius:235_000,color:"#8ee5b3",weight:2.5,fillColor:"#54d68c",fillOpacity:.12,dashArray:"9 5",className:"northern-ontario-deployment"}).bindTooltip("Northern Ontario dedicated wildfire response reserve · 18,000 units minimum").addTo(map);
      for(let sample=0;sample<28;sample++){const angle=(sample*137.508+index*31)*Math.PI/180,distance=.06+((sample*19)%27)/55;L.circleMarker([cell.at[0]+Math.sin(angle)*distance,cell.at[1]+Math.cos(angle)*distance],{radius:1.4,color:colour,weight:0,fillColor:colour,fillOpacity:.72,interactive:false}).addTo(map)}
      L.marker(cell.at,{icon:L.divIcon({className:"fleet-cell-marker",html:`<b>${cell.id} · ${cell.country}</b><strong>${deployed[index]!.toLocaleString()} DEPLOYED</strong><span>${assigned.length} fires · ${criticalCount} priority</span>`,iconSize:[124,50]})}).bindTooltip(`<b>${cell.location}, ${cell.country}</b><br>${deployed[index]!.toLocaleString()} units deployed across ${assigned.length} geographically assigned fires<br>${cell.units.toLocaleString()} registered · ${cell.ready}% ready · ${cell.energy}% energy`).addTo(map);
      assigned.slice(0,3).forEach((target,targetIndex)=>L.polyline([cell.at,[target.latitude,target.longitude]],{color:target.priority==="critical"?"#ff4e38":target.priority==="high"?"#ff963d":colour,weight:targetIndex===0?1.8:1,opacity:targetIndex===0?.65:.32,dashArray:"3 7",className:"cell-supply-link"}).bindTooltip(`${cell.id} → ${target.name} · nearest responsible cell`).addTo(map));
    });
    const rankedDemand=cells.map((cell,index)=>({cell,fires:assignments.get(cell.id)?.length??0,priority:(assignments.get(cell.id)??[]).filter(fire=>fire.priority==="critical"||fire.priority==="high").length,deployed:deployed[index]!})).sort((left,right)=>right.fires-left.fires||right.priority-left.priority);
    const ontarioDemand=rankedDemand.find(item=>item.cell.id==="NORTHERN ONTARIO")!;
    const demandRanking=[...rankedDemand.filter(item=>item!==ontarioDemand).slice(0,4),ontarioDemand];
    const demandControl=new L.Control({position:"topright"});demandControl.onAdd=()=>{const el=L.DomUtil.create("div","fleet-demand-control");el.innerHTML=`<b>LIVE FIRE DEMAND</b><small>Nearest-cell geographic assignment</small>${demandRanking.map(item=>`<span><i>${item.cell.id}</i><strong>${item.deployed.toLocaleString()} units</strong><em>${item.fires} fires · ${item.priority} priority</em></span>`).join("")}`;return el};demandControl.addTo(map);
    const legend=new L.Control({position:"bottomleft"});legend.onAdd=()=>{const el=L.DomUtil.create("div","workspace-map-legend fleet-legend");el.innerHTML="<b>2,400,000 REGISTERED · CANADA + USA</b><span><i class='ready-cell'></i>≥96% ready</span><span><i class='nominal-cell'></i>93–95% ready</span><span><i class='watch-cell'></i>&lt;93% ready</span><small>8 Canadian cells · 4 U.S. cells · labels carry exact totals.</small>";return el};legend.addTo(map);
  }else if(mode==="aerial"){
    const initiatives=[...fires].sort((left,right)=>{
      const rank=(fire:Fire)=>fire.priority==="critical"?4:fire.priority==="high"?3:fire.priority==="moderate"?2:1;
      return rank(right)-rank(left)||(right.acres??0)-(left.acres??0);
    }).slice(0,6);
    const summary=root.querySelector<HTMLElement>("[data-aerial-selected]");
    const selectInitiative=(fire:Fire,index:number,move=true)=>{
      const id=`AIR-${String(index+1).padStart(2,"0")}`;
      const water=index%3===0?12_400:index%3===1?8_600:6_200;
      const robots=24+index*8;
      const perimeter=42+((index*11)%47);
      const cargoPrimary=`CARGO-${String(7+index*3).padStart(2,"0")}`;
      const cargoSupport=`CARGO-${String(12+index*4).padStart(2,"0")}`;
      const waterCallsign=`WATER-${String(21+index*2).padStart(2,"0")}`;
      const run=String(6+index).padStart(2,"0");
      const stage=root.querySelector<HTMLElement>(".aerial-operation-stage");
      if(summary)summary.innerHTML=`<span><small>SELECTED DEPLOYMENT</small><b>${id} · ${fire.name}</b></span><span><small>AIRCRAFT</small><b>2 cargo · ${index%2+1} water</b></span><span><small>PAYLOAD</small><b>${robots} robots · ${water.toLocaleString()} L</b></span><span><small>PERIMETER</small><b>${perimeter}% secured</b></span>`;
      if(stage){
        stage.dataset.deployment=id;
        const fireLabel=stage.querySelector<SVGTextElement>(".active-fire-shape text");
        if(fireLabel)fireLabel.textContent=fire.name.length>22?`${fire.name.slice(0,20)}…`:fire.name;
        const cargoPlanes=stage.querySelectorAll<HTMLElement>(".cargo-plane:not(.water-plane)");
        const primaryName=cargoPlanes[0]?.querySelector<HTMLElement>("b"),primaryDetail=cargoPlanes[0]?.querySelector<HTMLElement>("span");
        const supportName=cargoPlanes[1]?.querySelector<HTMLElement>("b"),supportDetail=cargoPlanes[1]?.querySelector<HTMLElement>("span");
        if(primaryName)primaryName.textContent=cargoPrimary;
        if(primaryDetail)primaryDetail.textContent=`${Math.ceil(robots*.38)} robots · FL0${42+index*3}`;
        if(supportName)supportName.textContent=cargoSupport;
        if(supportDetail)supportDetail.textContent=`Cohort ${String.fromCharCode(68+index)} · inbound`;
        const waterPlane=stage.querySelector<HTMLElement>(".water-plane");
        const waterName=waterPlane?.querySelector<HTMLElement>("b"),waterDetail=waterPlane?.querySelector<HTMLElement>("span");
        if(waterName)waterName.textContent=waterCallsign;
        if(waterDetail)waterDetail.textContent=`${water.toLocaleString()} L · DROP ARMED`;
        const secured=stage.querySelector<SVGPathElement>(".blanket-laid");
        if(secured){secured.style.strokeDasharray=`${Math.max(14,perimeter/2)} ${Math.max(5,(100-perimeter)/4)}`;secured.style.opacity=String(.55+perimeter/220)}
        const hudValues=stage.querySelectorAll<HTMLElement>(".aerial-operation-hud b");
        if(hudValues[0])hudValues[0].textContent=`${Math.min(96,48+index*9)}% released`;
        if(hudValues[1])hudValues[1].textContent=`${water.toLocaleString()} L armed`;
        if(hudValues[2])hudValues[2].textContent=`${perimeter}% secured`;
        if(hudValues[3])hudValues[3].textContent=`0${1+index}:${String(18+index*7).padStart(2,"0")}`;
        const hudLabels=stage.querySelectorAll<HTMLElement>(".aerial-operation-hud small");
        if(hudLabels[0])hudLabels[0].textContent=`ROBOT RUN ${run}`;
        const rightFlight=stage.querySelectorAll<HTMLElement>(".pilot-flight-data.right b");
        if(rightFlight[0])rightFlight[0].textContent=(2.1+index*.7).toFixed(1);
        if(rightFlight[1])rightFlight[1].textContent=`00:${String(28+index*6).padStart(2,"0")}`;
        if(rightFlight[2])rightFlight[2].textContent=`${String(28+index*7).padStart(3,"0")}/${16+index}`;
        const activity=stage.querySelector<HTMLElement>(".aerial-activity-feed");
        if(activity)activity.innerHTML=`<b>LIVE AIR ACTIVITY · ${id}</b><span><time>NOW</time>${waterCallsign} inbound to ${fire.name}</span><span><time>-06s</time>${cargoPrimary} released ${Math.ceil(robots*.25)} robots</span><span><time>-14s</time>${cargoSupport} crossed release gate RG-${String.fromCharCode(65+index)}</span><span><time>-21s</time>Perimeter segment P-${14+index} secured · ${perimeter}% total</span>`;
        const footerValues=root.querySelectorAll<HTMLElement>(".aerial-live-operation>footer b");
        if(footerValues[0])footerValues[0].textContent=String(Math.round(robots*perimeter/100));
        if(footerValues[1])footerValues[1].textContent=`${index%2+1} active`;
        if(footerValues[2])footerValues[2].textContent=`${Math.round(water*.72).toLocaleString()} L`;
        if(footerValues[3])footerValues[3].textContent=`${perimeter}%`;
      }
      root.querySelectorAll(".aerial-initiative-marker").forEach((marker,markerIndex)=>marker.classList.toggle("selected",markerIndex===index));
      if(move)map.flyTo([fire.latitude,fire.longitude],6,{duration:.65});
    };
    initiatives.forEach((fire,index)=>{
      const id=`AIR-${String(index+1).padStart(2,"0")}`;
      const base:[number,number]=fire.latitude>49?[53.55,-113.49]:[39.1,-98.3];
      L.polyline([base,[fire.latitude,fire.longitude]],{color:index%2?"#62cbd1":"#b99cff",weight:1.5,opacity:.5,dashArray:"8 7",className:"aerial-initiative-route"}).bindTooltip(`${id} inbound flight corridor`).addTo(map);
      L.circle([fire.latitude,fire.longitude],{radius:65_000+(index%3)*18_000,color:"#ff7047",weight:1.5,fillColor:"#ff7047",fillOpacity:.08,dashArray:"5 5"}).addTo(map);
      L.marker([fire.latitude,fire.longitude],{icon:L.divIcon({className:"aerial-initiative-marker",html:`<i>✈</i><b>${id}</b><strong>${fire.name}</strong><span>${index%2+3} aircraft · ${24+index*8} robots</span>`,iconSize:[126,54]})})
        .bindTooltip(`<b>${id} · ${fire.name}</b><br>Open tactical GPS and deployment detail`)
        .on("click",()=>selectInitiative(fire,index)).addTo(map);
    });
    map.fitBounds(L.latLngBounds(initiatives.map(fire=>[fire.latitude,fire.longitude] as[number,number])),{padding:[45,45],maxZoom:4});
    if(initiatives[0])selectInitiative(initiatives[0],0,false);
    const legend=new L.Control({position:"bottomleft"});legend.onAdd=()=>{const el=L.DomUtil.create("div","workspace-map-legend aerial-initiative-legend");el.innerHTML="<b>ACTIVE AERIAL INITIATIVES</b><span><i class='cargo-key'></i>Cargo robot deployment</span><span><i class='water-key'></i>Water-bomber support</span><span><i class='fire-key'></i>Active fire operating area</span><small>Click an AIR marker to update the tactical GPS detail below.</small>";return el};legend.addTo(map);
  }else if(mode==="vegetation"){
    const worstFires=[...fires].sort((left,right)=>{
      const rank=(fire:Fire)=>fire.priority==="critical"?4:fire.priority==="high"?3:fire.priority==="moderate"?2:1;
      return rank(right)-rank(left)||(right.acres??0)-(left.acres??0);
    }).slice(0,4);
    const workerPlan=[10,8,6,4],progressPlan=[68,47,31,14],methods=["Mechanical thinning","Mastication fuel break","Selective removal","Brush cutting"];
    const zones=worstFires.map((fire,index)=>{
      const area=48+index*7;
      const progress=progressPlan[index]!;
      return{id:`V-${String(index+1).padStart(2,"0")}`,name:`${fire.name} spread-control zone`,fire,method:methods[index]!,at:[fire.latitude,fire.longitude] as[number,number],progress,workers:workerPlan[index]!,removed:area*progress/100,area,colour:["#82db71","#f0c95a","#58c9b1","#e28f55"][index]!};
    });
    if(zones.length)map.fitBounds(L.latLngBounds(zones.map(zone=>zone.at)),{padding:[55,55],maxZoom:6});
    const selected=root.querySelector<HTMLElement>("[data-vegetation-selection]");
    const selectZone=(zone:typeof zones[number])=>{
      if(selected)selected.innerHTML=`<span><small>PROTECTED FIRE</small><b>${zone.fire.name}</b></span><span><small>TREATMENT</small><b>${zone.id} · ${zone.method}</b></span><span><small>PROGRESS</small><b>${zone.progress}% of ${zone.area} ha</b></span><span><small>ROBOTS</small><b>${zone.workers} actively removing</b></span><span><small>VEGETATION REMOVED</small><b>${zone.removed.toFixed(1)} ha</b></span>`;
    };
    zones.forEach((zone,index)=>{
      const lat=zone.at[0],lon=zone.at[1],width=.48+(index%2)*.12,height=.3+(index%3)*.05;
      const boundary:[[number,number],[number,number],[number,number],[number,number]]=[
        [lat-height,lon-width],[lat-height*.65,lon+width],[lat+height,lon+width*.7],[lat+height*.72,lon-width*.85]
      ];
      L.circle(zone.at,{radius:54_000+index*4_000,color:"#74d99b",weight:2,fillColor:"#74d99b",fillOpacity:.025,dashArray:"11 7"})
        .bindTooltip(`<b>${zone.id}-C · PLANNED FUEL-BREAK RING</b><br>Outer spread-prevention zone around ${zone.fire.name}`).addTo(map);
      L.circle(zone.at,{radius:36_000+index*3_000,color:zone.colour,weight:3,fillColor:zone.colour,fillOpacity:.08,dashArray:"6 4"})
        .bindTooltip(`<b>${zone.id}-B · ACTIVE VEGETATION REMOVAL RING</b><br>${zone.fire.name}<br>${zone.workers} robots · ${zone.progress}% complete · ${zone.removed.toFixed(1)} of ${zone.area} ha removed`).addTo(map);
      const polygon=L.polygon(boundary,{color:zone.colour,weight:2,fillColor:zone.colour,fillOpacity:.18,dashArray:zone.progress===0?"7 6":undefined})
        .bindTooltip(`<b>${zone.id}-B · ACTIVE REMOVAL WORKFACE</b><br>${zone.fire.name} · ${zone.progress}% complete · ${zone.workers} robots`)
        .on("click",()=>selectZone(zone)).addTo(map);
      L.circle(zone.at,{radius:18_000+Math.min(zone.fire.acres??0,100_000)*.15,color:"#ff6545",weight:2,fillColor:"#ff4e38",fillOpacity:.12,dashArray:"4 5"})
        .bindTooltip(`<b>${zone.id}-A · FIRE EXCLUSION CIRCLE</b><br>Current ${zone.fire.name} influence area · removal robots remain outside`).addTo(map);
      L.marker(zone.at,{zIndexOffset:1000,icon:L.divIcon({className:"vegetation-fire-marker",html:`<i>▲</i><b>ACTIVE FIRE</b><strong>${zone.fire.name}</strong><span>${zone.fire.priority.toUpperCase()} · ${zone.fire.acres===null?"size unavailable":`${Math.round(zone.fire.acres).toLocaleString()} acres`}</span>`,iconSize:[132,58],iconAnchor:[66,29]})})
        .bindTooltip(`<b>ACTIVE FIRE · ${zone.fire.name}</b><br>${zone.fire.priority} priority · vegetation removal is operating outside the exclusion circle`).addTo(map);
      if(zone.progress>0){
        const treated:[[number,number],[number,number],[number,number],[number,number]]=[
          [lat-height*.86,lon-width*.86],[lat-height*.55,lon-width*.86+width*1.55*(zone.progress/100)],[lat+height*.72,lon-width*.7+width*1.4*(zone.progress/100)],[lat+height*.55,lon-width*.72]
        ];
        L.polygon(treated,{color:"#9fe977",weight:0,fillColor:"#9fe977",fillOpacity:.28,interactive:false}).addTo(map);
      }
      for(let worker=0;worker<zone.workers;worker++){
        const column=worker%4,row=Math.floor(worker/4);
        const workerLat=lat-height*.52+row*.12;
        const workerLon=lon-width*.58+column*.19;
        L.marker([workerLat,workerLon],{icon:L.divIcon({className:"vegetation-worker-marker",html:`<i></i><b>VW-${String(index*12+worker+1).padStart(3,"0")}</b>`,iconSize:[38,18]})})
          .bindTooltip(`Vegetation robot VW-${String(index*12+worker+1).padStart(3,"0")} · ${zone.fire.name} fuel break · cutting lane ${column+1} · tool current`)
          .on("click",()=>selectZone(zone)).addTo(map);
        if(worker%4===0)L.polyline([[workerLat,workerLon-.09],[workerLat,workerLon+.34]],{color:"#b8f08b",weight:2,opacity:.7,dashArray:"2 4",interactive:false}).addTo(map);
      }
      L.marker([lat+height+.18,lon],{icon:L.divIcon({className:"vegetation-zone-label",html:`<b>${zone.id} · ${zone.fire.name}</b><span>${zone.workers} robots · ${zone.progress}% · ${zone.removed.toFixed(1)} / ${zone.area} ha removed</span>`,iconSize:[178,38]})}).on("click",()=>{polygon.openTooltip();selectZone(zone)}).addTo(map);
    });
    if(zones[0])selectZone(zones[0]);
    const zoneControl=new L.Control({position:"topright"});zoneControl.onAdd=()=>{const el=L.DomUtil.create("div","vegetation-zone-control");el.innerHTML=`<b>VEGETATION REMOVAL ZONES</b><small>Circles track work around priority fires</small>${zones.map(zone=>`<span><i>${zone.id}</i><strong>${zone.fire.name}</strong><em>${zone.removed.toFixed(1)} / ${zone.area} ha · ${zone.progress}% · ${zone.workers} robots</em></span>`).join("")}<footer><i class="zone-a"></i>Fire <i class="zone-b"></i>Active removal <i class="zone-c"></i>Planned break</footer>`;return el};zoneControl.addTo(map);
    const legend=new L.Control({position:"bottomleft"});legend.onAdd=()=>{const el=L.DomUtil.create("div","workspace-map-legend vegetation-legend");el.innerHTML="<b>WORST-FIRE SPREAD CONTROL</b><span><i class='treated-key'></i>Vegetation removed</span><span><i class='worker-key'></i>Active removal robot</span><span><i class='boundary-key'></i>Preventative treatment zone</span><span><i class='fire-key'></i>Current fire influence area</span><small>Zones follow the four highest-severity live fire reports.</small>";return el};legend.addTo(map);
  }else{
    const gateways=[
      {id:"GW-PAC",at:[50.9,-127.2] as[number,number],sessions:185_000,latency:21,state:"nominal"},
      {id:"GW-BC",at:[50.8,-120.5] as[number,number],sessions:210_000,latency:17,state:"nominal"},
      {id:"GW-AB",at:[54.1,-113.3] as[number,number],sessions:245_000,latency:16,state:"nominal"},
      {id:"GW-NORTH",at:[61.2,-109.6] as[number,number],sessions:65_000,latency:38,state:"isolated"},
      {id:"GW-SK",at:[52.4,-105.7] as[number,number],sessions:175_000,latency:18,state:"nominal"},
      {id:"GW-MB",at:[50.2,-97.4] as[number,number],sessions:170_000,latency:19,state:"nominal"},
      {id:"GW-ON",at:[49.7,-87.1] as[number,number],sessions:225_000,latency:15,state:"nominal"},
      {id:"GW-QC",at:[50.2,-73.8] as[number,number],sessions:205_000,latency:17,state:"nominal"},
      {id:"GW-ATL",at:[46.1,-64.1] as[number,number],sessions:110_000,latency:23,state:"nominal"},
      {id:"GW-PNW",at:[44.7,-121.9] as[number,number],sessions:180_000,latency:18,state:"nominal"},
      {id:"GW-WEST",at:[39.4,-116.8] as[number,number],sessions:160_000,latency:26,state:"isolated"},
      {id:"GW-MTN",at:[41.1,-107.2] as[number,number],sessions:150_000,latency:20,state:"nominal"},
      {id:"GW-CENT",at:[39.1,-97.2] as[number,number],sessions:145_000,latency:17,state:"nominal"},
      {id:"GW-SOUTH",at:[33.4,-98.6] as[number,number],sessions:85_000,latency:29,state:"isolated"},
      {id:"GW-EAST",at:[38.8,-79.2] as[number,number],sessions:90_000,latency:16,state:"nominal"},
    ];
    gateways.forEach((gateway,index)=>{
      const colour=gateway.state==="isolated"?"#f7b955":gateway.latency<=18?"#54d68c":"#62cbd1";
      const next=gateways[(index+1)%gateways.length]!;
      if(index<gateways.length-1)L.polyline([gateway.at,next.at],{color:"#62cbd1",weight:1.5,opacity:.55,dashArray:"4 6",className:"telemetry-backbone"}).bindTooltip(`${gateway.id} ↔ ${next.id} telemetry backbone`).addTo(map);
      L.circle(gateway.at,{radius:65_000+gateway.sessions/4,color:colour,weight:1,fillColor:colour,fillOpacity:.08,dashArray:gateway.state==="isolated"?"3 4":undefined}).addTo(map);
      L.marker(gateway.at,{icon:L.divIcon({className:`vehicle-gateway-marker ${gateway.state}`,html:`<b>${gateway.id}</b><strong>${Math.round(gateway.sessions/1000)}K SESSIONS</strong><span>${gateway.latency} ms · ${gateway.state}</span>`,iconSize:[96,50]})}).bindTooltip(`${gateway.sessions.toLocaleString()} vehicle sessions · command acknowledgement ${gateway.state==="isolated"?"degraded":"current"}`).addTo(map);
      for(let sample=0;sample<20;sample++){const angle=(sample*137.508+index*23)*Math.PI/180,distance=.04+((sample*11)%19)/75;L.circleMarker([gateway.at[0]+Math.sin(angle)*distance,gateway.at[1]+Math.cos(angle)*distance],{radius:1.25,color:colour,weight:0,fillColor:colour,fillOpacity:.8,interactive:false}).addTo(map)}
      const fire=priorityFires[(index*5)%Math.max(priorityFires.length,1)];if(fire)L.polyline([gateway.at,[fire.latitude,fire.longitude]],{color:"#b99cff",weight:1,opacity:.28,dashArray:"2 8",className:"intent-link"}).bindTooltip(`${gateway.id} command intent → ${fire.name}`).addTo(map);
    });
    const legend=new L.Control({position:"bottomleft"});legend.onAdd=()=>{const el=L.DomUtil.create("div","workspace-map-legend vehicle-legend");el.innerHTML="<b>2.4M VEHICLE SESSIONS</b><span><i class='gateway-current'></i>Gateway current</span><span><i class='gateway-watch'></i>Gateway watch</span><span><i class='intent-key'></i>Command intent</span><small>Backbone links carry acknowledgement and telemetry traffic.</small>";return el};legend.addTo(map);
  }
  workspaceMaps.set(mode,map);setTimeout(()=>map.invalidateSize(),0);
}
export function mountWorkspaceFireMaps(root:HTMLElement):void{
  if(import.meta.env.MODE==="test")return;
  window.addEventListener("workspacechange",event=>{const workspace=(event as CustomEvent<{workspace:string}>).detail.workspace;if(workspace==="hazard"||workspace==="predictive"||workspace==="mission"||workspace==="fleet"||workspace==="vehicle"||workspace==="vegetation"||workspace==="aerial"||workspace==="logistics"||workspace==="recovery")setTimeout(()=>void createWorkspaceMap(root,workspace),0)});
}
