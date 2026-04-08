function createUser(name: string, age: number, email: string) {
  return { name, age, email };
}

const user1 = createUser("Alice", 30, "alice@example.com");
const user2 = createUser("Bob", 25, "bob@example.com");
