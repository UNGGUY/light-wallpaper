<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
<!-- [![Contributors][contributors-shield]][contributors-url] -->
<!-- [![Forks][forks-shield]][forks-url] -->
<!-- [![Stargazers][stars-shield]][stars-url] -->
<!-- [![Issues][issues-shield]][issues-url] -->
<!-- [![Unlicense License][license-shield]][license-url] -->
<!-- [![LinkedIn][linkedin-shield]][linkedin-url] -->
<!---->


<!-- PROJECT LOGO -->
<br />
<div align="center">

```text
.____    .__       .__     __     __      __                                    
|    |   |__| ____ |  |___/  |_  /  \    /  \_____  ______ ______   ___________ 
|    |   |  |/ ___\|  |  \   __\ \   \/\/   /\__  \ \____ \\____ \_/ __ \_  __ \
|    |___|  / /_/  >   Y  \  |    \        /  / __ \|  |_> >  |_> >  ___/|  | \/
|_______ \__\___  /|___|  /__|     \__/\  /  (____  /   __/|   __/ \___  >__|   
        \/ /_____/      \/              \/        \/|__|   |__|        \/       

```

  <h3 align="center">light wallpaper</h3>

  <p align="center">
    An ultra-lightweight Wayland dynamic wallpaper engine powered purely by native Vulkan.
    <!-- <br /> -->
    <!-- <a href="https://github.com/othneildrew/Best-README-Template"><strong>Explore the docs »</strong></a> -->
    <!-- <br /> -->
    <!-- <br /> -->
    <!-- <a href="https://github.com/othneildrew/Best-README-Template">View Demo</a> -->
    <!-- &middot; -->
    <!-- <a href="https://github.com/othneildrew/Best-README-Template/issues/new?labels=bug&template=bug-report---.md">Report Bug</a> -->
    <!-- &middot; -->
    <!-- <a href="https://github.com/othneildrew/Best-README-Template/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a> -->
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project


light-wallpaper is a pure native Vulkan dynamic wallpaper engine for Wayland.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With



* [![Rust][Rust]][Rust-url]
* [![VulkanSDK][VulkanSDK]][VulkanSDK-url]


<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

1. Clone this repository to local:
```sh
git clone https://github.com/UNGGUY/light-wallpaper.git
```

2. Vulkan SDK
The most important component you'll need for developing Vulkan applications is the SDK. It includes the headers, standard validation layers, debugging tools and a loader for the Vulkan functions. The loader looks up the functions in the driver at runtime, similarly to GLEW for OpenGL - if you're familiar with that.

  - Download: Get the installer from [VulkanSDK-url].

  - Extract: After downloading, extract the archive to your desired installation directory (e.g., ~/Programs/vulkan/).

  - Configure: Add it to your system environment variables. Note: Please modify the version number in the path below to match the folder you just extracted.

If you are using the default Bash or Zsh shell (common on most Linux distributions), append the following commands to your ~/.bashrc ~/.zshrc file:
```sh
# Set the Vulkan SDK root directory (Modify this to match your actual installation path)
export VULKAN_SDK=~/Program/vulkan/1.4.350.0/x86_64

# Add Vulkan tools and libraries to the system PATH
export PATH=$VULKAN_SDK/bin:$PATH
export LD_LIBRARY_PATH=$VULKAN_SDK/lib:$LD_LIBRARY_PATH

# Configure validation layer paths
export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d
export VK_ADD_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d

# Provide build support for CMake and pkg-config
export PKG_CONFIG_PATH=$VULKAN_SDK/lib/pkgconfig/:$PKG_CONFIG_PATH
export CMAKE_PREFIX_PATH=$VULKAN_SDK:$VULKAN_SDK/lib/VulkanLoader:$CMAKE_PREFIX_PATH``
```


If you are using the Fish shell, add the following commands to your ~/.config/fish/config.fish file:
```fish
# Set the Vulkan SDK root directory (Modify this to match your actual installation path)
set -gx VULKAN_SDK ~/Program/vulkan/1.4.350.0/x86_64

# Add Vulkan tools and libraries to the system PATH
set -gx PATH $VULKAN_SDK/bin $PATH
set -gx LD_LIBRARY_PATH $VULKAN_SDK/lib $LD_LIBRARY_PATH

# Configure validation layer paths
set -gx VK_LAYER_PATH $VULKAN_SDK/share/vulkan/explicit_layer.d
set -gx VK_ADD_LAYER_PATH $VULKAN_SDK/share/vulkan/explicit_layer.d

# Provide build support for CMake and pkg-config
set -gx PKG_CONFIG_PATH $VULKAN_SDK/lib/pkgconfig/ $PKG_CONFIG_PATH
set -gx CMAKE_PREFIX_PATH $VULKAN_SDK $VULKAN_SDK/lib/VulkanLoader $CMAKE_PREFIX_PATH
```

Then you need to install the required system libraries. Please run the corresponding command based on your Linux distribution:

*   **Ubuntu / Debian:**
    ```bash
    sudo apt install vulkan-headers libvulkan-dev libxcb1-dev
    ```
*   **Fedora:**
    ```bash
    sudo dnf install vulkan-headers vulkan-loader-devel libxcb-devel
    ```
*   **Arch Linux / Manjaro:**
    ```bash
    sudo pacman -S vulkan-headers vulkan-icd-loader libxcb
    ```

3. Configuration
  XDG Configuration Path
    - $XDG_CONFIG_HOME/lightwallpaper/config.toml
  Syntax 
    lightwallpaper uses a custom, but very simplistic key = value syntax. The syntax is documented below using comments in a sample configuration file.
  ```toml
  #pictures path
  path = "~/Pictures/assets/wallpapers/"
  #shaders path
  shader = "~/.config/lightwallpaper/shaders/"
  ```

4. Add your wallpaper
**This project does not include any wallpapers.** You will need to add them yourself By default, wallpaper files are placed in the ~/Pictures/assets/wallpapers/ folder at the project root. 

5. Wayland Compositer Setup
  - Niri
  ```bash
  layer-rule {
    match namespace="^lightwallpaper$"
    place-within-backdrop true
  }
  ```
<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage
After completing the steps above, run the following command to launch the project:
```sh
cargo run --release
```

### Modify the wallpaper transition

To modify the wallpaper transition effect, you can write your own fragment shader (shader.frag) in the shader directory and compile it into a SPIR-V binary (.spv). Finally, copy the compiled file to your designated shader path.


**e.g., shader3.frag is an example of a fade-in/fade-out wallpaper transition.** 

<p align="right">(<a href="#readme-top">back to top</a>)</p>




<!-- ROADMAP -->
## Roadmap


- [x] **Core Rendering**
    - [x] Vulkan graphics pipeline initialization
    - [x] Wayland protocol integration & window management
    - [x] Shader loading and compilation
- [x] **Basic Functionality**
    - [x] Image wallpaper rendering
    - [x] Wallpaper transitions (e.g., fade-in/fade-out)
- [ ] Tauri-Gui 
    - [ ]  IndexPage
- [ ] **Planned Features**
    - [x] Configuration file support
    - [x] Play BackGround Music
    - [ ] video wallpaper rendering
    - [ ] Dynamic wallpaper scripting interface
    - [ ] Multi-monitor support
- [ ] **Optimizations**
    - [ ] Memory leak detection and fixes
    - [ ] Rendering performance profiling

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing
Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this project better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement". 
Don't forget to give the project a star! Thanks again!

### Getting Started

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/shader-optimization`)
3. Commit your Changes (`git commit -m 'Optimize vulkan pipeline for smoother transitions'`)
4. Push to the Branch (`git push origin feature/shader-optimization`)
5. Open a Pull Request

> 💡 **Tip:** Since this project involves low-level APIs like Vulkan and Wayland, if you're planning a major feature or architectural change, please open an issue first so we can discuss it before you start coding!``

<!-- ### Top contributors: -->
<!---->
<!-- <a href="https://github.com/othneildrew/Best-README-Template/graphs/contributors"> -->
<!--   <img src="https://contrib.rocks/image?repo=othneildrew/Best-README-Template" alt="contrib.rocks image" /> -->
<!-- </a> -->

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- LICENSE -->
## License

Distributed under the Unlicense License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTACT -->
## Contact

Your Name - [@your_twitter](https://twitter.com/your_username) - email@example.com

Project Link: [https://github.com/your_username/repo_name](https://github.com/your_username/repo_name)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

Use this space to list resources you find helpful and would like to give credit to. I've included a few of my favorites to kick things off!




<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->



[contributors-shield]: https://img.shields.io/github/contributors/othneildrew/Best-README-Template.svg?style=for-the-badge
[contributors-url]: https://github.com/othneildrew/Best-README-Template/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/othneildrew/Best-README-Template.svg?style=for-the-badge
[forks-url]: https://github.com/othneildrew/Best-README-Template/network/members
[stars-shield]: https://img.shields.io/github/stars/othneildrew/Best-README-Template.svg?style=for-the-badge
[stars-url]: https://github.com/othneildrew/Best-README-Template/stargazers
[issues-shield]: https://img.shields.io/github/issues/othneildrew/Best-README-Template.svg?style=for-the-badge
[issues-url]: https://github.com/othneildrew/Best-README-Template/issues
[license-shield]: https://img.shields.io/github/license/othneildrew/Best-README-Template.svg?style=for-the-badge
[license-url]: https://github.com/othneildrew/Best-README-Template/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/othneildrew
[product-screenshot]: images/screenshot.png


[Rust]:https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://rust-lang.org/

[VulkanSDK]:https://img.shields.io/badge/vulkan-A41E22?style=for-the-badge&logo=vulkan&logoColor=white
[VulkanSDK-url]:https://vulkan.lunarg.com/
