import { useEffect, useId, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderPlus } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Spinner } from "@/components/ui/spinner";
import type { Repositories } from "@/hooks/use-repositories";

export function RepositoryRegistration({
  repositories,
  connected,
}: {
  repositories: Repositories;
  connected: boolean;
}) {
  const [isOpen, setOpen] = useState(false);
  const [path, setPath] = useState(repositories.pending?.request.path ?? "");
  const [picking, setPicking] = useState(false);
  const [pickerError, setPickerError] = useState<string>();
  const generation = useRef(0);
  const id = useId();
  useEffect(
    () => () => {
      generation.current += 1;
    },
    [],
  );
  const error = pickerError ?? repositories.actionError?.message;
  const pending =
    repositories.submitting ||
    (!!repositories.pending && !repositories.actionError);
  function disclose(open: boolean) {
    generation.current += 1;
    setPicking(false);
    if (open && repositories.pending)
      setPath(repositories.pending.request.path);
    setOpen(open);
  }
  async function browse() {
    const current = generation.current;
    setPicking(true);
    setPickerError(undefined);
    try {
      const folder = await open({
        directory: true,
        multiple: false,
        title: "Choose an existing Git repository",
      });
      if (current === generation.current && typeof folder === "string")
        setPath(folder);
    } catch (error) {
      if (current === generation.current)
        setPickerError(
          `Cannot open the folder picker: ${String(error)}. Enter the absolute folder path instead.`,
        );
    } finally {
      if (current === generation.current) setPicking(false);
    }
  }
  return (
    <Dialog open={isOpen} onOpenChange={disclose}>
      <DialogTrigger
        render={
          <Button
            variant="ghost"
            size="sm"
            className="w-full justify-start"
            disabled={!connected}
          />
        }
      >
        <FolderPlus data-icon="inline-start" />
        Add repository
      </DialogTrigger>
      <DialogContent className="max-h-[calc(100dvh-3rem)] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Add repository</DialogTitle>
          <DialogDescription>
            Register an existing local Git repository. Its files stay untouched
            and no agent starts.
          </DialogDescription>
        </DialogHeader>
        <form
          className="flex flex-col gap-4"
          onSubmit={async (event) => {
            event.preventDefault();
            const current = generation.current;
            setPickerError(undefined);
            if (await repositories.register(path)) {
              if (generation.current === current) disclose(false);
            }
          }}
        >
          <FieldGroup>
            <Field data-invalid={!!error || undefined}>
              <FieldLabel htmlFor={id}>Repository folder</FieldLabel>
              <InputGroup>
                <InputGroupInput
                  id={id}
                  autoComplete="off"
                  spellCheck={false}
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                  disabled={pending || !!repositories.pending}
                  aria-invalid={!!error || undefined}
                  aria-describedby={`${id}-help${error ? ` ${id}-error` : ""}`}
                  placeholder="/Users/you/code/project"
                  required
                />
                <InputGroupAddon align="inline-end">
                  <InputGroupButton
                    onClick={() => void browse()}
                    disabled={picking || pending || !!repositories.pending}
                  >
                    {picking ? "Choosing…" : "Browse…"}
                  </InputGroupButton>
                </InputGroupAddon>
              </InputGroup>
              <FieldDescription id={`${id}-help`}>
                Choose a checkout folder, or enter its absolute path.
              </FieldDescription>
              {error && <FieldError id={`${id}-error`}>{error}</FieldError>}
            </Field>
          </FieldGroup>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => disclose(false)}
            >
              Close
            </Button>
            <Button
              type="submit"
              size="sm"
              disabled={!connected || !path || pending || picking}
            >
              {pending && (
                <Spinner data-icon="inline-start" aria-hidden="true" />
              )}
              {pending
                ? "Registering…"
                : repositories.pending
                  ? "Retry registration"
                  : "Register"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
