"use client";

import { MapContainer, TileLayer, Marker, Popup } from "react-leaflet";
import "leaflet-defaulticon-compatibility";
import type { LatLngExpression } from "leaflet";
import { useEffect, useState } from "react";

const center: LatLngExpression = [33.423322, -111.932648];

type OsmElement = {
  type: string;
  id: number;
  lat?: number;
  lon?: number;
  tags?: Record<string, string>;
};

type OsmResponse = {
  elements?: OsmElement[];
};

export default function Map() {
  const [nodes, setNodes] = useState<OsmElement[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await fetch("/api/osm", { cache: "no-store" });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data: OsmResponse = await res.json();
        const elems = (data.elements ?? []).filter(
          (e) => e.type === "node" && typeof e.lat === "number" && typeof e.lon === "number"
        );
        if (!cancelled) setNodes(elems);
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
