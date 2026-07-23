import { defineConfig } from "vite";
export default defineConfig({
  test: { environment: "jsdom" },
  build: { target: "es2022" },
  server: {
    proxy: {
      "/live-data/eonet": {
        target: "https://eonet.gsfc.nasa.gov",
        changeOrigin: true,
        rewrite: (path) =>
          path.replace(/^\/live-data\/eonet/, "/api/v3/events/geojson"),
      },
      "/live-data/osm": {
        target: "https://tile.openstreetmap.org",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/live-data\/osm/, ""),
        headers: { "User-Agent": "WildfireRoboticsOperator/0.1" },
      },
      "/live-data/gibs": {
        target: "https://gibs.earthdata.nasa.gov",
        changeOrigin: true,
        rewrite: (path) =>
          path.replace(/^\/live-data\/gibs/, "/wms/epsg4326/best/wms.cgi"),
      },
      "/live-data/cwfis": {
        target: "https://cwfis.cfs.nrcan.gc.ca",
        changeOrigin: true,
        rewrite: (path) =>
          path.replace(
            /^\/live-data\/cwfis/,
            "/downloads/reportedfires/activefires.csv",
          ),
      },
    },
  },
});
