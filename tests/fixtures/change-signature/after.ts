function createUser(opts: { name: string; age: number; email: string }) {
  return { name: opts.name, age: opts.age, email: opts.email };
}

const user1 = createUser({ name: "Alice", age: 30, email: "alice@example.com" });
const user2 = createUser({ name: "Bob", age: 25, email: "bob@example.com" });
