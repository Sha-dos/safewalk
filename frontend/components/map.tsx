"use client";

import { MapContainer, TileLayer, Marker, Popup, Polyline, Polygon } from "react-leaflet";
import "leaflet-defaulticon-compatibility";
import type { LatLngExpression } from "leaflet";
import { useEffect, useState } from "react";

const center: LatLngExpression = [33.423322, -111.932648];

// Represent the Overpass elements we care about
type OsmNode = {
  type: "node";
  id: number;
  lat: number;
  lon: number;
  tags?: Record<string, string>;
};

type OsmWay = {
  type: "way";
  id: number;
  geometry: { lat: number; lon: number }[];
  tags?: Record<string, string>;
};

type OsmElement = OsmNode | OsmWay;

export default function Map() {
  const [nodes, setNodes] = useState<OsmNode[]>([]);
  const [ways, setWays] = useState<OsmWay[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await fetch("/api/osm", { cache: "no-store" });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data: { elements?: OsmElement[] } = await res.json();
        const elems = data.elements ?? [];
        const nodeElems = elems.filter(
          (e): e is OsmNode => e.type === "node" && typeof (e as any).lat === "number" && typeof (e as any).lon === "number"
        );
        const wayElems = elems.filter(
          (e): e is OsmWay => e.type === "way" && Array.isArray((e as any).geometry)
        );
        if (!cancelled) {
          setNodes(nodeElems);
          setWays(wayElems);
        }
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div style={{ height: "100vh", width: "100%" }}>
      <MapContainer center={center} zoom={12} style={{ height: "100%", width: "100%" }} scrollWheelZoom>
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        {/* Ways (as Polyline/Polygon) */}
        {ways.map((w) => {
          const positions = w.geometry.map((p) => [p.lat, p.lon] as [number, number]);
          const first = positions[0];
          const last = positions[positions.length - 1];
          const isClosed =
            positions.length >= 3 &&
            first &&
            last &&
            Math.abs(first[0] - last[0]) < 1e-9 &&
            Math.abs(first[1] - last[1]) < 1e-9;

          const popup = (
            <Popup>
              <div style={{ fontSize: 12, lineHeight: 1.4 }}>
                <div><b>ID:</b> {w.id}</div>
                {w.tags && (
                  <div>
                    {Object.entries(w.tags).map(([k, v]) => (
                      <div key={k}>
                        <b>{k}:</b> {String(v)}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </Popup>
          );

          return isClosed ? (
            <Polygon key={w.id} positions={positions as unknown as LatLngExpression[]} pathOptions={{ color: "#e85d04", weight: 3, fillOpacity: 0.2 }}>
              {popup}
            </Polygon>
          ) : (
            <Polyline key={w.id} positions={positions as unknown as LatLngExpression[]} pathOptions={{ color: "#1d4ed8", weight: 4 }}>
              {popup}
            </Polyline>
          );
        })}

        {/* Nodes (as markers) */}
        {nodes.map((n) => (
          <Marker key={n.id} position={[n.lat!, n.lon!] as LatLngExpression}>
            <Popup>
              <div style={{ fontSize: 12, lineHeight: 1.4 }}>
                <div><b>ID:</b> {n.id}</div>
                {n.tags && (
                  <div>
                    {Object.entries(n.tags).map(([k, v]) => (
                      <div key={k}>
                        <b>{k}:</b> {String(v)}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </Popup>
          </Marker>
        ))}
      </MapContainer>
      {error && (
        <div style={{ position: "absolute", top: 8, left: 8, background: "#fff", padding: 8, borderRadius: 4, boxShadow: "0 1px 4px rgba(0,0,0,0.2)" }}>
          Failed to load data: {error}
        </div>
      )}
    </div>
  );
}
