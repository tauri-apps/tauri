export default function ({ route, redirect }) {
  // redirect tauri's no-server default URL to /
  if (route.path.startsWith('/text/html,')) {
    redirect('301', '/')
  }
}
