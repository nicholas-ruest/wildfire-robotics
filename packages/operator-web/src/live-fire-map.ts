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
  detail.innerHTML='<div class="incident-select-prompt"><b>Select a fire on the map</b><p>Open its satellite image, agency report, response brief, and robot deployment routes.</p></div>';
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
    const [eonet, cwfis] = await Promise.all([
      fetch(FEED, { cache: "no-store" }),
      fetch(CANADA_FEED, { cache: "no-store" }),
    ]);
    if (!eonet.ok) throw new Error(`US feed returned ${eonet.status}`);
    if (!cwfis.ok) throw new Error(`Canada feed returned ${cwfis.status}`);
    const body = (await eonet.json()) as { features?: Raw[] };
    const canadian = normalizeCanada(await cwfis.text());
    const fires = [...canadian, ...normalize(body.features ?? [])];
    if (!fires.length) throw new Error("no Canada/US events");
    await render(host, fires);
    if (status)
      status.textContent = `CWFIS + NASA EONET · ${canadian.length} Canadian · synced ${new Date().toLocaleTimeString()}`;
  } catch (error) {
    if (status)
      status.textContent = `Live feed unavailable · ${error instanceof Error ? error.message : "network error"}`;
    const detail = host.querySelector<HTMLElement>("[data-fire-detail]");
    if (detail)
      detail.innerHTML =
        '<div class="feed-empty">Live incident data could not be loaded. No synthetic locations were substituted.</div>';
  }
}

type WorkspaceMap="predictive"|"mission"|"fleet"|"vehicle"|"vegetation";
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
  const selector=mode==="predictive"?"[data-predictive-fire-map]":mode==="mission"?"[data-mission-fire-map]":mode==="fleet"?"[data-fleet-fire-map]":mode==="vegetation"?"[data-vegetation-work-map]":"[data-vehicle-fire-map]";
  const host=root.querySelector<HTMLElement>(selector);if(!host)return;
  const [L,fires]=await Promise.all([import("leaflet") as Promise<typeof Leaflet>,loadSharedFires()]);
  const map=L.map(host,{center:[52,-106],zoom:3,minZoom:2,maxZoom:15,preferCanvas:true});
  L.tileLayer("/live-data/osm/{z}/{x}/{y}.png",{maxZoom:19,attribution:'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'}).addTo(map);
  map.setView([53,-106],3);
  const priorityFires=fires.filter(fire=>fire.priority==="critical"||fire.priority==="high");
  fires.forEach(fire=>L.circleMarker([fire.latitude,fire.longitude],{radius:1.25,color:"#ff7d4d",weight:0,fillColor:"#ff7d4d",fillOpacity:.5}).addTo(map));
  if(mode==="predictive"){
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
      {id:"PAC-01",at:[53.2,-128.1] as[number,number],units:230_000,ready:96,energy:84},
      {id:"BC-02",at:[51.3,-121.4] as[number,number],units:245_000,ready:94,energy:78},
      {id:"AB-03",at:[54.4,-114.1] as[number,number],units:275_000,ready:92,energy:73},
      {id:"PR-04",at:[57.4,-106.2] as[number,number],units:180_000,ready:97,energy:89},
      {id:"MB-05",at:[52.7,-97.1] as[number,number],units:205_000,ready:95,energy:81},
      {id:"ON-06",at:[50.1,-87.6] as[number,number],units:240_000,ready:93,energy:76},
      {id:"QC-07",at:[50.8,-74.8] as[number,number],units:210_000,ready:96,energy:86},
      {id:"ATL-08",at:[46.4,-63.2] as[number,number],units:125_000,ready:98,energy:91},
      {id:"NW-09",at:[44.2,-121.8] as[number,number],units:210_000,ready:90,energy:69},
      {id:"MTN-10",at:[40.9,-110.2] as[number,number],units:185_000,ready:94,energy:82},
      {id:"CTR-11",at:[38.6,-98.3] as[number,number],units:155_000,ready:97,energy:88},
      {id:"EAST-12",at:[38.1,-79.4] as[number,number],units:140_000,ready:95,energy:80},
    ];
    const cellColours=(ready:number)=>ready>=96?"#54d68c":ready>=93?"#62cbd1":"#f7b955";
    cells.forEach((cell,index)=>{
      const colour=cellColours(cell.ready);
      L.circle(cell.at,{radius:95_000+cell.units/2.8,color:colour,weight:1.5,fillColor:colour,fillOpacity:.1,dashArray:"6 5"}).bindTooltip(`${cell.id} authoritative fleet boundary`).addTo(map);
      for(let sample=0;sample<28;sample++){const angle=(sample*137.508+index*31)*Math.PI/180,distance=.06+((sample*19)%27)/55;L.circleMarker([cell.at[0]+Math.sin(angle)*distance,cell.at[1]+Math.cos(angle)*distance],{radius:1.4,color:colour,weight:0,fillColor:colour,fillOpacity:.72,interactive:false}).addTo(map)}
      L.marker(cell.at,{icon:L.divIcon({className:"fleet-cell-marker",html:`<b>${cell.id}</b><strong>${Math.round(cell.units/1000)}K UNITS</strong><span>${cell.ready}% ready · ${cell.energy}% energy</span>`,iconSize:[94,50]})}).bindTooltip(`${cell.units.toLocaleString()} registered robots · ${Math.round(cell.units*cell.ready/100).toLocaleString()} allocatable`).addTo(map);
      const target=priorityFires[(index*7)%Math.max(priorityFires.length,1)];if(target)L.polyline([cell.at,[target.latitude,target.longitude]],{color:colour,weight:1,opacity:.3,dashArray:"3 7",className:"cell-supply-link"}).bindTooltip(`${cell.id} supplying ${target.name}`).addTo(map);
    });
    const legend=new L.Control({position:"bottomleft"});legend.onAdd=()=>{const el=L.DomUtil.create("div","workspace-map-legend fleet-legend");el.innerHTML="<b>2,400,000 REGISTERED</b><span><i class='ready-cell'></i>≥96% ready</span><span><i class='nominal-cell'></i>93–95% ready</span><span><i class='watch-cell'></i>&lt;93% ready</span><small>Particles are density samples; cell labels carry exact totals.</small>";return el};legend.addTo(map);
  }else if(mode==="vegetation"){
    map.setView([50.8,-117.5],5);
    const zones=[
      {id:"V-12",name:"Thompson Ridge",method:"Mechanical thinning",at:[50.52,-120.31] as[number,number],progress:63,workers:12,removed:38.4,area:61,colour:"#82db71"},
      {id:"V-08",name:"Cariboo Fuel Break",method:"Mastication",at:[52.63,-122.16] as[number,number],progress:41,workers:8,removed:21.7,area:53,colour:"#f0c95a"},
      {id:"V-21",name:"Rocky Mountain House",method:"Selective removal",at:[52.36,-114.92] as[number,number],progress:78,workers:8,removed:44.5,area:57,colour:"#58c9b1"},
      {id:"V-31",name:"Okanogan Corridor",method:"Brush cutting",at:[48.55,-119.62] as[number,number],progress:0,workers:0,removed:0,area:42,colour:"#7e9188"},
    ];
    const selected=root.querySelector<HTMLElement>("[data-vegetation-selection]");
    const selectZone=(zone:typeof zones[number])=>{
      if(selected)selected.innerHTML=`<span><small>SELECTED WORK AREA</small><b>${zone.id} · ${zone.name}</b></span><span><small>METHOD</small><b>${zone.method}</b></span><span><small>PROGRESS</small><b>${zone.progress}%</b></span><span><small>WORKERS</small><b>${zone.workers} ${zone.workers?"active":"staged"}</b></span><span><small>REMOVED</small><b>${zone.removed.toFixed(1)} ha</b></span>`;
    };
    zones.forEach((zone,index)=>{
      const lat=zone.at[0],lon=zone.at[1],width=.42+(index%2)*.13,height=.25+(index%3)*.05;
      const boundary:[[number,number],[number,number],[number,number],[number,number]]=[
        [lat-height,lon-width],[lat-height*.65,lon+width],[lat+height,lon+width*.7],[lat+height*.72,lon-width*.85]
      ];
      const polygon=L.polygon(boundary,{color:zone.colour,weight:2,fillColor:zone.colour,fillOpacity:.18,dashArray:zone.progress===0?"7 6":undefined})
        .bindTooltip(`<b>${zone.id} · ${zone.name}</b><br>${zone.progress}% complete · ${zone.workers} workers`)
        .on("click",()=>selectZone(zone)).addTo(map);
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
          .bindTooltip(`Vegetation worker VW-${String(index*12+worker+1).padStart(3,"0")} · cutting lane ${column+1} · tool current`)
          .on("click",()=>selectZone(zone)).addTo(map);
        if(worker%4===0)L.polyline([[workerLat,workerLon-.09],[workerLat,workerLon+.34]],{color:"#b8f08b",weight:2,opacity:.7,dashArray:"2 4",interactive:false}).addTo(map);
      }
      L.marker(zone.at,{icon:L.divIcon({className:"vegetation-zone-label",html:`<b>${zone.id}</b><span>${zone.progress}% · ${zone.removed.toFixed(1)} ha removed</span>`,iconSize:[116,34]})}).on("click",()=>{polygon.openTooltip();selectZone(zone)}).addTo(map);
    });
    const legend=new L.Control({position:"bottomleft"});legend.onAdd=()=>{const el=L.DomUtil.create("div","workspace-map-legend vegetation-legend");el.innerHTML="<b>ACTIVE FUEL TREATMENT</b><span><i class='treated-key'></i>Removal completed</span><span><i class='worker-key'></i>Vegetation worker</span><span><i class='boundary-key'></i>Authorized boundary</span><small>Click a boundary or worker to inspect the work area.</small>";return el};legend.addTo(map);
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
  window.addEventListener("workspacechange",event=>{const workspace=(event as CustomEvent<{workspace:string}>).detail.workspace;if(workspace==="predictive"||workspace==="mission"||workspace==="fleet"||workspace==="vehicle"||workspace==="vegetation")setTimeout(()=>void createWorkspaceMap(root,workspace),0)});
}
