// Private shared mechanism. Concrete feature modules own tag mapping and interests.
export class FeatureFreshness {
  private identities = new Map<string, number>();
  private clock = 0;
  private global = 0;
  private owner?: string;
  private baseline = false;
  private listeners = new Set<() => void>();

  connect(owner: string | undefined) {
    if (owner !== this.owner) {
      this.identities.clear();
      this.global = ++this.clock;
    }
    this.owner = owner;
    this.baseline = false;
    this.publish();
  }
  currentOwner() {
    return this.owner;
  }
  matches(owner: string) {
    return this.owner === owner;
  }
  ready(owner: string) {
    return this.owner === owner && this.baseline;
  }
  mark(identity: string) {
    this.identities.set(identity, ++this.clock);
    if (this.identities.size > 256) this.resync();
  }
  resync() {
    this.identities.clear();
    this.global = ++this.clock;
  }
  version(identities: readonly string[]) {
    return Math.max(
      this.global,
      ...identities.map((id) => this.identities.get(id) ?? 0),
    );
  }
  accept(owner: string) {
    if (this.owner !== owner) return;
    this.baseline = true;
    this.publish();
  }
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };
  private publish() {
    this.listeners.forEach((listener) => listener());
  }
}
