import { TargetError } from "./target-error";
import {
  Link,
  useParams,
  SearchParamError,
  type ErrorComponentProps,
} from "@tanstack/react-router";
export function InvalidView(props: ErrorComponentProps) {
  const params = useParams({ strict: false });
  if (!(props.error instanceof SearchParamError))
    return <TargetError {...props} />;
  return (
    <main id="workspace" className="workspace-region flex-1">
      <div className="workspace-content">
        <h1>Invalid workspace view</h1>
        <p>Open Workspace or Changes for this destination.</p>
        {params.repositoryId && params.workspaceId && (
          <>
            <Link
              to="/repository/$repositoryId/workspace/$workspaceId"
              params={{
                repositoryId: params.repositoryId,
                workspaceId: params.workspaceId,
              }}
              search={{ view: "workspace" }}
            >
              Workspace
            </Link>
            <Link
              to="/repository/$repositoryId/workspace/$workspaceId"
              params={{
                repositoryId: params.repositoryId,
                workspaceId: params.workspaceId,
              }}
              search={{ view: "changes" }}
            >
              Changes
            </Link>
          </>
        )}
      </div>
    </main>
  );
}
