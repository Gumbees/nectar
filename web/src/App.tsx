import { Show } from "solid-js";
import { Router, Route } from "@solidjs/router";
import { AuthProvider, useAuth } from "./stores/auth";
import { Layout } from "./components/Layout";
import { Home } from "./pages/Home";
import { Library } from "./pages/Library";
import { Detail } from "./pages/Detail";
import { Player } from "./pages/Player";
import { Search } from "./pages/Search";
import { Settings } from "./pages/Settings";
import { Login } from "./pages/Login";

function AuthGuard(props: { children: any }) {
  const auth = useAuth();

  return (
    <Show when={!auth.isLoading()} fallback={
      <div style={{
        display: "flex",
        "align-items": "center",
        "justify-content": "center",
        height: "100vh",
        color: "var(--text-muted)",
      }}>
        Loading...
      </div>
    }>
      <Show when={auth.isAuthenticated()} fallback={<Login />}>
        {props.children}
      </Show>
    </Show>
  );
}

function ProtectedLayout(props: { children: any }) {
  return (
    <AuthGuard>
      <Layout>{props.children}</Layout>
    </AuthGuard>
  );
}

export default function App() {
  return (
    <AuthProvider>
      <Router root={ProtectedLayout}>
        <Route path="/login" component={Login} />
        <Route path="/" component={Home} />
        <Route path="/library/:id" component={Library} />
        <Route path="/detail/:id" component={Detail} />
        <Route path="/play/:id" component={Player} />
        <Route path="/search" component={Search} />
        <Route path="/settings" component={Settings} />
      </Router>
    </AuthProvider>
  );
}
