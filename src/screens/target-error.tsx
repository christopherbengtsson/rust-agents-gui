import {
  Link,
  useRouter,
  type ErrorComponentProps,
} from "@tanstack/react-router";
import { Button } from "@/components/ui/button";
export function TargetError({ error }: ErrorComponentProps) {
  const router = useRouter();
  return (
    <main id="workspace" tabIndex={-1} className="workspace-region flex-1">
      <header className="workspace-header">
        <h1>Destination unavailable</h1>
      </header>
      <div className="workspace-content">
        <p>
          {error instanceof Error
            ? error.message
            : "The destination could not be loaded. Retry its observations."}
        </p>
        <Button variant="outline" onClick={() => void router.invalidate()}>
          Retry destination
        </Button>
        <Link to="/">Return home</Link>
      </div>
    </main>
  );
}
