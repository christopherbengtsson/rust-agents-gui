// Private shared mechanism. Concrete feature modules own tag mapping and interests.
export class FeatureFreshness {
  private identities = new Map<string, number>();
  private clock = 0;
  private global = 0;
  private incarnation?: string;
  private baseline = false;
  private listeners = new Set<() => void>();

  connect(incarnation: string | undefined) {
    if (incarnation !== this.incarnation) {
      this.identities.clear();
      this.global = ++this.clock;
    }
    this.incarnation = incarnation;
    this.baseline = false;
    this.publish();
  }
  currentIncarnation() {
    return this.incarnation;
  }
  matches(incarnation: string) {
    return this.incarnation === incarnation;
  }
  ready(incarnation: string) {
    return this.incarnation === incarnation && this.baseline;
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
  accept(incarnation: string) {
    if (this.incarnation !== incarnation) return;
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
