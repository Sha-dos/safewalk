"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {AlertCircle, MapPin, AlertTriangle, Activity, Loader2, Vibrate, Heart} from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui/alert";

interface SystemStats {
  cpu_usage: number;
  available_memory: number;
}

export default function Home() {
  const [latitude, setLatitude] = useState<string | null>(null);
  const [longitude, setLongitude] = useState<string | null>(null);
  const [heading, setHeading] = useState<string | null>(null);
  const [speeds, setSpeeds] = useState<string[] | null>(null);
  const [hazards, setHazards] = useState<string[]>([]);
  const [allTelemetry, setAllTelemetry] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [systemStatus, setSystemStatus] = useState<SystemStats | null>(null);

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

        const headingRes = await fetch("/telemetry/heading", { cache: "no-store" });
        if (headingRes.ok) {
          const headingText = await headingRes.text();
          if (!cancelled) setHeading(headingText === "null" ? null : headingText);
        }

        const speedsRes = await fetch("/telemetry/speeds", { cache: "no-store" });
        if (speedsRes.ok) {
          const speedsText = await speedsRes.json();
          if (!cancelled) setSpeeds(speedsText === "null" ? null : speedsText);
        }

        const statsRes = await fetch("/health", { cache: "no-store" });
        if (statsRes.ok) {
          const healthJson = await statsRes.json();
          if (!cancelled) setSystemStatus(healthJson === "null" ? null : healthJson);
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
    const interval = setInterval(fetchTelemetry, 100);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 p-6">
      <div className="max-w-6xl mx-auto space-y-6">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-4xl font-bold text-slate-900 mb-2">SafeWalk Telemetry</h1>
        </div>

        {/* Error Alert */}
        {error && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Location Card */}
        <Card className="shadow-lg">
          <CardHeader>
            <div className="flex items-center gap-2">
              <MapPin className="h-5 w-5 text-blue-600" />
              <CardTitle className="text-blue-900">Current Location</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="pt-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
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
              <div>
                <p className="text-sm font-medium text-slate-600 mb-2">Heading</p>
                <p className="text-3xl font-mono font-bold text-slate-900">
                  {heading || "—"}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Hazards Card */}
        <Card className="shadow-lg">
          <CardHeader>
            <div className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-red-600" />
              <CardTitle className="text-red-900">Nearby Hazards</CardTitle>
              <span className="ml-auto inline-flex items-center justify-center h-6 w-6 rounded-full bg-red-200 text-red-800 text-sm font-bold">
                {hazards.length}
              </span>
            </div>
          </CardHeader>
          <CardContent className="pt-6">
          </CardContent>
        </Card>

        {/* Motor Card */}
        <Card className="shadow-lg">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Vibrate className="h-5 w-5 text-green-700" />
              <CardTitle className="text-green-700">Motor Speeds</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="pt-6">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
              <div>
                <p className="text-sm font-medium text-slate-600 mb-2">Front</p>
                <p className="text-3xl font-mono font-bold text-slate-900">
                  {speeds && speeds[0]}
                </p>
              </div>
              <div>
                <p className="text-sm font-medium text-slate-600 mb-2">Right</p>
                <p className="text-3xl font-mono font-bold text-slate-900">
                  {speeds && speeds[1]}
                </p>
              </div>
              <div>
                <p className="text-sm font-medium text-slate-600 mb-2">Back</p>
                <p className="text-3xl font-mono font-bold text-slate-900">
                  {speeds && speeds[2]}
                </p>
              </div>
              <div>
                <p className="text-sm font-medium text-slate-600 mb-2">Left</p>
                <p className="text-3xl font-mono font-bold text-slate-900">
                  {speeds && speeds[3]}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* All Telemetry */}
        <Card className="shadow-lg">
          <CardHeader>
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

        {/* Health Card */}
        <Card className="shadow-lg">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Heart className="h-5 w-5 text-red-600" />
              <CardTitle className="text-red-600">System Health</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="pt-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <p className="text-sm font-medium text-slate-600 mb-2">CPU Usage</p>
                <p className="text-3xl font-mono font-bold text-slate-900">
                  {systemStatus?.cpu_usage.toFixed(2) || "—"}
                </p>
              </div>
              <div>
                <p className="text-sm font-medium text-slate-600 mb-2">Available Memory</p>
                <p className="text-3xl font-mono font-bold text-slate-900">
                  {systemStatus?.available_memory || "—"}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
