# Your first vork scene

Edit scene.json, then render again.

```sh
vork render scene.json --out render.png
vork help
```

Shape/revolve examples: `vork help geometry`. Sun angular size: `vork help lights`. Tone curve, grain and movies: `vork help render`.

Open render.png. Try changing the colours, camera target or light positions.
The default `std/final` profile is warmed by setup; use `--profile std/draft` for quicker previews. For interiors and full bounce lighting, use `--profile std/path --set samples=64` to start; raise samples for delivery. See `vork help profiles` for the speed/quality table. No fetched assets are needed.
