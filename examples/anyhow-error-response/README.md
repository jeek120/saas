### matchit
这是一个极速的URL路由
```rust
use matchit::Router;

let mut router = Router::new();
router.insert("/home", "Welcome!")?;
router.insert("/users/:id", "A user")?;

let matched = router.at("/users/978")?;
assert_eq!(matched.params.get("id"), Some("978"));
assert_eq!(*matched.value, "A User");
```
