import { createSignal, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { useAuth } from "../stores/auth";
import styles from "./Login.module.css";

export function Login() {
  const auth = useAuth();
  const navigate = useNavigate();

  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [error, setError] = createSignal("");
  const [loading, setLoading] = createSignal(false);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      await auth.login(username(), password());
      navigate("/", { replace: true });
    } catch (err: any) {
      setError(err.message || "Login failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class={styles.page}>
      <div class={styles.card}>
        <div class={styles.brand}>
          <div class={styles.brandName}>Nectar</div>
          <div class={styles.brandTagline}>Your media, your way</div>
        </div>

        <Show when={error()}>
          <div class={styles.error}>{error()}</div>
        </Show>

        <form class={styles.form} onSubmit={handleSubmit}>
          <div class={styles.field}>
            <label class={styles.label} for="username">Username</label>
            <input
              id="username"
              class={styles.input}
              type="text"
              placeholder="Enter username"
              value={username()}
              onInput={(e) => setUsername(e.currentTarget.value)}
              autocomplete="username"
              required
            />
          </div>

          <div class={styles.field}>
            <label class={styles.label} for="password">Password</label>
            <input
              id="password"
              class={styles.input}
              type="password"
              placeholder="Enter password"
              value={password()}
              onInput={(e) => setPassword(e.currentTarget.value)}
              autocomplete="current-password"
              required
            />
          </div>

          <button
            class={styles.submit}
            type="submit"
            disabled={loading()}
          >
            {loading() ? "Signing in..." : "Sign In"}
          </button>
        </form>

        <div class={styles.divider}>or</div>

        <button class={styles.ssoButton} disabled>
          Sign in with SSO
        </button>
      </div>
    </div>
  );
}
