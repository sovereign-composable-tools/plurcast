// A brass plate with sand lying on the nodal lines of an idealised square-plate mode:
//   cos(n pi x) cos(m pi y) - cos(m pi x) cos(n pi y) = 0,   x, y in [0, 1] across the plate
//   (free edges are antinodes, so the pattern does not hug the rim).
// Sand density falls off with |f|; grain noise breaks the edge up.
import vork;
import blender.procedurals;

struct Inputs {
    float n = 3.0;
    float m = 5.0;
    float half = 0.25;       // half the plate's side, metres
    float width = 0.26;      // how far from a nodal line sand still lies, in units of f
};

float mode(float3 p, Inputs i) {
    float x = 0.5 * (p.x / i.half + 1.0), y = 0.5 * (p.z / i.half + 1.0);   // 0..1 across the plate
    const float PI = 3.14159265;
    return cos(i.n * PI * x) * cos(i.m * PI * y) - cos(i.m * PI * x) * cos(i.n * PI * y);
}

float sand_amount(float3 p, Inputs i) {
    // density falls from 1 on the line to 0 at `width`; a fine noise decides grain by grain
    float f = abs(mode(p, i));
    float density = 1.0 - smoothstep(0.0, i.width, f);
    float grain = noise(p * 1500.0, 1.0, 0.5);
    return step(0.25 + 0.65 * grain, density * density * 1.15);
}

float3 displace(DisplacementContext ctx, Inputs i) {
    float f = abs(mode(ctx.POSITION, i));
    return float3(0.0, 0.0003 * (1.0 - smoothstep(0.0, i.width, f)), 0.0);
}

Closure surface(ShadingContext ctx, Inputs i) {
    Closure c = {};
    float s = sand_amount(ctx.POSITION, i);
    float tone = noise(ctx.POSITION * 1500.0, 1.0, 0.5);
    c.base_color = lerp(float3(0.78, 0.57, 0.26), float3(0.80, 0.74, 0.62) * (0.75 + 0.35 * tone), s);
    c.base_metalness = 1.0 - s;
    c.specular_roughness = lerp(0.32, 0.95, s);
    return c;
}
