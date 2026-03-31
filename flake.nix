{
  description = "light-wallpaper";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        
        # 定义变量以便引用
        vulkan-loader-pkg = pkgs.vulkan-loader;
        vulkan-layers-pkg = pkgs.vulkan-validation-layers;
        # Intel/AMD 驱动通常包含在 mesa-drivers 或 vulkan-drivers 中
        # 在 Nixpkgs 中，vulkan-loader 通常不直接包含 ICD json，需要依赖 mesa 或具体的驱动包
        # 为了通用性，我们尝试从 pkgs.mesa.drivers 或系统路径查找，或者让 loader 自动发现
        # 但在纯 Nix Shell 中，最好显式指向 mesa 的 vulkan icd
        mesa-pkg = pkgs.mesa; 
      in {
        devShells.default = with pkgs; mkShell rec {
          buildInputs = [
            pkg-config
            rust-bin.nightly.latest.default
            libxkbcommon
            vulkan-loader-pkg
            vulkan-tools
            vulkan-layers-pkg
            wayland
            shaderc
            # 确保包含 mesa 以获取 Intel/AMD 的 Vulkan ICD (驱动)
            mesa-pkg 
          ];

          shellHook = ''
            # 1. 设置库路径 (你原本有的)
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";

            # 2. 【关键】设置 Vulkan Layer 路径
            # 指向包含 VkLayer_khronos_validation.json 的目录
            export VK_LAYER_PATH="${vulkan-layers-pkg}/share/vulkan/explicit_layer.d"

            # 3. 【关键】设置 Vulkan ICD (驱动) 路径
            # 告诉 Loader 去哪里找 Intel/AMD 的驱动描述文件 (icd.json)
            # 注意：如果你的宿主机是 NixOS，驱动通常在 /run/opengl-driver，但在 nix-shell 中需要显式指定
            export VK_ICD_FILENAMES="${mesa-pkg}/share/vulkan/icd.d/intel_icd.x86_64.json:${mesa-pkg}/share/vulkan/icd.d/radeon_icd.x86_64.json"
            
            # 调试输出
            echo "----------------------------------------"
            echo "Vulkan Environment Configured:"
            echo "VK_LAYER_PATH: $VK_LAYER_PATH"
            echo "VK_ICD_FILENAMES: $VK_ICD_FILENAMES"
            
            # 验证文件是否存在
            if [ -f "${vulkan-layers-pkg}/share/vulkan/explicit_layer.d/VkLayer_khronos_validation.json" ]; then
              echo "✅ Validation Layer JSON found."
            else
              echo "❌ ERROR: Validation Layer JSON NOT found!"
              ls -R ${vulkan-layers-pkg}
            fi
            
            if [ -f "${mesa-pkg}/share/vulkan/icd.d/intel_icd.x86_64.json" ]; then
              echo "✅ Intel Vulkan ICD JSON found."
            else
              echo "⚠️  Warning: Intel Vulkan ICD JSON not found in expected path. Trying fallback..."
              # 尝试查找是否有其他名字的 json
              find ${mesa-pkg} -name "*icd*.json" 2>/dev/null
            fi
            echo "----------------------------------------"
            
            # 建议运行一次 vulkaninfo 测试
            # vulkaninfo --summary
          '';
        };
      }
    );
}
