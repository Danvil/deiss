DEISS is an audio visualizer inspired by the famous Winamp plugin [Geiss](https://www.geisswerks.com/geiss/).
It listens to an audio stream and renders fluent procedurally generated images similar to an old screensaver.
Geiss was created around 1998 by [Ryan Geiss](https://www.geisswerks.com) and source was [made available](https://github.com/geissomatik/geiss) on GitHub.

DEISS is currently a faithful rewrite of Geiss in Rust using wgpu, winit and rodio.
As computers are wastely more powerful today the "warp map" is currently directly implemented in Rust compared to the [originally heavily optimized assembler code](https://www.geisswerks.com/geiss/secrets.html).
