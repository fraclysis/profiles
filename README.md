# Profiles

Sets your configuration to the parent process's environmental variables.

## Quick Start

Requires clang in Path.

```cmd
cargo run --release --features=build-setenv
```

## Profiles.toml

`Profiles.toml` takes an array of profile elements. Each profile element has own name, add, remove and an env block.

|        |                                                                       |
| ------ | --------------------------------------------------------------------- |
| name   | A string used for referencing the profile.                            |
| add    | An array of profile names to join with current profile's env block.   |
| remove | An array of profile names to remove from current profile's env block. |
| env    | A table of environment variables to set.                              |

### Example of Profiles.toml

```toml
[[profile]]
name = "clang"
[profile.env]
Path = ["D:/Apps/LLVM/bin"]

[[profile]]
name = "mingw"
remove = ["clang"]
[profile.env]
Path = ["D:/msys64/mingw64/bin"]

[[profile]]
name = "c-tools"
add = ["tools"]
[profile.env]
Path = ["D:/Apps/doxygen/bin", "D:/Apps/CMake/bin"]

[[profile]]
name = "vk10"
[profile.env]
VULKAN_SDK = "D:/VulkanSDK/1.0.3.0" 

[[profile]]
name = "vk13"
[profile.env]
VULKAN_SDK = "D:/VulkanSDK/1.3.239.0" 
```
