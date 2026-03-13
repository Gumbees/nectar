import { Router, Route } from "@solidjs/router";
import { Home } from "./pages/Home";
import { Library } from "./pages/Library";
import { Player } from "./pages/Player";
import { Search } from "./pages/Search";
import { Settings } from "./pages/Settings";
import { Layout } from "./components/Layout";

export default function App() {
  return (
    <Router root={Layout}>
      <Route path="/" component={Home} />
      <Route path="/library/:id" component={Library} />
      <Route path="/play/:id" component={Player} />
      <Route path="/search" component={Search} />
      <Route path="/settings" component={Settings} />
    </Router>
  );
}
