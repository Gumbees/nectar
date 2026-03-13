import { createContext, useContext, createSignal, onMount, JSX } from "solid-js";
import type { ParentProps } from "solid-js";
import type { User } from "../types";
import { fetchMe, login as apiLogin } from "../lib/api";

interface AuthContextValue {
  user: () => User | null;
  isAuthenticated: () => boolean;
  isAdmin: () => boolean;
  isLoading: () => boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
  checkAuth: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue>();

export function AuthProvider(props: ParentProps): JSX.Element {
  const [user, setUser] = createSignal<User | null>(null);
  const [isLoading, setIsLoading] = createSignal(true);

  const isAuthenticated = () => user() !== null;
  const isAdmin = () => user()?.is_admin ?? false;

  const checkAuth = async () => {
    const token = localStorage.getItem("nectar_token");
    if (!token) {
      setUser(null);
      setIsLoading(false);
      return;
    }
    try {
      const me = await fetchMe();
      setUser(me);
    } catch {
      setUser(null);
      localStorage.removeItem("nectar_token");
    } finally {
      setIsLoading(false);
    }
  };

  const login = async (username: string, password: string) => {
    const res = await apiLogin(username, password);
    localStorage.setItem("nectar_token", res.token);
    await checkAuth();
  };

  const logout = () => {
    localStorage.removeItem("nectar_token");
    setUser(null);
    window.location.href = "/login";
  };

  onMount(() => {
    checkAuth();
  });

  const value: AuthContextValue = {
    user,
    isAuthenticated,
    isAdmin,
    isLoading,
    login,
    logout,
    checkAuth,
  };

  return (
    <AuthContext.Provider value={value}>
      {props.children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return ctx;
}
