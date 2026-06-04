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
  <a href="https://github.com/othneildrew/Best-README-Template">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>

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

[![Product Name Screen Shot][product-screenshot]](https://example.com)

light-wallpaper is a pure native Vulkan dynamic wallpaper engine for Wayland.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With



* [![Rust][Rust]][Rust-url]
* [![VulkanSDK][VulkanSDK]][VulkanSDK-url]


<p align="right">(<a href="#readme-top">back to top</a>)</p>


``
<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

1. Clone this repository to local:
```sh
git clone https://github.com/UNGGUY/light-wallpaper.git
```

2. Vulkan SDK
The most important component you'll need for developing Vulkan applications is the SDK. It includes the headers, standard validation layers, debugging tools and a loader for the Vulkan functions. The loader looks up the functions in the driver at runtime, similarly to GLEW for OpenGL - if you're familiar with that.

Download the Vulkan SDK installer from [VulkanSDK-url], and add it to your system environment variables after installation.

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

3. Add your wallpaper
**This project does not include any wallpapers.** You will need to add them yourself.By default, wallpaper files are placed in the assets/wallpapers/ folder at the project root.
To change the default wallpaper directory, edit line 40 in main.rs: 
```rust
let directory = Path::new("assets/wallpapers/");
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage
After completing the steps above, run the following command to launch the project:

```sh
cargo run --release
```

Run the following command to check out the branch that includes wallpaper transitions:
```sh
git checkout wManager

cargo run --release
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>




<!-- ROADMAP -->
## Roadmap


- [x] **Core Rendering**
    - [x] Vulkan graphics pipeline initialization
    - [x] Wayland protocol integration & window management
    - [x] Shader loading and compilation
- [x] **Basic Functionality**
    - [x] Image/video wallpaper rendering
    - [x] Wallpaper transitions (e.g., fade-in/fade-out)
- [ ] **Planned Features**
    - [ ] Configuration file support
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

* [Choose an Open Source License](https://choosealicense.com)
* [GitHub Emoji Cheat Sheet](https://www.webpagefx.com/tools/emoji-cheat-sheet)
* [Malven's Flexbox Cheatsheet](https://flexbox.malven.co/)
* [Malven's Grid Cheatsheet](https://grid.malven.co/)
* [Img Shields](https://shields.io)
* [GitHub Pages](https://pages.github.com)
* [Font Awesome](https://fontawesome.com)
* [React Icons](https://react-icons.github.io/react-icons/search)

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
