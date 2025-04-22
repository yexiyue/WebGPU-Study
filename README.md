## 八、WebGPU 基础入门——加载图像纹理

本节将介绍如何加载图像纹理到 WebGPU 中，并在屏幕上绘制出来。我们将使用 `image` crate 来加载图像文件，并将其转换为 WebGPU 可用的纹理格式。本节会使用`egui` 来实现一个简单的 UI 界面。如何集成`egui`，请参考[kaphula/winit-egui-wgpu-template](https://github.com/kaphula/winit-egui-wgpu-template)。

