"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AlertCircle, MapPin, AlertTriangle, Activity, Loader2 } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui/alert";

type Hazard = {
  id: string;
  lat: number;
  lon: number;
  type: string;
  severity: "low" | "medium" | "high";
  description: string;
};

export default function Home() {
  const [latitude, setLatitude] = useState<string | null>(null);
  const [longitude, setLongitude] = useState<string | null>(null);
  const [hazards, setHazards] = useState<Hazard[]>([]);
  const [allTelemetry, setAllTelemetry] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    const fetchTelemetry = async () => {
      try {
        setLoading(true);

        const latRes = await fetch("/telemetry/latitude", { cache: "no-store" });
        if (latRes.ok) {
          const latText = await latRes.text();
          if (!cancelled) setLatitude(latText === "null" ? null : latText);
        }

        const lonRes = await fetch("/telemetry/longitude", { cache: "no-store" });
        if (lonRes.ok) {
          const lonText = await lonRes.text();
          if (!cancelled) setLongitude(lonText === "null" ? null : lonText);
        }

        const hazardsRes = await fetch("/telemetry/hazards", { cache: "no-store" });
        if (hazardsRes.ok) {
          const hazardsText = await hazardsRes.text();
          if (hazardsText !== "null") {
            try {
              const parsed = JSON.parse(hazardsText);
              if (!cancelled) setHazards(Array.isArray(parsed) ? parsed : []);
            } catch {
              if (!cancelled) setHazards([]);
            }
          }
        }

        const allRes = await fetch("/telemetry", { cache: "no-store" });
        if (allRes.ok) {
          const allData = await allRes.json();
          const obj: Record<string, string> = {};
          allData.forEach((item: { key: string; value: string }) => {
            obj[item.key] = item.value;
          });
          if (!cancelled) setAllTelemetry(obj);
        }
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    fetchTelemetry();
    const interval = setInterval(fetchTelemetry, 1000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case "high":
        return "border-red-500 bg-red-50";
      case "medium":
        return "border-amber-500 bg-amber-50";
      case "low":
        return "border-green-500 bg-green-50";
      default:
        return "border-gray-200 bg-white";
    }
  };

  const getSeverityBadgeColor = (severity: string) => {
    switch (severity) {
      case "high":
        return "bg-red-100 text-red-800";
      case "medium":
        return "bg-amber-100 text-amber-800";
      case "low":
        return "bg-green-100 text-green-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 p-6">
      <div className="max-w-6xl mx-auto space-y-6">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-4xl font-bold text-slate-900 mb-2">SafeWalk Telemetry</h1>
          <p className="text-lg text-slate-600">Real-time Monitoring</p>
        </div>

        {/* Error Alert */}
        {error && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Loading State */}
        {loading && (
          <Alert>
            <Loader2 className="h-4 w-4 animate-spin" />
            <AlertDescription>Loading telemetry data...</AlertDescription>
          </Alert>
        )}

        {/* Location Card */}
        <Card className="border-2 border-blue-200 shadow-lg">
          <CardHeader className="bg-gradient-to-r from-blue-50 to-blue-100">
            <div className="flex items-center gap-2">
              <MapPin className="h-5 w-5 text-blue-600" />
              <CardTitle className="text-blue-900">Current Location</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="pt-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <p className="text-sm font-medium text-slate-600 mb-2">Latitude</p>
                <p className="text-3xl font-mono font-bold text-slate-900">
                  {latitude || "—"}
                </p>
              </div>
              <div>
                <p className="text-sm font-medium text-slate-600 mb-2">Longitude</p>
                <p className="text-3xl font-mono font-bold text-slate-900">
                  {longitude || "—"}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Hazards Card */}
        <Card className="shadow-lg">
          <CardHeader className="bg-gradient-to-r from-orange-50 to-red-100">
            <div className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-red-600" />
              <CardTitle className="text-red-900">Nearby Hazards</CardTitle>
              <span className="ml-auto inline-flex items-center justify-center h-6 w-6 rounded-full bg-red-200 text-red-800 text-sm font-bold">
                {hazards.length}
              </span>
            </div>
          </CardHeader>
          <CardContent className="pt-6">
            {hazards.length === 0 ? (
              <p className="text-slate-600">No hazards detected</p>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {hazards.map((hazard) => (
                  <Card key={hazard.id} className={`border-2 ${getSeverityColor(hazard.severity)}`}>
                    <CardContent className="pt-4">
                      <div className="flex justify-between items-start mb-3">
                        <h3 className="font-bold text-slate-900 text-sm">{hazard.type}</h3>
                        <span className={`text-xs font-semibold px-2 py-1 rounded ${getSeverityBadgeColor(hazard.severity)}`}>
                          {hazard.severity.toUpperCase()}
                        </span>
                      </div>
                      <p className="text-sm text-slate-700 mb-3">{hazard.description}</p>
                      <p className="text-xs text-slate-500 font-mono">
                        {hazard.lat.toFixed(6)}, {hazard.lon.toFixed(6)}
                      </p>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* All Telemetry */}
        <Card className="shadow-lg">
          <CardHeader className="bg-gradient-to-r from-purple-50 to-indigo-100">
            <div className="flex items-center gap-2">
              <Activity className="h-5 w-5 text-purple-600" />
              <CardTitle className="text-purple-900">All Telemetry Data</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="pt-6">
            {Object.keys(allTelemetry).length === 0 ? (
              <p className="text-slate-600">No telemetry data available</p>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {Object.entries(allTelemetry).map(([key, value]) => (
                  <div key={key} className="bg-slate-50 border border-slate-200 rounded-lg p-4">
                    <p className="text-xs font-bold text-slate-600 uppercase tracking-wide mb-2">
                      {key}
                    </p>
                    <p className="text-sm font-mono text-slate-900 break-words max-h-20 overflow-y-auto">
                      {value}
                    </p>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
