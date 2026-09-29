# dobby-sys

[psyche314/Dobby](https://github.com/psyche314/Dobby) 的底层 Rust 绑定。
提供原始 C ABI 和原生库构建。所有字段公开，所有原生函数都是 unsafe。

当前绑定版本为 0.1.0，对应 Dobby 1.0.0；具体 revision 固定在子模块中。
Git 依赖会取得固定的子模块；构建脚本不下载源码，也不执行 bindgen。
Cargo 打包会包含所需的原生源码和许可证。

## 使用

```toml
[dependencies]
dobby-sys = { git = "https://github.com/psyche314/dobby-sys", tag = "v0.1.0" }
```

完整调用示例见 [Rust/C E2E](tests/e2e/src/lib.rs)。先 Prepare，
再发布原函数指针，最后 Commit；AtomicPtr 必须通过原子 store 发布，
不能将其内部存储直接用作 C 的输出参数。

直接使用 DobbyPrepare、DobbyCommit、DobbyDisable、DobbyEnable、DobbyHook、
DobbyDestroy、DobbyInstrument、DobbyCodePatch、DobbySymbolResolver、
DobbyGetVersion 及三个配置函数。原生未实现的 DobbyImportTableReplace 不导出。

## 构建

需要 Rust 1.85+、CMake 3.18+、Ninja 和目标平台的 C/C++ 工具链。
库静态链接进调用方，不需要额外分发 libdobby.so。
Rust 绑定使用 no_std，但原生 Dobby 仍依赖操作系统及 C++ 运行库。

- Windows：x86_64-pc-windows-msvc，安装 LLVM Clang 与 Visual Studio C++ 工具链。
- Linux：GCC/Clang 和对应的 C++ 运行库；交叉编译需配置 CC/CXX 或 CMake 工具链。
- macOS：Xcode 命令行工具；支持 x86_64/aarch64。
- Android：NDK 四 ABI；使用 cargo-ndk 或配置 Rust 的目标 linker。

Android 设置 ANDROID_NDK_HOME，或由 cargo-ndk 提供 CARGO_NDK_CMAKE_TOOLCHAIN_PATH。
原生 API level 默认 21，可通过 ANDROID_PLATFORM=26 或 android-26 指定。
本 crate 自动链接 NDK 的 C++ 静态运行库及 Clang builtins，调用方不需要
手动补链接库；不会把整个 NDK sysroot 提前放进链接搜索路径，以免误链接 libc.a。

```sh
ANDROID_PLATFORM=26 cargo ndk -t arm64-v8a -P 26 build
```

可选 features：

- full-floating-point-register-pack：ARM64 插桩保存 q0–q31，默认仅保存 q0–q7。
- native-debug：开启原生诊断日志；不改变调用方的 Rust profile。

不支持的平台会明确报错，不会只生成一个无法链接的空壳。

## 安全约定

管理锁不使机器码写入原子化，也不会自动暂停线程。提交、启停与销毁时，
调用者必须保证没有线程正在执行相关入口、替换函数或 trampoline。
返回 0 表示成功；-1 表示当前操作失败；-2 表示字节写入后的系统操作失败，
不能据此假设 hook 没有生效。遇到 -2 应保持相关执行静止并处理错误。

插桩回调参数顺序为 (address, context)，不得让 panic 穿过 C ABI，也不能保留 context 指针。
只通过裸指针读取有效字段，不复制整个上下文，不读取 dummy 槽位。
ARM64 的 q8–q31 需要启用对应 feature。Destroy 后不能再调用原函数指针；
原生代码 arena 仍保留到进程结束。完整原生约定见 [Dobby API 文档](https://github.com/psyche314/Dobby/blob/master/docs/api.md)。

## 维护与验收

从源码克隆时先执行 git submodule update --init --recursive。
绑定手写并直接对应固定的原生标头，使用具名寄存器字段，不保留
bindgen 的匿名类型层级。bindgen 可以用于临时对照，但不能替代 ABI 验证。
更新子模块或结构布局后运行：

```sh
cargo run --manifest-path tests/e2e/Cargo.toml
```

E2E 将 Rust 暂存器布局与 C 标头对照，并实际执行 Rust 替换函数、原函数
trampoline、准备／提交、启停、销毁、C++→Rust 插桩回调及寄存器修改。
Windows/Linux/macOS 的 CI 还验证 Cargo 包及固定 revision 的 Git 消费者。

Android E2E 使用原生仓库的测试 launcher，不再复制一套 APK 构建逻辑：

```sh
ANDROID_PLATFORM=26 cargo ndk -t arm64-v8a -P 26 build --manifest-path tests/e2e/Cargo.toml
python tools/android-e2e.py --library tests/e2e/target/aarch64-linux-android/debug/libdobby_e2e_jni.so --sdk <android-sdk> --abi arm64-v8a --serial <serial> --out <output-dir>
```

使用 Apache-2.0 许可；原生依赖保留各自的来源与许可证。
