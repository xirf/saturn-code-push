// Only reachable through the Vite development preview. Never used for production API data.
const stamp = (days: number, hours = 0) =>
  new Date(Date.now() - days * 86400000 - hours * 3600000).toISOString();
export const previewData = {
  apps: [
    { id: "app-lumen", display_name: "Lumen Mobile", created_at: stamp(30) },
    { id: "app-orbit", display_name: "Orbit Companion", created_at: stamp(20) },
    { id: "app-probe", display_name: "Android probe", created_at: stamp(10) },
  ],
  channels: [
    { id: 1, app_id: "app-lumen", name: "stable" },
    { id: 2, app_id: "app-lumen", name: "staging" },
    { id: 3, app_id: "app-lumen", name: "beta" },
  ],
  releases: [
    {
      id: 1,
      app_id: "app-lumen",
      version: "2.3.0+48",
      flutter_revision: "baf23dc375905d9103ca4325b062cadb7b8f90e5",
      platforms: { android: { status: "active" } },
      created_at: stamp(15),
    },
    {
      id: 2,
      app_id: "app-lumen",
      version: "2.3.1+49",
      flutter_revision: "baf23dc375905d9103ca4325b062cadb7b8f90e5",
      platforms: { android: { status: "active" } },
      created_at: stamp(9),
    },
    {
      id: 3,
      app_id: "app-lumen",
      version: "2.4.0+50",
      flutter_revision: "baf23dc375905d9103ca4325b062cadb7b8f90e5",
      platforms: { android: { status: "active" } },
      created_at: stamp(2),
    },
  ],
  patches: [
    {
      id: 1,
      release_id: 3,
      number: 1,
      channel_ids: [2],
      is_rolled_back: false,
      created_at: stamp(2),
    },
    {
      id: 2,
      release_id: 3,
      number: 2,
      channel_ids: [1, 3],
      is_rolled_back: false,
      created_at: stamp(1),
    },
    {
      id: 3,
      release_id: 2,
      number: 1,
      channel_ids: [1],
      is_rolled_back: true,
      created_at: stamp(6),
    },
  ],
  artifacts: [1, 2, 3].flatMap((id) => [
    {
      id: id * 2,
      app_id: "app-lumen",
      release_id: id,
      patch_id: null,
      arch: "aarch64",
      platform: "android",
      uploaded: true,
      size: 18300000,
    },
    {
      id: id * 2 + 1,
      app_id: "app-lumen",
      release_id: id === 3 ? 2 : 3,
      patch_id: id,
      arch: "aarch64",
      platform: "android",
      uploaded: true,
      size: 58209,
    },
  ]),
  events: [31, 43, 38, 52, 44, 69, 61].flatMap((n, i) =>
    Array.from({ length: n }, (_, j) => ({
      received_at: stamp(6 - i, j / 60),
      event: {
        app_id: "app-lumen",
        type: j % 3 === 0 ? "__patch_install__" : "__patch_download__",
        release_version: "2.4.0+50",
        patch_number: 2,
        client_id: `preview-install-${j}`,
        platform: "android",
        arch: "aarch64",
      },
    })),
  ),
};
