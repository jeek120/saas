# Demo源码解析

## 三方库

### tracing

这是一个异步的日志库，
`tracing_subscriber::register().with(fmt::layer()).init()`这个是将日志输出到标准的屏幕，日志采用`event`,`traceId`, `span`组成
`?变量`是采用Debug方式输出, `%变量`是采用Display方式输出
