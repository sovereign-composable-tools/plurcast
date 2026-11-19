// Dune terrain: displaces a flat grid upward and shades it as sand.
// First custom program in this harness. Height is two octaves of ridged noise
// plus a long swell that makes one main ridge across x.
import vork;
import blender.procedurals;

struct Inputs {
    float height = 1.0;
    float3 sand = float3(0.42, 0.26, 0.14);
    float3 dark = float3(0.16, 0.09, 0.05);
};

float terrain(float3 p) {
    float swell = exp(-pow((p.z + 18.0) / 9.0, 2.0)) * 10.0;          // the ridge itself
    float dunes = 1.0 - abs(2.0 * noise(float3(p.x * 0.022, 0.0, p.z * 0.05), 2.0, 0.45) - 1.0);
    float ripples = noise(float3(p.x * 0.6, 0.0, p.z * 0.6), 2.0, 0.5);
    return swell + 3.0 * dunes * dunes + 0.04 * ripples;
}

float3 displace(DisplacementContext ctx, Inputs inputs) {
    return float3(0.0, inputs.height * terrain(ctx.POSITION), 0.0);
}

Closure surface(ShadingContext ctx, Inputs inputs) {
    Closure c = {};
    float fine = noise(ctx.POSITION * 3.0, 2.0, 0.5);
    c.base_color = lerp(inputs.dark, inputs.sand, 0.55 + 0.45 * fine);
    c.specular_roughness = 0.9;
    return c;
}
