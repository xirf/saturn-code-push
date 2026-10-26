import React, { useEffect, useMemo, useState } from "react";

import { createRoot } from "react-dom/client";

import {
  Theme,
  Button,
  Badge,
  Select,
  TextField,
  Dialog,
  DropdownMenu,
  Callout,
  Spinner,
  Tooltip,
  IconButton,
} from "@radix-ui/themes";

import {
  ArrowUpRight,
  ArrowRight,
  ArrowDownToLine,
  Check,
  ChevronDown,
  ChevronRight,
  CircleHelp,
  Clock3,
  Code2,
  Copy,
  Download,
  ExternalLink,
  GitBranch,
  Layers3,
  LayoutDashboard,
  LogOut,
  Menu,
  MoreHorizontal,
  Plus,
  RefreshCw,
  Rocket,
  Search,
  Settings,
  ShieldCheck,
  Smartphone,
  Sparkles,
  Terminal,
  TriangleAlert,
  Users,
  X,
  Activity,
  Box,
  CheckCircle2,
  LockKeyhole,
} from "lucide-react";

import {
  ResponsiveContainer,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as ChartTooltip,
} from "recharts";

import "@fontsource-variable/dm-sans";
import "@fontsource-variable/manrope";
import "@radix-ui/themes/styles.css";

import "./styles.css";

type Application = { id: string; display_name: string; created_at: string };

type Release = {
  id: number;
  app_id: string;
  version: string;
  flutter_revision: string;
  platforms: Record<string, { status: string }>;
  created_at: string;
};

type Patch = {
  id: number;
  release_id: number;
  number: number;
  channel_ids: number[];
  is_rolled_back: boolean;
  created_at: string;
};

type Channel = { id: number; app_id: string; name: string };

type Artifact = {
  id: number;
  app_id: string;
  release_id: number;
  patch_id: number | null;
  arch: string;
  platform: string;
  uploaded: boolean;
  size: number;
};

type AssetBundleRow = {
  id: number;
  app_id: string;
  patch_id: number;
  platform: string;
  uploaded: boolean;
  size: number;
};

type Report = {
  received_at: string;
  event: {
    app_id: string;
    type: string;
    release_version?: string;
    patch_number?: number;
    client_id?: string;
    platform?: string;
    arch?: string;
    message?: string;
  };
};

type Catalog = {
  apps: Application[];
  releases: Release[];
  patches: Patch[];
  channels: Channel[];
  artifacts: Artifact[];
  asset_bundles?: AssetBundleRow[];
  events: Report[];
};

type User = { username: string };

type Page =
  | "overview"
  | "applications"
  | "releases"
  | "patches"
  | "analytics"
  | "settings";

const nav = [
  { id: "overview", label: "Overview", icon: LayoutDashboard },
  { id: "applications", label: "Applications", icon: Box },
  { id: "releases", label: "Releases", icon: Layers3 },
  { id: "patches", label: "Patches", icon: GitBranch },
  { id: "analytics", label: "Analytics", icon: Activity },
] as const;

const preview =
  import.meta.env.DEV && new URLSearchParams(location.search).has("preview");

class ApiError extends Error {
  status: number;
  constructor(message: string, status: number) {
    super(message);
    this.status = status;
  }
}

async function api<T>(path: string, body?: unknown): Promise<T> {
  const r = await fetch(path, {
    method: body === undefined ? "GET" : "POST",
    credentials: "same-origin",
    headers: { "Content-Type": "application/json" },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
  });

  const data = await r.json().catch(() => ({}));
  if (!r.ok)
    throw new ApiError(
      data.message || `Request failed (${r.status})`,
      r.status,
    );
  return data;
}

function Orbit({ size = 30 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      aria-hidden="true"
    >
      <circle cx="24" cy="24" r="10" fill="currentColor" />
      <ellipse
        cx="24"
        cy="24"
        rx="22"
        ry="7"
        transform="rotate(-30 24 24)"
        stroke="currentColor"
        strokeWidth="2.8"
      />
    </svg>
  );
}

const short = (s: string) => s.slice(0, 8);

const date = (s: string) =>
  new Date(s).toLocaleDateString(undefined, { month: "short", day: "numeric" });

const bytes = (n: number) =>
  n >= 1048576
    ? `${(n / 1048576).toFixed(1)} MB`
    : `${Math.round(n / 1024)} KB`;

function ago(s: string) {
  const n = Math.max(0, Math.floor((Date.now() - Date.parse(s)) / 60000));
  return n < 1
    ? "Just now"
    : n < 60
      ? `${n}m ago`
      : n < 1440
        ? `${Math.floor(n / 60)}h ago`
        : `${Math.floor(n / 1440)}d ago`;
}

function reportName(e: Report) {
  return (
    (
      {
        __patch_download__: "Patch downloaded",
        __patch_install__: "Patch launched",
        __patch_install_failure__: "Launch failed",
        __patch_update_failure__: "Update failed",
      } as Record<string, string>
    )[e.event.type] || "Device report"
  );
}

function SignIn({ onLogin }: { onLogin: (user: User) => void }) {
  const [username, setUsername] = useState(""),
    [password, setPassword] = useState(""),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      onLogin(await api<User>("/auth/login", { username, password }));
      setPassword("");
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="login-layout">
      <section className="login-story">
        <a className="brand" href="/">
          <span className="brand-symbol">
            <Orbit />
          </span>
          saturn<span className="brand-tag">CODE PUSH</span>
        </a>
        <div className="story-content">
          <Badge color="gray" variant="surface">
            <span className="status-dot" /> Your infrastructure. Your updates.
          </Badge>
          <h1>
            Your releases.
            <br />
            <span>In orbit.</span>
          </h1>
          <p>
            A quieter way to ship. Deliver signed patches,
            <br className="desktop-break" /> keep releases under control, and
            see what lands.
          </p>
          <div className="orbit-art" aria-hidden="true">
            <div className="orbit-grid" />
            <div className="orbital o1" />
            <div className="orbital o2" />
            <div className="planet">
              <Orbit size={116} />
            </div>
            <span className="satellite s1" />
            <span className="satellite s2" />
            <div className="floating-label">
              <ShieldCheck size={15} /> Signed. Delivered. Verified.
              <CheckCircle2 size={14} />
            </div>
          </div>
          <div className="story-features">
            <span>
              <ShieldCheck size={16} /> Signed patches
            </span>
            <span>
              <GitBranch size={16} /> Controlled rollouts
            </span>
            <span>
              <Activity size={16} /> Device insights
            </span>
          </div>
        </div>
        <div className="story-footer">
          An independent home for your updates.<span>Saturn Code Push</span>
        </div>
      </section>
      <section className="login-form-side">
        <div className="login-card">
          <div className="login-emblem">
            <LockKeyhole size={23} />
          </div>
          <h2>Welcome back</h2>
          <p>Sign in to your release console.</p>
          <form onSubmit={submit}>
            <label htmlFor="username">Username</label>
            <TextField.Root
              id="username"
              size="3"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              autoComplete="username"
              autoFocus
              required
              placeholder="Your username"
            />
            <div className="password-label">
              <label htmlFor="password">Password</label>
              <span>Administrator access</span>
            </div>
            <TextField.Root
              id="password"
              size="3"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="current-password"
              required
              placeholder="Enter your password"
            />
            {error && (
              <Callout.Root color="red" size="1" role="alert">
                <Callout.Icon>
                  <TriangleAlert size={16} />
                </Callout.Icon>
                <Callout.Text>{error}</Callout.Text>
              </Callout.Root>
            )}
            <Button
              type="submit"
              size="3"
              className="login-submit"
              loading={busy}
            >
              Sign in <ArrowRight size={17} />
            </Button>
          </form>
          <div className="secure-note">
            <ShieldCheck size={14} />
            <span>Protected by your self-hosted Saturn server.</span>
          </div>
          <p className="login-help">
            Need access? Contact your workspace administrator.
          </p>
        </div>
        <div className="login-bottom">
          <span className="status-dot" /> Release control, on your terms.
        </div>
      </section>
    </div>
  );
}

function Empty({
  icon: Icon = Layers3,
  title,
  description,
  action,
}: {
  icon?: typeof Layers3;
  title: string;
  description: string;
  action?: React.ReactNode;
}) {
  return (
    <div className="empty">
      <div className="empty-icon">
        <Icon size={24} />
      </div>
      <h3>{title}</h3>
      <p>{description}</p>
      {action}
    </div>
  );
}

function Console() {
  const [user, setUser] = useState<User | null>(
      preview ? { username: "alex" } : null,
    ),
    [checking, setChecking] = useState(!preview),
    [data, setData] = useState<Catalog | null>(null),
    [page, setPage] = useState<Page>("overview"),
    [appId, setAppId] = useState(""),
    [busy, setBusy] = useState(false),
    [error, setError] = useState(""),
    [notice, setNotice] = useState(""),
    [mobile, setMobile] = useState(false),
    [search, setSearch] = useState(""),
    [days, setDays] = useState("7"),
    [synced, setSynced] = useState<Date | null>(null);

  const [createOpen, setCreateOpen] = useState(false),
    [appName, setAppName] = useState(""),
    [saving, setSaving] = useState(false),
    [action, setAction] = useState<{
      kind: "promote" | "rollback" | "restore";
      patch: Patch;
    } | null>(null),
    [channelId, setChannelId] = useState("");

  useEffect(() => {
    sessionStorage.removeItem("saturnToken");
    if (preview) {
      import("./preview").then((m) => {
        setData(m.previewData);
        setAppId(m.previewData.apps[0].id);
        setSynced(new Date());
      });
      return;
    }
    api<User>("/auth/session")
      .then(setUser)
      .catch(() => setUser(null))
      .finally(() => setChecking(false));
  }, []);

  async function refresh() {
    if (preview) {
      setSynced(new Date());
      return;
    }
    setBusy(true);
    try {
      const state = await api<Catalog>("/admin/state");
      setData(state);
      setAppId((id) =>
        state.apps.some((a) => a.id === id) ? id : state.apps[0]?.id || "",
      );
      setSynced(new Date());
      setError("");
    } catch (e) {
      if ((e as ApiError).status === 401) {
        setUser(null);
        setData(null);
      } else setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    if (user && !preview) void refresh();
  }, [user]);

  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(""), 5000);
    return () => clearTimeout(timer);
  }, [notice]);

  const app = data?.apps.find((a) => a.id === appId),
    releases =
      data?.releases
        .filter((r) => r.app_id === appId)
        .slice()
        .reverse() || [],
    releaseIds = new Set(releases.map((r) => r.id)),
    patches =
      data?.patches
        .filter((p) => releaseIds.has(p.release_id))
        .slice()
        .reverse() || [],
    channels = data?.channels.filter((c) => c.app_id === appId) || [],
    artifacts = data?.artifacts.filter((a) => a.app_id === appId) || [];

  const periodStart = new Date(Date.now() - (Number(days) - 1) * 86400000)
    .toISOString()
    .slice(0, 10);
  const events = (data?.events || []).filter((e) => e.event.app_id === appId),
    period = events.filter((e) => e.received_at.slice(0, 10) >= periodStart),
    downloads = period.filter(
      (e) => e.event.type === "__patch_download__",
    ).length,
    installs = period.filter(
      (e) => e.event.type === "__patch_install__",
    ).length,
    failures = period.filter((e) => e.event.type.includes("failure")).length,
    reporting = new Set(period.map((e) => e.event.client_id).filter(Boolean))
      .size;

  const chart = useMemo(
    () =>
      Array.from({ length: Number(days) }, (_, i) => {
        const day = new Date();
        day.setUTCDate(day.getUTCDate() - (Number(days) - 1 - i));
        const key = day.toISOString().slice(0, 10);
        const rows = events.filter((e) => e.received_at.slice(0, 10) === key);
        return {
          date: day.toLocaleDateString(undefined, {
            month: "short",
            day: "numeric",
            timeZone: "UTC",
          }),
          downloads: rows.filter((e) => e.event.type === "__patch_download__")
            .length,
          launches: rows.filter((e) => e.event.type === "__patch_install__")
            .length,
        };
      }),
    [data, appId, days],
  );

  function navigate(p: Page) {
    setPage(p);
    setSearch("");
    setMobile(false);
  }

  async function copy(value: string, label = "Copied to clipboard") {
    try {
      await navigator.clipboard.writeText(value);
      setNotice(label);
    } catch {
      setError(
        "Clipboard access is unavailable. Select and copy the value manually.",
      );
    }
  }

  async function signout() {
    if (preview) {
      setUser(null);
      setData(null);
      return;
    }
    try {
      await api("/auth/logout", {});
      setUser(null);
      setData(null);
      setAppId("");
    } catch (e) {
      setError((e as Error).message);
    }
  }

  async function createApp(e: React.FormEvent) {
    e.preventDefault();
    if (preview) {
      setError("Design preview does not change server data.");
      return;
    }
    setSaving(true);
    try {
      const result = await api<Application>("/api/v1/apps", {
        display_name: appName,
        organization_id: 1,
      });
      await refresh();
      setAppId(result.id);
      setAppName("");
      setCreateOpen(false);
      setNotice("Application created. Ready for its first release.");
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSaving(false);
    }
  }

  function chooseAction(
    kind: "promote" | "rollback" | "restore",
    patch: Patch,
  ) {
    setChannelId(
      String(
        channels.find((c) => c.name === "stable")?.id || channels[0]?.id || "",
      ),
    );
    setAction({ kind, patch });
  }

  async function executeAction() {
    if (!action) return;
    if (preview) {
      setNotice("This is a design preview. No server data was changed.");
      setAction(null);
      return;
    }
    setSaving(true);
    try {
      if (action.kind === "promote")
        await api(`/api/v1/apps/${appId}/patches/promote`, {
          patch_id: action.patch.id,
          channel_id: Number(channelId),
        });
      else
        await api(
          `/api/v1/apps/${appId}/releases/${action.patch.release_id}/patches/${action.patch.id}/${action.kind === "rollback" ? "rollback" : "rollforward"}`,
          {},
        );
      setNotice(
        action.kind === "promote"
          ? "Patch promoted to the selected channel."
          : action.kind === "rollback"
            ? "Patch rolled back. Devices will act on their next check."
            : "Patch restored.",
      );
      setAction(null);
      await refresh();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSaving(false);
    }
  }

  const matchedReleases = releases.filter((r) =>
      `${r.version} ${r.flutter_revision}`
        .toLowerCase()
        .includes(search.toLowerCase()),
    ),
    matchedPatches = patches.filter((p) =>
      `${p.number} ${releases.find((r) => r.id === p.release_id)?.version} ${p.is_rolled_back ? "rolled back" : "active"}`.includes(
        search.toLowerCase(),
      ),
    );

  function releaseTable(rows: Release[]) {
    return rows.length ? (
      <div className="table-scroll">
        <table>
          <thead>
            <tr>
              <th>Release</th>
              <th>Status</th>
              <th>Platform</th>
              <th>Build</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id}>
                <td>
                  <div className="table-title">
                    <span className="row-icon">
                      <Layers3 size={16} />
                    </span>
                    <div>
                      <strong>{r.version}</strong>
                      <span className="row-sub">
                        {
                          artifacts.filter(
                            (a) =>
                              a.release_id === r.id &&
                              !a.patch_id &&
                              a.uploaded,
                          ).length
                        }{" "}
                        artifacts
                      </span>
                    </div>
                  </div>
                </td>
                <td>
                  <Badge
                    color={
                      Object.values(r.platforms).some(
                        (p) => p.status === "active",
                      )
                        ? "green"
                        : "gray"
                    }
                    variant="soft"
                  >
                    <span className="status-dot" />
                    {Object.values(r.platforms).some(
                      (p) => p.status === "active",
                    )
                      ? "Active"
                      : "Draft"}
                  </Badge>
                </td>
                <td>
                  <span className="platform">
                    <Smartphone size={14} />
                    {Object.keys(r.platforms).join(", ") || "Pending"}
                  </span>
                </td>
                <td>
                  <code>{short(r.flutter_revision)}</code>
                </td>
                <td className="muted">{date(r.created_at)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    ) : (
      <Empty
        title="Your first release starts here"
        description="Create a release with the CLI to establish a baseline for future patches."
      />
    );
  }

  function patchTable(rows: Patch[]) {
    return rows.length ? (
      <div className="table-scroll">
        <table>
          <thead>
            <tr>
              <th>Patch</th>
              <th>Release</th>
              <th>Channels</th>
              <th>Artifacts</th>
              <th>Assets</th>
              <th>Status</th>
              <th className="align-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((p) => (
              <tr key={p.id}>
                <td>
                  <div className="table-title">
                    <span className="row-icon">
                      <GitBranch size={16} />
                    </span>
                    <strong>Patch #{p.number}</strong>
                  </div>
                </td>
                <td>
                  <code>
                    {releases.find((r) => r.id === p.release_id)?.version}
                  </code>
                </td>
                <td>
                  <div className="badge-group">
                    {p.channel_ids.map((id) => (
                      <Badge
                        key={id}
                        variant="outline"
                        color={
                          channels.find((c) => c.id === id)?.name === "stable"
                            ? "green"
                            : "gray"
                        }
                      >
                        {channels.find((c) => c.id === id)?.name || "Unknown"}
                      </Badge>
                    ))}
                    {!p.channel_ids.length && (
                      <span className="muted">Unpublished</span>
                    )}
                  </div>
                </td>
                <td className="muted">
                  {artifacts
                    .filter((a) => a.patch_id === p.id)
                    .map((a) => `${a.arch} · ${bytes(a.size)}`)
                    .join(", ") || "No artifacts"}
                </td>
                <td className="muted">
                  {data?.asset_bundles
                    ?.filter((bundle) => bundle.patch_id === p.id && bundle.uploaded)
                    .map((bundle) => `${bundle.platform} · ${bytes(bundle.size)}`)
                    .join(", ") || "No asset bundle"}
                </td>
                <td>
                  <Badge
                    color={p.is_rolled_back ? "amber" : "green"}
                    variant="soft"
                  >
                    {p.is_rolled_back ? "Rolled back" : "Available"}
                  </Badge>
                </td>
                <td>
                  <div className="row-actions">
                    <Button
                      size="1"
                      variant="soft"
                      color={p.is_rolled_back ? "gray" : undefined}
                      disabled={p.is_rolled_back}
                      onClick={() => chooseAction("promote", p)}
                    >
                      Promote
                      <ArrowUpRight size={13} />
                    </Button>
                    <DropdownMenu.Root>
                      <DropdownMenu.Trigger>
                        <IconButton
                          variant="ghost"
                          color="gray"
                          aria-label={`Actions for patch ${p.number}`}
                        >
                          <MoreHorizontal size={17} />
                        </IconButton>
                      </DropdownMenu.Trigger>
                      <DropdownMenu.Content>
                        <DropdownMenu.Item
                          color={p.is_rolled_back ? "green" : "red"}
                          onSelect={() =>
                            chooseAction(
                              p.is_rolled_back ? "restore" : "rollback",
                              p,
                            )
                          }
                        >
                          {p.is_rolled_back
                            ? "Restore patch"
                            : "Roll back patch"}
                        </DropdownMenu.Item>
                      </DropdownMenu.Content>
                    </DropdownMenu.Root>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    ) : (
      <Empty
        icon={GitBranch}
        title="No patches yet"
        description="Publish a signed patch to deliver a fix to this application's release."
      />
    );
  }

  const stats = (
    <div className="stats-grid">
      {[
        {
          label: "Patch downloads",
          value: downloads,
          icon: ArrowDownToLine,
          caption: "Download reports",
        },
        {
          label: "Successful launches",
          value: installs,
          icon: Rocket,
          caption: "First-launch reports",
        },
        {
          label: "Reporting installs",
          value: reporting,
          icon: Users,
          caption: "Distinct reporting install IDs",
        },
        {
          label: "Failure reports",
          value: failures,
          icon: TriangleAlert,
          caption: "Update and launch failures",
        },
      ].map((s) => (
        <section className="stat-card" key={s.label}>
          <div>
            <span>{s.label}</span>
            <s.icon size={17} />
          </div>
          <strong>{s.value.toLocaleString()}</strong>
          <small>
            <span
              className={`tiny-dot ${s.label === "Failure reports" && s.value ? "amber" : ""}`}
            />
            {s.caption}
          </small>
        </section>
      ))}
    </div>
  );

  const activityChart = (
    <section className="panel chart-panel">
      <div className="panel-heading">
        <div>
          <h2>Patch activity</h2>
          <p>Downloads and successful first launches, by UTC day.</p>
        </div>
        <div className="chart-legend">
          <span>
            <i />
            Downloads
          </span>
          <span>
            <i className="blue" />
            Launches
          </span>
        </div>
      </div>
      <div className="chart-wrap">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart
            data={chart}
            margin={{ top: 16, right: 8, left: -20, bottom: 0 }}
          >
            <defs>
              <linearGradient id="downloads-fill" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#ededed" stopOpacity={0.2} />
                <stop offset="100%" stopColor="#ededed" stopOpacity={0} />
              </linearGradient>
              <linearGradient id="launches-fill" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#6d9dea" stopOpacity={0.08} />
                <stop offset="100%" stopColor="#6d9dea" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid
              stroke="#262626"
              vertical={false}
              strokeDasharray="3 5"
            />
            <XAxis
              dataKey="date"
              tickLine={false}
              axisLine={false}
              tick={{ fill: "#858585", fontSize: 11 }}
              minTickGap={34}
              dy={10}
            />
            <YAxis
              allowDecimals={false}
              tickLine={false}
              axisLine={false}
              tick={{ fill: "#858585", fontSize: 11 }}
            />
            <ChartTooltip
              contentStyle={{
                background: "#171717",
                border: "1px solid #333333",
                borderRadius: 10,
                color: "#ededed",
                fontSize: 12,
              }}
            />
            <Area
              name="Downloads"
              type="monotone"
              dataKey="downloads"
              stroke="#ededed"
              strokeWidth={2.2}
              fill="url(#downloads-fill)"
            />
            <Area
              name="Launches"
              type="monotone"
              dataKey="launches"
              stroke="#6d9dea"
              strokeWidth={2}
              fill="url(#launches-fill)"
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
      <div className="chart-foot">
        <span>
          <Activity size={13} /> Retained client reports
        </span>
        <span>Last {days} days · UTC</span>
      </div>
    </section>
  );

  if (checking)
    return (
      <div className="loading-screen">
        <Orbit size={38} />
        <Spinner size="3" />
        <span>Connecting to Saturn…</span>
      </div>
    );

  if (!user) return <SignIn onLogin={setUser} />;

  return (
    <div className="console-shell">
      {mobile && (
        <button
          className="sidebar-scrim"
          aria-label="Close navigation"
          onClick={() => setMobile(false)}
        />
      )}
      <aside className={`sidebar ${mobile ? "is-open" : ""}`}>
        <a
          className="brand"
          href="#"
          onClick={(e) => {
            e.preventDefault();
            navigate("overview");
          }}
        >
          <span className="brand-symbol">
            <Orbit size={28} />
          </span>
          saturn
          <Badge color="gray" size="1" variant="outline">
            CONSOLE
          </Badge>
        </a>
        <div className="workspace-switch">
          <div className="workspace-avatar">S</div>
          <div>
            <strong>Saturn workspace</strong>
            <small>Self-hosted</small>
          </div>
          <ShieldCheck size={15} />
        </div>
        <div className="nav-label">WORKSPACE</div>
        <nav>
          {nav.map((item) => (
            <button
              key={item.id}
              className={page === item.id ? "active" : ""}
              onClick={() => navigate(item.id)}
            >
              <item.icon size={18} />
              {item.label}
              {item.id === "applications" && (
                <span className="nav-count">{data?.apps.length || 0}</span>
              )}
              {page === item.id && <span className="nav-active-dot" />}
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <div className="hosting-card">
            <span className="hosting-icon">
              <ShieldCheck size={19} />
            </span>
            <strong>Your updates, your server.</strong>
            <p>Signed patches. Independent infrastructure.</p>
            <a
              href="https://docs.shorebird.dev/"
              target="_blank"
              rel="noreferrer"
            >
              CLI documentation
              <ArrowUpRight size={13} />
            </a>
          </div>
          <button
            className={`settings-nav ${page === "settings" ? "active" : ""}`}
            onClick={() => navigate("settings")}
          >
            <Settings size={18} />
            Settings
          </button>
          <DropdownMenu.Root>
            <DropdownMenu.Trigger>
              <button className="account">
                <div className="account-avatar">
                  {user.username.slice(0, 1).toUpperCase()}
                </div>
                <span>
                  <strong>{user.username}</strong>
                  <small>Administrator</small>
                </span>
                <ChevronDown size={15} />
              </button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content>
              <DropdownMenu.Label>
                Signed in as {user.username}
              </DropdownMenu.Label>
              <DropdownMenu.Item onSelect={() => navigate("settings")}>
                Workspace settings
              </DropdownMenu.Item>
              <DropdownMenu.Separator />
              <DropdownMenu.Item color="red" onSelect={() => void signout()}>
                <LogOut size={14} />
                Sign out
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        </div>
      </aside>
      <div className="main-shell">
        <header className="topbar">
          <div className="breadcrumb">
            <IconButton
              className="mobile-menu"
              variant="ghost"
              color="gray"
              aria-label="Open navigation"
              onClick={() => setMobile(true)}
            >
              <Menu size={20} />
            </IconButton>
            <span>Workspace</span>
            <ChevronRight size={13} />
            <strong>
              {nav.find((n) => n.id === page)?.label || "Settings"}
            </strong>
          </div>
          <div className="topbar-right">
            <span className="connection">
              <span className="status-dot" />
              {preview ? "Design preview" : synced ? "Connected" : "Connecting"}
            </span>
            <span className="top-divider" />
            <Tooltip content="Refresh server data">
              <IconButton
                variant="ghost"
                color="gray"
                onClick={() => void refresh()}
                aria-label="Refresh server data"
                disabled={busy}
              >
                <RefreshCw size={16} className={busy ? "spin" : ""} />
              </IconButton>
            </Tooltip>
          </div>
        </header>
        <main className="content">
          {preview && (
            <div className="preview-banner">
              <Sparkles size={14} /> Design preview — illustrative data, no
              server changes.
            </div>
          )}
          {error && (
            <Callout.Root color="red" className="error-banner" role="alert">
              <Callout.Icon>
                <TriangleAlert size={17} />
              </Callout.Icon>
              <Callout.Text>{error}</Callout.Text>
              <IconButton
                variant="ghost"
                color="red"
                aria-label="Dismiss error"
                onClick={() => setError("")}
              >
                <X size={14} />
              </IconButton>
            </Callout.Root>
          )}
          <div className="page-heading">
            <div>
              <div className="eyebrow">
                {page === "settings"
                  ? "WORKSPACE"
                  : app?.display_name || "YOUR WORKSPACE"}
              </div>
              <h1>{nav.find((n) => n.id === page)?.label || "Settings"}</h1>
              <p>
                {
                  {
                    overview:
                      "Every release, patch, and device report. In one place.",
                    applications:
                      "A dedicated home for every application you ship.",
                    releases: "The reliable baselines behind every update.",
                    patches: "Small changes. A controlled path to your users.",
                    analytics:
                      "See what downloads, what launches, and what needs attention.",
                    settings: "Your workspace, access, and connection details.",
                  }[page]
                }
              </p>
            </div>
            <div className="heading-actions">
              {page !== "applications" &&
                page !== "settings" &&
                !!data?.apps.length && (
                  <Select.Root value={appId} onValueChange={setAppId}>
                    <Select.Trigger
                      placeholder="Select application"
                      className="app-select"
                    />
                    <Select.Content>
                      {data.apps.map((a) => (
                        <Select.Item key={a.id} value={a.id}>
                          {a.display_name}
                        </Select.Item>
                      ))}
                    </Select.Content>
                  </Select.Root>
                )}
              {(page === "overview" || page === "applications") && (
                <Button onClick={() => setCreateOpen(true)}>
                  <Plus size={16} />
                  New application
                </Button>
              )}
              {page === "analytics" && (
                <Select.Root value={days} onValueChange={setDays}>
                  <Select.Trigger />
                  <Select.Content>
                    <Select.Item value="7">Last 7 days</Select.Item>
                    <Select.Item value="30">Last 30 days</Select.Item>
                  </Select.Content>
                </Select.Root>
              )}
            </div>
          </div>
          {!data ? (
            <div className="data-loading">
              <Spinner size="3" />
              Loading your workspace…
            </div>
          ) : !data.apps.length && page !== "settings" ? (
            <section className="panel">
              <Empty
                icon={Rocket}
                title="Ready for your first launch?"
                description="Create an application, then publish its baseline release with the CLI."
                action={
                  <Button onClick={() => setCreateOpen(true)}>
                    <Plus size={16} />
                    Create application
                  </Button>
                }
              />
            </section>
          ) : (
            <>
              {(page === "overview" || page === "analytics") && (
                <>
                  {page === "overview" && (
                    <div className="section-kicker">
                      <span>At a glance</span>
                      <span className="muted">
                        <Clock3 size={13} />
                        Last {days} days · UTC
                      </span>
                    </div>
                  )}
                  {stats}
                  <div
                    className={
                      page === "overview" ? "overview-grid" : "analytics-chart"
                    }
                  >
                    {activityChart}
                    {page === "overview" && (
                      <section className="panel recent-panel">
                        <div className="panel-heading">
                          <div>
                            <h2>Recent activity</h2>
                            <p>The latest signals from your devices.</p>
                          </div>
                          <Activity size={16} />
                        </div>
                        <div className="activity-list">
                          {events
                            .slice(-5)
                            .reverse()
                            .map((e, i) => (
                              <div className="activity-item" key={i}>
                                <div
                                  className={`activity-icon ${e.event.type.includes("failure") ? "failure" : ""}`}
                                >
                                  {e.event.type === "__patch_download__" ? (
                                    <Download size={15} />
                                  ) : e.event.type.includes("failure") ? (
                                    <TriangleAlert size={15} />
                                  ) : (
                                    <Check size={15} />
                                  )}
                                </div>
                                <div>
                                  <strong>{reportName(e)}</strong>
                                  <p>
                                    {e.event.release_version || "Release"}
                                    <span>·</span>Patch #
                                    {e.event.patch_number || "—"}
                                  </p>
                                </div>
                                <small>{ago(e.received_at)}</small>
                              </div>
                            ))}
                          {!events.length && (
                            <Empty
                              icon={Activity}
                              title="Listening for device reports"
                              description="Downloads and launches will appear here."
                            />
                          )}
                        </div>
                        <button
                          className="panel-link"
                          onClick={() => navigate("analytics")}
                        >
                          Explore analytics
                          <ArrowRight size={14} />
                        </button>
                      </section>
                    )}
                  </div>
                </>
              )}

              {page === "overview" && (
                <section className="panel">
                  <div className="panel-heading">
                    <div>
                      <h2>
                        Latest releases{" "}
                        <span className="heading-count">{releases.length}</span>
                      </h2>
                      <p>Baseline builds ready for their next patch.</p>
                    </div>
                    <Button
                      variant="ghost"
                      color="gray"
                      size="1"
                      onClick={() => navigate("releases")}
                    >
                      View all releases
                      <ArrowRight size={14} />
                    </Button>
                  </div>
                  {releaseTable(releases.slice(0, 3))}
                </section>
              )}

              {page === "applications" && (
                <div className="application-grid">
                  {data.apps
                    .filter((a) =>
                      a.display_name
                        .toLowerCase()
                        .includes(search.toLowerCase()),
                    )
                    .map((a) => {
                      const rs = data.releases.filter((r) => r.app_id === a.id),
                        ps = data.patches.filter((p) =>
                          rs.some((r) => r.id === p.release_id),
                        );
                      return (
                        <section className="application-card" key={a.id}>
                          <div className="application-card-top">
                            <span className="application-icon">
                              <Box size={24} />
                            </span>
                            <Badge
                              color={rs.length ? "green" : "gray"}
                              variant="soft"
                            >
                              {rs.length ? "Configured" : "New application"}
                            </Badge>
                          </div>
                          <h2>{a.display_name}</h2>
                          <div className="app-id">
                            <code>{a.id}</code>
                            <IconButton
                              variant="ghost"
                              size="1"
                              color="gray"
                              aria-label="Copy application ID"
                              onClick={() =>
                                void copy(a.id, "Application ID copied")
                              }
                            >
                              <Copy size={13} />
                            </IconButton>
                          </div>
                          <div className="application-facts">
                            <div>
                              <strong>{rs.length}</strong>
                              <span>Releases</span>
                            </div>
                            <div>
                              <strong>{ps.length}</strong>
                              <span>Patches</span>
                            </div>
                            <div>
                              <strong>{rs.at(-1)?.version || "—"}</strong>
                              <span>Latest release</span>
                            </div>
                          </div>
                          <button
                            className="application-open"
                            onClick={() => {
                              setAppId(a.id);
                              navigate("overview");
                            }}
                          >
                            Open application
                            <ArrowUpRight size={16} />
                          </button>
                        </section>
                      );
                    })}
                </div>
              )}

              {(page === "releases" || page === "patches") && (
                <section className="panel">
                  <div className="table-toolbar">
                    <div className="table-toolbar-title">
                      {page === "releases" ? (
                        <Layers3 size={17} />
                      ) : (
                        <GitBranch size={17} />
                      )}
                      <strong>
                        {page === "releases" ? "All releases" : "All patches"}
                      </strong>
                      <span className="heading-count">
                        {page === "releases" ? releases.length : patches.length}
                      </span>
                    </div>
                    <TextField.Root
                      value={search}
                      onChange={(e) => setSearch(e.target.value)}
                      placeholder={
                        page === "releases"
                          ? "Search releases…"
                          : "Search patches…"
                      }
                      aria-label="Search records"
                    >
                      <TextField.Slot>
                        <Search size={15} />
                      </TextField.Slot>
                    </TextField.Root>
                  </div>
                  {page === "releases"
                    ? releaseTable(matchedReleases)
                    : patchTable(matchedPatches)}
                  <div className="table-foot">
                    <ShieldCheck size={13} />
                    {page === "releases"
                      ? "Release artifacts are immutable baselines."
                      : "Promotion controls delivery. Rollbacks take effect after a device checks and restarts."}
                  </div>
                </section>
              )}

              {page === "analytics" && (
                <section className="panel">
                  <div className="panel-heading">
                    <div>
                      <h2>Device event log</h2>
                      <p>
                        Most recent 100 reports within your selected period.
                      </p>
                    </div>
                    <Badge variant="outline" color="gray">
                      {period.length} reports
                    </Badge>
                  </div>
                  {period.length ? (
                    <div className="table-scroll">
                      <table>
                        <thead>
                          <tr>
                            <th>Event</th>
                            <th>Release</th>
                            <th>Patch</th>
                            <th>Device</th>
                            <th>Received</th>
                          </tr>
                        </thead>
                        <tbody>
                          {period
                            .slice(-100)
                            .reverse()
                            .map((e, i) => (
                              <tr key={i}>
                                <td>
                                  <span
                                    className={`event-label ${e.event.type.includes("failure") ? "event-failure" : ""}`}
                                  >
                                    <span className="status-dot" />
                                    {reportName(e)}
                                  </span>
                                  {e.event.message && (
                                    <small
                                      className="row-sub event-message"
                                      title={e.event.message}
                                    >
                                      {e.event.message}
                                    </small>
                                  )}
                                </td>
                                <td>
                                  <code>{e.event.release_version || "—"}</code>
                                </td>
                                <td>#{e.event.patch_number || "—"}</td>
                                <td className="muted">
                                  {e.event.platform || "—"} /{" "}
                                  {e.event.arch || "—"}
                                </td>
                                <td className="muted">
                                  {new Date(e.received_at).toLocaleString()}
                                </td>
                              </tr>
                            ))}
                        </tbody>
                      </table>
                    </div>
                  ) : (
                    <Empty
                      icon={Activity}
                      title="No reports in this period"
                      description="Device events will appear after a patch downloads or launches."
                    />
                  )}
                  <div className="table-foot">
                    <CircleHelp size={14} />
                    Client-reported, retained events. Repeated reports are
                    counted; these are not total device or adoption figures.
                  </div>
                </section>
              )}

              {page === "settings" && (
                <div className="settings-grid">
                  <section className="panel">
                    <div className="panel-heading">
                      <div>
                        <h2>Connection</h2>
                        <p>One origin for your console and update API.</p>
                      </div>
                      <Code2 size={19} />
                    </div>
                    <div className="setting-row">
                      <span>API origin</span>
                      <code>{location.origin}</code>
                      <IconButton
                        variant="ghost"
                        color="gray"
                        aria-label="Copy API origin"
                        onClick={() => void copy(location.origin)}
                      >
                        <Copy size={15} />
                      </IconButton>
                    </div>
                    <div className="setting-row">
                      <span>Environment</span>
                      <Badge variant="soft" color="green">
                        {location.hostname === "apps.gmjroaming.com"
                          ? "Production"
                          : preview
                            ? "Preview"
                            : "Development"}
                      </Badge>
                    </div>
                    <div className="setting-help">
                      <Terminal size={17} />
                      <div>
                        <strong>Connect your CLI</strong>
                        <p>
                          Use your server's API key for CLI requests. Dashboard
                          sessions stay in an HTTP-only cookie.
                        </p>
                        <code>SHOREBIRD_HOSTED_URL={location.origin}</code>
                      </div>
                    </div>
                  </section>
                  <section className="panel">
                    <div className="panel-heading">
                      <div>
                        <h2>Access & session</h2>
                        <p>Your administrator session.</p>
                      </div>
                      <ShieldCheck size={19} />
                    </div>
                    <div className="setting-row">
                      <span>Username</span>
                      <strong>{user.username}</strong>
                    </div>
                    <div className="setting-row">
                      <span>Session lifetime</span>
                      <span>8 hours</span>
                    </div>
                    <div className="setting-row">
                      <span>Role</span>
                      <Badge variant="soft" color="gray">
                        Administrator
                      </Badge>
                    </div>
                    <div className="setting-help">
                      <LockKeyhole size={17} />
                      <div>
                        <strong>Password access</strong>
                        <p>
                          Your administrator can reset credentials on the
                          server. Password changes invalidate existing sessions.
                        </p>
                        <Button
                          color="gray"
                          variant="soft"
                          onClick={() => void signout()}
                        >
                          <LogOut size={14} />
                          Sign out
                        </Button>
                      </div>
                    </div>
                  </section>
                </div>
              )}
            </>
          )}
          <footer className="content-footer">
            <span>
              <Orbit size={15} />
              Saturn Code Push<span className="footer-separator">/</span>
              Self-hosted
            </span>
            <span>
              {synced
                ? `Synced ${synced.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" })}`
                : "Connecting to server"}
            </span>
          </footer>
        </main>
      </div>

      <Dialog.Root open={createOpen} onOpenChange={setCreateOpen}>
        <Dialog.Content maxWidth="440px">
          <Dialog.Title>New application</Dialog.Title>
          <Dialog.Description size="2" mb="5">
            Create a dedicated release pipeline for your app.
          </Dialog.Description>
          <form onSubmit={createApp}>
            <label className="dialog-label" htmlFor="app-name">
              Application name
            </label>
            <TextField.Root
              id="app-name"
              placeholder="e.g. Customer mobile app"
              size="3"
              value={appName}
              onChange={(e) => setAppName(e.target.value)}
              required
              maxLength={256}
            />
            <div className="dialog-buttons">
              <Dialog.Close>
                <Button type="button" variant="soft" color="gray">
                  Cancel
                </Button>
              </Dialog.Close>
              <Button type="submit" loading={saving}>
                <Plus size={15} />
                Create application
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Root>

      <Dialog.Root
        open={!!action}
        onOpenChange={(open) => {
          if (!open && !saving) setAction(null);
        }}
      >
        <Dialog.Content maxWidth="440px">
          <Dialog.Title>
            {action?.kind === "promote"
              ? "Promote"
              : action?.kind === "rollback"
                ? "Roll back"
                : "Restore"}{" "}
            patch #{action?.patch.number}
          </Dialog.Title>
          <Dialog.Description size="2" mb="5">
            {action?.kind === "promote"
              ? "Make this patch available to devices on the selected channel."
              : action?.kind === "rollback"
                ? "Devices will remove this patch on their next update check and use a prior patch or baseline after restart."
                : "Make this patch available again on its existing channels."}
          </Dialog.Description>
          {action?.kind === "promote" && (
            <div className="dialog-field">
              <label className="dialog-label">Delivery channel</label>
              <Select.Root value={channelId} onValueChange={setChannelId}>
                <Select.Trigger />
                <Select.Content>
                  {channels.map((c) => (
                    <Select.Item key={c.id} value={String(c.id)}>
                      {c.name}
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Root>
            </div>
          )}
          <div className="dialog-buttons">
            <Dialog.Close>
              <Button variant="soft" color="gray" disabled={saving}>
                Cancel
              </Button>
            </Dialog.Close>
            <Button
              loading={saving}
              color={action?.kind === "rollback" ? "red" : "gray"}
              onClick={() => void executeAction()}
            >
              {action?.kind === "promote"
                ? "Promote patch"
                : action?.kind === "rollback"
                  ? "Roll back patch"
                  : "Restore patch"}
              <ArrowRight size={15} />
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Root>

      {notice && (
        <div className="toast" role="status">
          <CheckCircle2 size={18} />
          {notice}
          <button
            onClick={() => setNotice("")}
            aria-label="Dismiss notification"
          >
            <X size={14} />
          </button>
        </div>
      )}
    </div>
  );
}

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Theme
      appearance="dark"
      accentColor="gray"
      grayColor="gray"
      radius="large"
      scaling="100%"
    >
      <Console />
    </Theme>
  </React.StrictMode>,
);
