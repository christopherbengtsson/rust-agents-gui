---
title: "Match Effect dependencies to each synchronization process"
impact: HIGH
impactDescription: "Avoids stale synchronization and unrelated resource restarts"
tags: effects, dependencies, lifecycle
---

# Match Effect dependencies to each synchronization process

Apply when an Effect reads changing inputs or combines unrelated resource work.
Keep dependencies complete, use actual synchronization inputs, and separate
independent processes with different lifetimes. Change the code before changing
its dependency list.

**Incorrect (the connection remains bound to the initial room):**

```tsx
useEffect(() => {
  const connection = connect(room.id);
  return () => connection.disconnect();
}, []);
```

**Correct (only the identity actually read controls this resource):**

```tsx
const roomId = room.id;
useEffect(() => {
  const connection = connect(roomId);
  return () => connection.disconnect();
}, [roomId]);
```

Here `connect` is an illustrative stable module function. Other fields read by the
Effect would need their own dependencies. A smaller list is not inherently better.
Splitting independent Effects does not require extracting a custom Hook, and
manually splitting render calculations just to imitate Compiler is unnecessary.

Source: [Removing Effect dependencies](https://react.dev/learn/removing-effect-dependencies).
