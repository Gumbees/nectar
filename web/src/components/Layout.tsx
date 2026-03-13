import { createSignal, createResource, For, Show, createMemo, ParentProps } from "solid-js";
import { A, useLocation } from "@solidjs/router";
import { useAuth } from "../stores/auth";
import { fetchLibraries } from "../lib/api";
import type { Library } from "../types";
import styles from "./Layout.module.css";

export function Layout(props: ParentProps) {
  const auth = useAuth();
  const location = useLocation();
  const [collapsed, setCollapsed] = createSignal(false);

  const [librariesData] = createResource(
    () => auth.isAuthenticated(),
    (authed) => authed ? fetchLibraries() : Promise.resolve({ libraries: [] })
  );

  const libraries = () => librariesData()?.libraries ?? [];

  const isPlayerPage = createMemo(() => location.pathname.startsWith("/play/"));

  const isActive = (path: string) => {
    if (path === "/") return location.pathname === "/";
    return location.pathname.startsWith(path);
  };

  const libraryIcon = (lib: Library) => {
    const icons: Record<string, string> = {
      movies: "\uD83C\uDFAC",
      shows: "\uD83D\uDCFA",
      music: "\uD83C\uDFB5",
      books: "\uD83D\uDCDA",
      photos: "\uD83D\uDCF7",
    };
    return icons[lib.library_type] ?? "\uD83D\uDCC1";
  };

  return (
    <div class={styles.layout}>
      {/* Desktop Sidebar */}
      <nav
        class={`${styles.sidebar} ${collapsed() ? styles.sidebarCollapsed : ""} ${isPlayerPage() ? styles.hidden : ""}`}
      >
        <div class={styles.sidebarHeader}>
          <A href="/" class={styles.brand}>Nectar</A>
          <button
            class={styles.collapseBtn}
            onClick={() => setCollapsed(!collapsed())}
            title={collapsed() ? "Expand" : "Collapse"}
          >
            {collapsed() ? "\u25B6" : "\u25C0"}
          </button>
        </div>

        <div class={styles.navSection}>
          <A
            href="/"
            class={`${styles.navLink} ${isActive("/") && !isActive("/search") && !isActive("/settings") && !isActive("/library") && !isActive("/detail") ? styles.navLinkActive : ""}`}
          >
            <span class={styles.navIcon}>{"\uD83C\uDFE0"}</span>
            <span class={styles.navLabel}>Home</span>
          </A>
          <A
            href="/search"
            class={`${styles.navLink} ${isActive("/search") ? styles.navLinkActive : ""}`}
          >
            <span class={styles.navIcon}>{"\uD83D\uDD0D"}</span>
            <span class={styles.navLabel}>Search</span>
          </A>
        </div>

        <Show when={libraries().length > 0}>
          <div class={styles.navSection}>
            <div class={styles.navSectionLabel}>Libraries</div>
            <For each={libraries()}>
              {(lib) => (
                <A
                  href={`/library/${lib.id}`}
                  class={`${styles.navLink} ${location.pathname === `/library/${lib.id}` ? styles.navLinkActive : ""}`}
                >
                  <span class={styles.navIcon}>{libraryIcon(lib)}</span>
                  <span class={styles.navLabel}>{lib.name}</span>
                </A>
              )}
            </For>
          </div>
        </Show>

        <div class={styles.sidebarFooter}>
          <Show when={auth.isAuthenticated()}>
            <A href="/settings" class={styles.userInfo}>
              <div class={styles.userAvatar}>
                {auth.user()?.username?.charAt(0).toUpperCase() ?? "?"}
              </div>
              <span class={styles.userName}>{auth.user()?.username}</span>
            </A>
          </Show>
        </div>
      </nav>

      {/* Main Content */}
      <main
        class={`${styles.content} ${collapsed() ? styles.contentCollapsed : ""} ${isPlayerPage() ? styles.contentCollapsed : ""}`}
        style={isPlayerPage() ? { "margin-left": "0", "padding-bottom": "0" } : {}}
      >
        {props.children}
      </main>

      {/* Mobile Bottom Bar */}
      <div class={`${styles.bottomBar} ${isPlayerPage() ? styles.hidden : ""}`}>
        <div class={styles.bottomBarInner}>
          <A href="/" class={`${styles.bottomLink} ${isActive("/") && !isActive("/search") && !isActive("/settings") ? styles.bottomLinkActive : ""}`}>
            <span class={styles.bottomIcon}>{"\uD83C\uDFE0"}</span>
            <span class={styles.bottomLabel}>Home</span>
          </A>
          <A href="/search" class={`${styles.bottomLink} ${isActive("/search") ? styles.bottomLinkActive : ""}`}>
            <span class={styles.bottomIcon}>{"\uD83D\uDD0D"}</span>
            <span class={styles.bottomLabel}>Search</span>
          </A>
          <Show when={libraries().length > 0}>
            <A href={`/library/${libraries()[0].id}`} class={`${styles.bottomLink} ${isActive("/library") ? styles.bottomLinkActive : ""}`}>
              <span class={styles.bottomIcon}>{"\uD83D\uDCDA"}</span>
              <span class={styles.bottomLabel}>Libraries</span>
            </A>
          </Show>
          <A href="/settings" class={`${styles.bottomLink} ${isActive("/settings") ? styles.bottomLinkActive : ""}`}>
            <span class={styles.bottomIcon}>{"\u2699\uFE0F"}</span>
            <span class={styles.bottomLabel}>Settings</span>
          </A>
        </div>
      </div>
    </div>
  );
}
