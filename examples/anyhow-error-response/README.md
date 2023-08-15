# Demo牵扯源码分析

## 内部类

### `Router`

```rust
pub struct Router {
    // 正常的业务处理函数
    path_router: PathRouter<S, false>
    // 异常业务的处理函数
    fallback_router: PathRouter<S, true>
    // 全局异常
    catch_all_fallback_: Fallback<S>
}
```

这个是一个总路由，主要包含正常的路径处理和异常处理

### PathRouter

```rust
pub struct PathRouter<S, const IS_FALLBACK: bool> {
    // 记录路由id对应的处理终端
    routes: HashMap<RouteId, Endpoint<S>>,
    // 主要记录路由id和Node的关系，Node主要是url路径
    Node: Arc<Node>,
    // 记录上次的路由id，每次自增1
    pre_route_id: RouteId,
}
```

`PathRouter`主要是把路径对应到`RouteId`

### Endpoint

```rust
pub struct Endpoint<S> {
    MethodRouter(MethodRouter<S>),
    Route(Route)
}
```

本Demo只用到了`MethodRouter`，

### MethodRouter

```rust
pub struct MethodRouter<S, E> {
    get: MethodEndpoint<S, E>,
    head: MethodEndpoint<S, E>,
    ...
}
```

`MethodRouter`这个是http method的具体处理路由，里面包含了各种method的处理终端

### MethodEndpoint

```rust
pub enum MethodEndpoint {
    None,
    Route(Route<E>),
    BoxedHandler(BoxedIntoRoute<S, E>)
}
```

`BoxedHandler`是一个可以将async handler转为route的一个类，其中`Handler`主要实现了`IntoResponse`，就可以了，也就是最后可以得到一个`Response`的响应。
`Response`，Response是`http.Response`，所以我们的handler有可能会犯`AppError`，所以`AppError`需要实现`IntoResponse`

## 第三方库

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
