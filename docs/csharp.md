# C#: the `Bui` and `Bui.Forms` assemblies

The canonical C# guide lives in
[`bindings/csharp/README.md`](../bindings/csharp/README.md). It covers the
build, the Windows Forms-style layer, the components, ownership and threads,
and regenerating the interop layer from `BUI_kit.h`.

```sh
cmake -S . -B build -DBUI_BUILD_CSHARP_BINDINGS=ON
cmake --build build --target bui_kit_shared
dotnet run --project bindings/csharp/examples/Hello
```
