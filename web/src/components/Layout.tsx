import { ParentProps } from "solid-js";
import { A } from "@solidjs/router";

export function Layout(props: ParentProps) {
  return (
    <div class="layout">
      <nav class="nav">
        <A href="/" class="nav-brand">Nectar</A>
        <div class="nav-links">
          <A href="/">Home</A>
          <A href="/search">Search</A>
          <A href="/settings">Settings</A>
        </div>
      </nav>
      <main class="content">{props.children}</main>
    </div>
  );
}
