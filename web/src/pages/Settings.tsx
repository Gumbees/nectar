import { createSignal, createResource, For, Show } from "solid-js";
import { useAuth } from "../stores/auth";
import { fetchLibraries, createLibrary, deleteLibrary, scanLibrary } from "../lib/api";
import type { Library } from "../types";
import styles from "./Settings.module.css";

type Tab = "profile" | "libraries" | "playback" | "about";

export function Settings() {
  const auth = useAuth();
  const [activeTab, setActiveTab] = createSignal<Tab>("profile");

  // Libraries
  const [librariesData, { refetch: refetchLibraries }] = createResource(fetchLibraries);
  const libraries = () => librariesData()?.libraries ?? [];

  // Add library form
  const [showAddForm, setShowAddForm] = createSignal(false);
  const [newName, setNewName] = createSignal("");
  const [newType, setNewType] = createSignal("movies");
  const [newPaths, setNewPaths] = createSignal("");
  const [addError, setAddError] = createSignal("");

  // Profile form
  const [profileMsg, setProfileMsg] = createSignal("");

  // Playback preferences
  const [subtitleLang, setSubtitleLang] = createSignal(
    localStorage.getItem("nectar_subtitle_lang") ?? "eng"
  );
  const [audioLang, setAudioLang] = createSignal(
    localStorage.getItem("nectar_audio_lang") ?? "eng"
  );
  const [qualityPref, setQualityPref] = createSignal(
    localStorage.getItem("nectar_quality") ?? "auto"
  );

  const handleAddLibrary = async (e: Event) => {
    e.preventDefault();
    setAddError("");
    try {
      const paths = newPaths().split("\n").map((p) => p.trim()).filter(Boolean);
      if (!newName().trim() || paths.length === 0) {
        setAddError("Name and at least one path required");
        return;
      }
      await createLibrary(newName().trim(), newType(), paths);
      setNewName("");
      setNewPaths("");
      setShowAddForm(false);
      refetchLibraries();
    } catch (err: any) {
      setAddError(err.message || "Failed to add library");
    }
  };

  const handleDeleteLibrary = async (id: string) => {
    if (!confirm("Delete this library? This cannot be undone.")) return;
    try {
      await deleteLibrary(id);
      refetchLibraries();
    } catch (err: any) {
      alert("Failed to delete: " + (err.message || "Unknown error"));
    }
  };

  const handleScan = async (id: string) => {
    try {
      await scanLibrary(id);
      alert("Scan started");
    } catch (err: any) {
      alert("Scan failed: " + (err.message || "Unknown error"));
    }
  };

  const savePlaybackPrefs = () => {
    localStorage.setItem("nectar_subtitle_lang", subtitleLang());
    localStorage.setItem("nectar_audio_lang", audioLang());
    localStorage.setItem("nectar_quality", qualityPref());
    setProfileMsg("Preferences saved");
    setTimeout(() => setProfileMsg(""), 2000);
  };

  const tabs: { key: Tab; label: string; adminOnly?: boolean }[] = [
    { key: "profile", label: "Profile" },
    { key: "libraries", label: "Libraries", adminOnly: true },
    { key: "playback", label: "Playback" },
    { key: "about", label: "About" },
  ];

  return (
    <div class={styles.page}>
      <h1 class={styles.title}>Settings</h1>

      <div class={styles.tabs}>
        <For each={tabs}>
          {(tab) => (
            <Show when={!tab.adminOnly || auth.isAdmin()}>
              <button
                class={`${styles.tab} ${activeTab() === tab.key ? styles.tabActive : ""}`}
                onClick={() => setActiveTab(tab.key)}
              >
                {tab.label}
              </button>
            </Show>
          )}
        </For>
      </div>

      {/* Profile Tab */}
      <Show when={activeTab() === "profile"}>
        <div class={styles.section}>
          <h2 class={styles.sectionTitle}>Profile</h2>
          <div class={styles.field}>
            <label class={styles.label}>Username</label>
            <input
              class={styles.input}
              type="text"
              value={auth.user()?.username ?? ""}
              disabled
            />
          </div>
          <div class={styles.field}>
            <label class={styles.label}>Email</label>
            <input
              class={styles.input}
              type="email"
              value={auth.user()?.email ?? ""}
              placeholder="Not set"
              disabled
            />
          </div>
        </div>

        <div class={styles.section}>
          <h2 class={styles.sectionTitle}>Change Password</h2>
          <div class={styles.field}>
            <label class={styles.label}>Current Password</label>
            <input class={styles.input} type="password" />
          </div>
          <div class={styles.field}>
            <label class={styles.label}>New Password</label>
            <input class={styles.input} type="password" />
          </div>
          <div class={styles.field}>
            <label class={styles.label}>Confirm New Password</label>
            <input class={styles.input} type="password" />
          </div>
          <div class={styles.btnRow}>
            <button class={`${styles.btn} ${styles.btnPrimary}`}>Update Password</button>
          </div>
        </div>

        <button
          class={`${styles.btn} ${styles.btnDanger} ${styles.logoutBtn}`}
          onClick={() => auth.logout()}
        >
          Sign Out
        </button>
      </Show>

      {/* Libraries Tab (Admin) */}
      <Show when={activeTab() === "libraries"}>
        <div class={styles.section}>
          <h2 class={styles.sectionTitle}>Libraries</h2>
          <div class={styles.libraryList}>
            <For each={libraries()}>
              {(lib) => (
                <div class={styles.libraryItem}>
                  <div class={styles.libraryInfo}>
                    <div class={styles.libraryName}>{lib.name}</div>
                    <div class={styles.libraryType}>{lib.library_type}</div>
                    <div class={styles.libraryPaths}>{lib.paths.join(", ")}</div>
                  </div>
                  <div class={styles.libraryActions}>
                    <button
                      class={styles.iconBtn}
                      onClick={() => handleScan(lib.id)}
                      title="Scan library"
                    >
                      {"\uD83D\uDD04"}
                    </button>
                    <button
                      class={styles.iconBtn}
                      onClick={() => handleDeleteLibrary(lib.id)}
                      title="Delete library"
                    >
                      {"\uD83D\uDDD1"}
                    </button>
                  </div>
                </div>
              )}
            </For>
          </div>

          <Show when={!showAddForm()}>
            <div class={styles.btnRow}>
              <button
                class={`${styles.btn} ${styles.btnPrimary}`}
                onClick={() => setShowAddForm(true)}
              >
                Add Library
              </button>
            </div>
          </Show>

          <Show when={showAddForm()}>
            <form class={styles.addForm} onSubmit={handleAddLibrary}>
              <Show when={addError()}>
                <div style={{ color: "var(--danger)", "font-size": "var(--font-sm)", "margin-bottom": "var(--space-3)" }}>
                  {addError()}
                </div>
              </Show>
              <div class={styles.field}>
                <label class={styles.label}>Name</label>
                <input
                  class={styles.input}
                  type="text"
                  value={newName()}
                  onInput={(e) => setNewName(e.currentTarget.value)}
                  placeholder="e.g. Movies"
                  required
                />
              </div>
              <div class={styles.field}>
                <label class={styles.label}>Type</label>
                <select
                  class={styles.select}
                  value={newType()}
                  onChange={(e) => setNewType(e.currentTarget.value)}
                >
                  <option value="movies">Movies</option>
                  <option value="shows">TV Shows</option>
                  <option value="music">Music</option>
                  <option value="books">Books</option>
                  <option value="photos">Photos</option>
                </select>
              </div>
              <div class={styles.field}>
                <label class={styles.label}>Paths (one per line)</label>
                <textarea
                  class={styles.input}
                  rows="3"
                  value={newPaths()}
                  onInput={(e) => setNewPaths(e.currentTarget.value)}
                  placeholder="/media/movies"
                  style={{ resize: "vertical" }}
                />
              </div>
              <div class={styles.btnRow}>
                <button class={`${styles.btn} ${styles.btnPrimary}`} type="submit">Create</button>
                <button
                  class={`${styles.btn} ${styles.btnSecondary}`}
                  type="button"
                  onClick={() => setShowAddForm(false)}
                >
                  Cancel
                </button>
              </div>
            </form>
          </Show>
        </div>
      </Show>

      {/* Playback Tab */}
      <Show when={activeTab() === "playback"}>
        <div class={styles.section}>
          <h2 class={styles.sectionTitle}>Playback Preferences</h2>

          <div class={styles.field}>
            <label class={styles.label}>Preferred Subtitle Language</label>
            <select
              class={styles.select}
              value={subtitleLang()}
              onChange={(e) => setSubtitleLang(e.currentTarget.value)}
            >
              <option value="eng">English</option>
              <option value="spa">Spanish</option>
              <option value="fre">French</option>
              <option value="deu">German</option>
              <option value="jpn">Japanese</option>
              <option value="kor">Korean</option>
              <option value="zho">Chinese</option>
              <option value="off">Off</option>
            </select>
          </div>

          <div class={styles.field}>
            <label class={styles.label}>Preferred Audio Language</label>
            <select
              class={styles.select}
              value={audioLang()}
              onChange={(e) => setAudioLang(e.currentTarget.value)}
            >
              <option value="eng">English</option>
              <option value="spa">Spanish</option>
              <option value="fre">French</option>
              <option value="deu">German</option>
              <option value="jpn">Japanese</option>
            </select>
          </div>

          <div class={styles.field}>
            <label class={styles.label}>Quality Preference</label>
            <select
              class={styles.select}
              value={qualityPref()}
              onChange={(e) => setQualityPref(e.currentTarget.value)}
            >
              <option value="auto">Auto</option>
              <option value="4k">4K (2160p)</option>
              <option value="1080p">1080p</option>
              <option value="720p">720p</option>
              <option value="480p">480p</option>
            </select>
          </div>

          <Show when={profileMsg()}>
            <div style={{ color: "var(--success)", "font-size": "var(--font-sm)", "margin-bottom": "var(--space-3)" }}>
              {profileMsg()}
            </div>
          </Show>

          <div class={styles.btnRow}>
            <button class={`${styles.btn} ${styles.btnPrimary}`} onClick={savePlaybackPrefs}>
              Save Preferences
            </button>
          </div>
        </div>
      </Show>

      {/* About Tab */}
      <Show when={activeTab() === "about"}>
        <div class={styles.section}>
          <h2 class={styles.sectionTitle}>About Nectar</h2>
          <div class={styles.aboutGrid}>
            <div class={styles.aboutItem}>
              <div class={styles.aboutLabel}>Version</div>
              <div class={styles.aboutValue}>0.1.0</div>
            </div>
            <div class={styles.aboutItem}>
              <div class={styles.aboutLabel}>Server</div>
              <div class={styles.aboutValue}>Nectar Media Server</div>
            </div>
            <div class={styles.aboutItem}>
              <div class={styles.aboutLabel}>Frontend</div>
              <div class={styles.aboutValue}>Solid.js PWA</div>
            </div>
            <div class={styles.aboutItem}>
              <div class={styles.aboutLabel}>License</div>
              <div class={styles.aboutValue}>MIT</div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
