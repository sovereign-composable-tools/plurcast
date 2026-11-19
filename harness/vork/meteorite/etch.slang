// An etched slice of iron meteorite: a drawn Widmanstatten pattern.
// Three families of lamellae at 60 degrees to each other (what a cut parallel to an octahedral
// plane shows). Each family is a set of parallel bands of uneven width that start and stop;
// where no band lies, the field is a finer, duller mix. Bands reflect more sharply than the field.
// This is a picture of the pattern, not a simulation of how it forms.
import vork;
import blender.procedurals;

struct Inputs {
    float scale = 55.0;      // bands per metre, roughly
};

float family(float3 p, float angle, float seed, float scale) {
    float c = cos(angle), s = sin(angle);
    float across = (p.x * c + p.z * s) * scale;        // which band
    across += 1.6 * noise(float3(across * 0.11 + seed * 5.0, seed, 0.0), 2.0, 0.5);   // uneven spacing
    float along = (-p.x * s + p.z * c) * scale;        // position along it
    float cell = floor(across);
    float f = across - cell;
    float width = 0.12 + 0.8 * pow(noise(float3(cell * 0.731 + seed, seed * 3.1, 0.0), 1.0, 0.5), 1.6);
    float present = step(0.47, noise(float3(cell * 1.37 + seed * 7.0, along * 0.035, seed), 2.0, 0.5));
    float edge = smoothstep(0.0, 0.06, f) * (1.0 - smoothstep(width - 0.06, width, f));
    return present * edge;
}

Closure surface(ShadingContext ctx, Inputs inputs) {
    Closure c = {};
    float3 p = ctx.POSITION;
    float a = family(p, 0.12, 1.0, inputs.scale);
    float b = family(p, 0.12 + 1.0472, 2.0, inputs.scale * 0.9);
    float d = family(p, 0.12 + 2.0944, 3.0, inputs.scale * 1.1);
    float band = max(a, max(b, d));
    float which = (a >= b && a >= d) ? 0.0 : ((b >= d) ? 1.0 : 2.0);
    float fine = noise(p * 900.0, 2.0, 0.6);
    float3 bright = float3(0.86, 0.85, 0.82) * (0.92 + 0.05 * which);
    float3 field = float3(0.52, 0.51, 0.50) * (0.8 + 0.3 * fine);
    c.base_color = lerp(field, bright, band);
    c.base_metalness = 1.0;
    c.specular_roughness = lerp(0.48, 0.16 + 0.07 * which, band);
    return c;
}
